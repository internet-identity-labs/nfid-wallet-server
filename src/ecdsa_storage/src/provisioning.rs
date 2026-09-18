//! Delivery of the lambda salts without exposing them in ingress messages.
//!
//! The canister keeps an X25519 key; the operator encrypts the salts to it with an ephemeral X25519
//! key and ChaCha20-Poly1305 (scripts/provision-salts.mjs):
//!   key = sha256(KDF_DOMAIN || shared_secret || ephemeral_public_key || canister_public_key)
//!   ciphertext = ChaCha20-Poly1305(key, nonce, JSON {ecdsa_salt, anonymous_salt}, aad = canister id)

use candid::{CandidType, Deserialize, Principal};
use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::ChaCha20Poly1305;
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroizing;

use crate::legacy_signer::Salts;

const KDF_DOMAIN: &[u8] = b"nfid-ecdsa-storage/provision-salts/v1";
const FINGERPRINT_DOMAIN: &[u8] = b"nfid-ecdsa-storage/salts-fingerprint/v1";

/// Hex-encoded sealed salts.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SealedSalts {
    pub ephemeral_public_key: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Deserialize)]
struct SaltsPayload {
    ecdsa_salt: String,
    anonymous_salt: String,
}

pub fn public_key(secret: &[u8; 32]) -> [u8; 32] {
    PublicKey::from(&StaticSecret::from(*secret)).to_bytes()
}

pub fn open_salts(
    secret: &[u8; 32],
    canister_id: &Principal,
    sealed: &SealedSalts,
) -> Result<Salts, String> {
    let ephemeral: [u8; 32] = hex::decode(&sealed.ephemeral_public_key)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or("Invalid ephemeral public key")?;
    let nonce: [u8; 12] = hex::decode(&sealed.nonce)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or("Invalid nonce")?;
    let mut plaintext =
        Zeroizing::new(hex::decode(&sealed.ciphertext).map_err(|_| "Invalid ciphertext")?);

    let static_secret = StaticSecret::from(*secret);
    let shared = static_secret.diffie_hellman(&PublicKey::from(ephemeral));
    if !shared.was_contributory() {
        return Err("Invalid ephemeral public key".into());
    }
    let key = derive_key(
        shared.as_bytes(),
        &ephemeral,
        &PublicKey::from(&static_secret).to_bytes(),
    );
    ChaCha20Poly1305::new(key.as_ref().into())
        .decrypt_in_place(&nonce.into(), canister_id.as_slice(), &mut *plaintext)
        .map_err(|_| "Cannot decrypt the salts")?;
    let payload: SaltsPayload =
        serde_json::from_slice(&plaintext).map_err(|_| "Invalid salts payload")?;
    if payload.ecdsa_salt.is_empty() || payload.anonymous_salt.is_empty() {
        return Err("Empty salt".into());
    }
    Ok(Salts {
        ecdsa_salt: payload.ecdsa_salt,
        anonymous_salt: payload.anonymous_salt,
    })
}

/// Short non-reversible identifier to confirm which salts are loaded.
pub fn fingerprint(salts: &Salts) -> String {
    let mut hasher = Sha256::new();
    hasher.update(FINGERPRINT_DOMAIN);
    for salt in [&salts.ecdsa_salt, &salts.anonymous_salt] {
        hasher.update((salt.len() as u64).to_be_bytes());
        hasher.update(salt.as_bytes());
    }
    hex::encode(&hasher.finalize()[..8])
}

fn derive_key(
    shared: &[u8; 32],
    ephemeral: &[u8; 32],
    recipient: &[u8; 32],
) -> Zeroizing<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(KDF_DOMAIN);
    hasher.update(shared);
    hasher.update(ephemeral);
    hasher.update(recipient);
    Zeroizing::new(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANISTER: &str = "zhr63-daaaa-aaaap-qbh4q-cai";

    fn seal(recipient: &[u8; 32], canister_id: &Principal, payload: &str) -> SealedSalts {
        let ephemeral = StaticSecret::from([9u8; 32]);
        let ephemeral_public = PublicKey::from(&ephemeral).to_bytes();
        let shared = ephemeral.diffie_hellman(&PublicKey::from(*recipient));
        let key = derive_key(shared.as_bytes(), &ephemeral_public, recipient);
        let nonce = [3u8; 12];
        let mut ciphertext = payload.as_bytes().to_vec();
        ChaCha20Poly1305::new(key.as_ref().into())
            .encrypt_in_place(&nonce.into(), canister_id.as_slice(), &mut ciphertext)
            .unwrap();
        SealedSalts {
            ephemeral_public_key: hex::encode(ephemeral_public),
            nonce: hex::encode(nonce),
            ciphertext: hex::encode(ciphertext),
        }
    }

    #[test]
    fn opens_sealed_salts() {
        let secret = [7u8; 32];
        let canister = Principal::from_text(CANISTER).unwrap();
        let sealed = seal(
            &public_key(&secret),
            &canister,
            r#"{"ecdsa_salt":"e","anonymous_salt":"a"}"#,
        );
        let Ok(salts) = open_salts(&secret, &canister, &sealed) else {
            panic!("cannot open sealed salts");
        };
        assert_eq!(
            (salts.ecdsa_salt.as_str(), salts.anonymous_salt.as_str()),
            ("e", "a")
        );
    }

    /// The lambda seals the salts (sms-sender-serverless canister-migration.service.ts) and this canister
    /// opens them, so both implementations are checked against one envelope.
    #[test]
    fn opens_an_envelope_sealed_by_the_lambda() {
        #[derive(serde::Deserialize)]
        struct Fixture {
            canister_id: String,
            recipient_secret: String,
            recipient_public_key: String,
            salts: FixtureSalts,
            fingerprint: String,
            sealed: SealedSalts,
        }
        #[derive(serde::Deserialize)]
        struct FixtureSalts {
            ecdsa_salt: String,
            anonymous_salt: String,
        }

        let fixture: Fixture = serde_json::from_str(include_str!(
            "../tests/fixtures/sealed_salts_from_lambda.json"
        ))
        .unwrap();
        let secret: [u8; 32] = hex::decode(&fixture.recipient_secret)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            hex::encode(public_key(&secret)),
            fixture.recipient_public_key
        );

        let canister = Principal::from_text(&fixture.canister_id).unwrap();
        let Ok(salts) = open_salts(&secret, &canister, &fixture.sealed) else {
            panic!("cannot open the envelope sealed by the lambda");
        };
        assert_eq!(salts.ecdsa_salt, fixture.salts.ecdsa_salt);
        assert_eq!(salts.anonymous_salt, fixture.salts.anonymous_salt);
        assert_eq!(fingerprint(&salts), fixture.fingerprint);

        // The envelope is bound to this canister only.
        let other = Principal::from_text("zhr63-daaaa-aaaap-qbh4q-cai").unwrap();
        assert!(open_salts(&secret, &other, &fixture.sealed).is_err());
    }

    #[test]
    fn rejects_other_canister_key_or_tampering() {
        let secret = [7u8; 32];
        let canister = Principal::from_text(CANISTER).unwrap();
        let payload = r#"{"ecdsa_salt":"e","anonymous_salt":"a"}"#;
        let sealed = seal(&public_key(&secret), &canister, payload);

        let other_canister = Principal::from_text("aaaaa-aa").unwrap();
        assert!(open_salts(&secret, &other_canister, &sealed).is_err());
        assert!(open_salts(&[8u8; 32], &canister, &sealed).is_err());

        let mut tampered = sealed.clone();
        tampered.ciphertext.replace_range(0..2, "00");
        assert!(open_salts(&secret, &canister, &tampered).is_err());

        let low_order = SealedSalts {
            ephemeral_public_key: hex::encode([0u8; 32]),
            ..sealed
        };
        assert_eq!(
            open_salts(&secret, &canister, &low_order).err().as_deref(),
            Some("Invalid ephemeral public key")
        );
    }

    #[test]
    fn rejects_empty_salts() {
        let secret = [7u8; 32];
        let canister = Principal::from_text(CANISTER).unwrap();
        let sealed = seal(
            &public_key(&secret),
            &canister,
            r#"{"ecdsa_salt":"","anonymous_salt":"a"}"#,
        );
        assert_eq!(
            open_salts(&secret, &canister, &sealed).err().as_deref(),
            Some("Empty salt")
        );
    }

    #[test]
    fn fingerprint_distinguishes_salt_order() {
        let a = Salts {
            ecdsa_salt: "x".into(),
            anonymous_salt: "y".into(),
        };
        let b = Salts {
            ecdsa_salt: "y".into(),
            anonymous_salt: "x".into(),
        };
        assert_ne!(fingerprint(&a), fingerprint(&b));
    }
}
