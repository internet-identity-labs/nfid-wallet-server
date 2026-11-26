use candid::Principal;
use ed25519_dalek::{Signer, SigningKey};
#[cfg(test)]
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::keys::der_encode;
use super::SignerError;

const DELEGATION_DOMAIN_SEPARATOR: &[u8] = b"\x1Aic-request-auth-delegation";

/// Lambda default when `delegationTtl` is not provided (2 hours).
pub const DEFAULT_DELEGATION_TTL_MS: u64 = 2 * 3_600_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delegation {
    /// Session public key, DER-encoded, taken as is.
    pub pubkey: Vec<u8>,
    /// Nanoseconds since epoch.
    pub expiration: u64,
    pub targets: Option<Vec<Principal>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedDelegation {
    pub delegation: Delegation,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DelegationChain {
    pub delegations: Vec<SignedDelegation>,
    /// DER-encoded public key of the signing identity.
    pub public_key: Vec<u8>,
}

/// `new Date(Date.now() + (delegationTtl ?? 2h))` converted to nanoseconds, as `DelegationChain.create` does.
pub fn expiration_ns(now_ms: u64, delegation_ttl_ms: Option<u64>) -> Result<u64, SignerError> {
    now_ms
        .checked_add(delegation_ttl_ms.unwrap_or(DEFAULT_DELEGATION_TTL_MS))
        .and_then(|ms| ms.checked_mul(1_000_000))
        .ok_or(SignerError::ExpirationOverflow)
}

impl Delegation {
    /// Representation-independent hash of the delegation map (`requestIdOf` in @dfinity/agent).
    pub fn representation_independent_hash(&self) -> [u8; 32] {
        let mut fields = vec![
            (sha256(b"pubkey"), sha256(&self.pubkey)),
            (sha256(b"expiration"), sha256(&leb128(self.expiration))),
        ];
        if let Some(targets) = &self.targets {
            let hashes: Vec<u8> = targets.iter().flat_map(|t| sha256(t.as_slice())).collect();
            fields.push((sha256(b"targets"), sha256(&hashes)));
        }
        fields.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut hasher = Sha256::new();
        for (key, value) in fields {
            hasher.update(key);
            hasher.update(value);
        }
        hasher.finalize().into()
    }
}

/// `DelegationChain.create(from, to, expiration, { targets })` without a previous chain.
pub(crate) fn sign(
    key: &SigningKey,
    session_public_key: &[u8],
    expiration: u64,
    targets: Option<Vec<Principal>>,
) -> Result<DelegationChain, SignerError> {
    if session_public_key.is_empty() {
        return Err(SignerError::EmptySessionKey);
    }
    let delegation = Delegation {
        pubkey: session_public_key.to_vec(),
        expiration,
        targets,
    };
    let challenge = [
        DELEGATION_DOMAIN_SEPARATOR,
        &delegation.representation_independent_hash(),
    ]
    .concat();
    let signature = key.sign(&challenge).to_bytes().to_vec();
    Ok(DelegationChain {
        delegations: vec![SignedDelegation {
            delegation,
            signature,
        }],
        public_key: der_encode(key.verifying_key().as_bytes()),
    })
}

#[cfg(test)]
#[derive(Serialize)]
struct DelegationJson {
    expiration: String,
    pubkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    targets: Option<Vec<String>>,
}

#[cfg(test)]
#[derive(Serialize)]
struct SignedDelegationJson {
    delegation: DelegationJson,
    signature: String,
}

#[cfg(test)]
#[derive(Serialize)]
struct DelegationChainJson {
    delegations: Vec<SignedDelegationJson>,
    #[serde(rename = "publicKey")]
    public_key: String,
}

// Only the vector tests compare against the lambda JSON; the canister returns candid.
#[cfg(test)]
impl DelegationChain {
    /// Same string as the lambda response `JSON.stringify(chain.toJSON())`: field order, lowercase
    /// hex, unpadded hex expiration and uppercase hex targets (`Principal.toHex`).
    pub fn to_json(&self) -> String {
        let json = DelegationChainJson {
            delegations: self
                .delegations
                .iter()
                .map(|d| SignedDelegationJson {
                    delegation: DelegationJson {
                        expiration: format!("{:x}", d.delegation.expiration),
                        pubkey: hex::encode(&d.delegation.pubkey),
                        targets: d.delegation.targets.as_ref().map(|targets| {
                            targets
                                .iter()
                                .map(|t| hex::encode_upper(t.as_slice()))
                                .collect()
                        }),
                    },
                    signature: hex::encode(&d.signature),
                })
                .collect(),
            public_key: hex::encode(&self.public_key),
        };
        serde_json::to_string(&json).expect("serializing strings cannot fail")
    }
}

fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn leb128(mut value: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(10);
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return out;
        }
        out.push(byte | 0x80);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leb128_encodes_unsigned() {
        assert_eq!(leb128(0), [0x00]);
        assert_eq!(leb128(127), [0x7f]);
        assert_eq!(leb128(128), [0x80, 0x01]);
        assert_eq!(leb128(624_485), [0xe5, 0x8e, 0x26]);
        assert_eq!(leb128(u64::MAX).len(), 10);
    }

    #[test]
    fn expiration_uses_default_ttl_and_rejects_overflow() {
        assert_eq!(
            expiration_ns(1_000, None),
            Ok((1_000 + DEFAULT_DELEGATION_TTL_MS) * 1_000_000)
        );
        assert_eq!(expiration_ns(1_000, Some(5)), Ok(1_005_000_000));
        assert_eq!(
            expiration_ns(u64::MAX / 1_000_000, Some(1)),
            Err(SignerError::ExpirationOverflow)
        );
    }

    #[test]
    fn rejects_empty_session_key() {
        let key = SigningKey::from_bytes(&[1; 32]);
        assert_eq!(
            sign(&key, &[], 1, None).unwrap_err(),
            SignerError::EmptySessionKey
        );
    }
}
