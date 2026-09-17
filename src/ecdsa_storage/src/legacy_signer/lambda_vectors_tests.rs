//! Checks the port against vectors produced by the lambda itself
//! (sms-sender-serverless/tests/unit/services/legacy-signer-vectors.test.ts).
//!
//! Regenerate: in sms-sender-serverless run
//!   UPDATE_LEGACY_SIGNER_VECTORS=1 LEGACY_SIGNER_VECTORS_OUT=<identity-manager>/src/ecdsa_storage/tests/vectors/lambda_vectors.json npm run unit-test

use super::{
    anonymous_public_key, expiration_ns, sign_anonymous_delegation, sign_global_delegation,
    DelegationChain, Salts, SignerError, StoredKeyPair,
};
use candid::Principal;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;

#[derive(Deserialize)]
struct Vectors {
    libraries: serde_json::Value,
    now_ms: u64,
    anonymous: Vec<AnonymousCase>,
    global: Vec<GlobalCase>,
    global_errors: Vec<GlobalCase>,
}

#[derive(Deserialize)]
struct SaltsJson {
    ecdsa_salt: String,
    anonymous_salt: String,
}

impl SaltsJson {
    fn to_salts(&self) -> Salts {
        Salts {
            ecdsa_salt: self.ecdsa_salt.clone(),
            anonymous_salt: self.anonymous_salt.clone(),
        }
    }
}

#[derive(Deserialize)]
struct AnonymousCase {
    name: String,
    salts: SaltsJson,
    root_principal: String,
    domain: String,
    session_public_key: String,
    targets: Vec<String>,
    delegation_ttl_ms: Option<u64>,
    expected_principal: String,
    expected_chain: String,
}

#[derive(Deserialize)]
struct GlobalCase {
    name: String,
    salts: SaltsJson,
    root_principal: String,
    public_key: String,
    private_key_encrypted: String,
    session_public_key: String,
    targets: Vec<String>,
    delegation_ttl_ms: Option<u64>,
    expected_principal: Option<String>,
    expected_chain: Option<String>,
}

fn parse_targets(targets: &[String]) -> Vec<Principal> {
    targets
        .iter()
        .map(|t| Principal::from_text(t).unwrap())
        .collect()
}

impl GlobalCase {
    fn stored(&self) -> StoredKeyPair {
        StoredKeyPair {
            public_key: self.public_key.clone(),
            private_key_encrypted: self.private_key_encrypted.clone(),
        }
    }

    fn targets(&self) -> Vec<Principal> {
        parse_targets(&self.targets)
    }
}

fn vectors() -> Vectors {
    let raw = include_str!("../../tests/vectors/lambda_vectors.json");
    let vectors: Vectors = serde_json::from_str(raw).expect("valid vectors file");
    assert_eq!(
        vectors.libraries["@dfinity/identity"], "0.18.1",
        "vectors must come from the deployed lambda"
    );
    vectors
}

/// Independent check that the chain is a valid Ed25519 delegation (not only equal to the lambda output).
fn assert_valid_signature(chain: &DelegationChain, name: &str) {
    let raw_public_key: [u8; 32] = chain.public_key[12..].try_into().unwrap();
    let verifying_key = VerifyingKey::from_bytes(&raw_public_key).unwrap();
    let signed = &chain.delegations[0];
    let challenge = [
        b"\x1Aic-request-auth-delegation".as_slice(),
        &signed.delegation.representation_independent_hash(),
    ]
    .concat();
    let signature = Signature::from_slice(&signed.signature).unwrap();
    verifying_key
        .verify(&challenge, &signature)
        .unwrap_or_else(|e| panic!("{name}: invalid signature: {e}"));
}

fn principal_text(der: &[u8]) -> String {
    Principal::self_authenticating(der).to_text()
}

#[test]
fn covers_dev_and_distinct_salts() {
    let v = vectors();
    for prefix in ["dev/", "distinct/"] {
        assert!(v.anonymous.iter().any(|c| c.name.starts_with(prefix)));
        assert!(v.global.iter().any(|c| c.name.starts_with(prefix)));
    }
    let distinct = v
        .anonymous
        .iter()
        .find(|c| c.name.starts_with("distinct/"))
        .unwrap();
    assert_ne!(distinct.salts.ecdsa_salt, distinct.salts.anonymous_salt);
}

#[test]
fn anonymous_delegations_match_lambda() {
    let v = vectors();
    for case in &v.anonymous {
        let session_public_key = hex::decode(&case.session_public_key).unwrap();
        let expiration = expiration_ns(v.now_ms, case.delegation_ttl_ms).unwrap();
        let salts = case.salts.to_salts();

        let chain = sign_anonymous_delegation(
            &case.root_principal,
            &case.domain,
            &salts,
            &session_public_key,
            &parse_targets(&case.targets),
            expiration,
        )
        .unwrap_or_else(|e| panic!("{}: {e}", case.name));

        assert_eq!(
            chain.to_json(),
            case.expected_chain,
            "{}: chain differs from lambda",
            case.name
        );
        assert_eq!(
            principal_text(&chain.public_key),
            case.expected_principal,
            "{}",
            case.name
        );
        assert_eq!(
            anonymous_public_key(&case.root_principal, &case.domain, &salts),
            chain.public_key,
            "{}",
            case.name
        );
        assert_valid_signature(&chain, &case.name);
    }
}

#[test]
fn global_delegations_match_lambda() {
    let v = vectors();
    for case in &v.global {
        let session_public_key = hex::decode(&case.session_public_key).unwrap();
        let expiration = expiration_ns(v.now_ms, case.delegation_ttl_ms).unwrap();

        let chain = sign_global_delegation(
            &case.root_principal,
            &case.stored(),
            &case.salts.ecdsa_salt,
            &session_public_key,
            &case.targets(),
            expiration,
        )
        .unwrap_or_else(|e| panic!("{}: {e}", case.name));

        assert_eq!(
            Some(chain.to_json()),
            case.expected_chain,
            "{}: chain differs from lambda",
            case.name
        );
        assert_eq!(
            Some(principal_text(&chain.public_key)),
            case.expected_principal,
            "{}",
            case.name
        );
        assert_valid_signature(&chain, &case.name);
    }
}

#[test]
fn rejects_what_lambda_rejects() {
    let v = vectors();
    assert!(!v.global_errors.is_empty());
    for case in &v.global_errors {
        let result = sign_global_delegation(
            &case.root_principal,
            &case.stored(),
            &case.salts.ecdsa_salt,
            &hex::decode(&case.session_public_key).unwrap(),
            &case.targets(),
            expiration_ns(v.now_ms, case.delegation_ttl_ms).unwrap(),
        );
        let error = result.expect_err(&case.name);
        // Wrong AES key: almost always a padding error, otherwise the decrypted garbage is rejected.
        assert!(
            matches!(
                error,
                SignerError::Decryption | SignerError::InvalidPrivateKey | SignerError::KeyMismatch
            ),
            "{}: unexpected {error:?}",
            case.name
        );
    }
}

#[test]
fn wrong_salt_changes_anonymous_identity() {
    let v = vectors();
    let case = &v.anonymous[0];
    let wrong = Salts {
        ecdsa_salt: format!("{}x", case.salts.ecdsa_salt),
        ..case.salts.to_salts()
    };
    let key = anonymous_public_key(&case.root_principal, &case.domain, &wrong);
    assert_ne!(principal_text(&key), case.expected_principal);
}
