use aes::Aes256;
use ecb::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyInit};
use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::SignerError;

/// DER prefix of an Ed25519 SubjectPublicKeyInfo (`Ed25519PublicKey.derEncode` in @dfinity/identity).
const ED25519_DER_PREFIX: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

/// Lambda environment secrets.
#[derive(Clone)]
pub struct Salts {
    /// `ECDSA_SALT`
    pub ecdsa_salt: String,
    /// `ANONYMOUS_SALT`
    pub anonymous_salt: String,
}

/// A record of `signer_ic`, as written by the lambda.
#[derive(Clone, Debug)]
pub struct StoredKeyPair {
    /// Hex of the DER-encoded Ed25519 public key.
    pub public_key: String,
    /// Hex of AES-256-ECB(hex of the 64-byte tweetnacl secret key).
    pub private_key_encrypted: String,
}

/// `Encryptor.sha2(value + ECDSA_SALT)`. The lambda turns the hex digest back into bytes
/// (`Buffer.from(hex, "hex")` / `hexStringToUint8Array`), so the raw digest is used directly.
fn salted_sha256(value: &str, ecdsa_salt: &str) -> Zeroizing<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher.update(ecdsa_salt.as_bytes());
    Zeroizing::new(hasher.finalize().into())
}

/// seed = sha256(root + domain + ANONYMOUS_SALT + ECDSA_SALT), key = `Ed25519KeyIdentity.generate(seed)`
/// (tweetnacl `sign.keyPair.fromSeed`, i.e. a standard RFC 8032 seed).
pub(crate) fn anonymous_signing_key(
    root_principal: &str,
    domain: &str,
    salts: &Salts,
) -> SigningKey {
    let value = Zeroizing::new(format!("{root_principal}{domain}{}", salts.anonymous_salt));
    let seed = salted_sha256(&value, &salts.ecdsa_salt);
    SigningKey::from_bytes(&seed)
}

/// Decrypts a stored global key: AES-256-ECB/PKCS7 with key sha256(root + ECDSA_SALT), plaintext is the
/// UTF-8 hex of the tweetnacl secret key (seed || public key).
pub(crate) fn decrypt_global_signing_key(
    root_principal: &str,
    stored: &StoredKeyPair,
    ecdsa_salt: &str,
) -> Result<SigningKey, SignerError> {
    let aes_key = salted_sha256(root_principal, ecdsa_salt);
    let ciphertext = hex::decode(&stored.private_key_encrypted)
        .map_err(|_| SignerError::InvalidHex("private_key_encrypted"))?;
    let plaintext = Zeroizing::new(
        ecb::Decryptor::<Aes256>::new(aes_key.as_ref().into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
            .map_err(|_| SignerError::Decryption)?,
    );
    let secret_hex = std::str::from_utf8(&plaintext).map_err(|_| SignerError::InvalidPrivateKey)?;
    let secret =
        Zeroizing::new(hex::decode(secret_hex).map_err(|_| SignerError::InvalidPrivateKey)?);
    // tweetnacl.sign.detached accepts only 64-byte secret keys; every lambda version stored this format.
    if secret.len() != 64 {
        return Err(SignerError::InvalidPrivateKey);
    }
    let seed: Zeroizing<[u8; 32]> =
        Zeroizing::new(secret[..32].try_into().expect("length checked"));
    let key = SigningKey::from_bytes(&seed);

    let stored_public_key =
        hex::decode(&stored.public_key).map_err(|_| SignerError::InvalidHex("public_key"))?;
    let stored_public_key = der_decode(&stored_public_key)?;
    // tweetnacl signs with the public key embedded in the secret key and the chain carries the stored
    // one; both must be the real public key for the output to match (and to be valid at all).
    let derived = key.verifying_key();
    if derived.as_bytes() != &secret[32..] || derived.as_bytes() != stored_public_key {
        return Err(SignerError::KeyMismatch);
    }
    Ok(key)
}

pub(crate) fn der_encode(raw_public_key: &[u8; 32]) -> Vec<u8> {
    [ED25519_DER_PREFIX.as_slice(), raw_public_key].concat()
}

fn der_decode(der: &[u8]) -> Result<&[u8], SignerError> {
    match der.strip_prefix(ED25519_DER_PREFIX.as_slice()) {
        Some(raw) if raw.len() == 32 => Ok(raw),
        _ => Err(SignerError::InvalidPublicKey),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::BlockEncryptMut;

    fn salts() -> Salts {
        Salts {
            ecdsa_salt: "ecdsa".into(),
            anonymous_salt: "anon".into(),
        }
    }

    fn encrypt(root: &str, ecdsa_salt: &str, plaintext: &str) -> String {
        let key = salted_sha256(root, ecdsa_salt);
        hex::encode(
            ecb::Encryptor::<Aes256>::new(key.as_ref().into())
                .encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes()),
        )
    }

    fn stored_for(root: &str, key: &SigningKey) -> StoredKeyPair {
        let secret = [key.to_bytes().as_slice(), key.verifying_key().as_bytes()].concat();
        StoredKeyPair {
            public_key: hex::encode(der_encode(key.verifying_key().as_bytes())),
            private_key_encrypted: encrypt(root, "ecdsa", &hex::encode(secret)),
        }
    }

    #[test]
    fn anonymous_key_depends_on_every_input() {
        let base = anonymous_signing_key("root", "https://a.com", &salts()).verifying_key();
        assert_eq!(
            base,
            anonymous_signing_key("root", "https://a.com", &salts()).verifying_key()
        );
        assert_ne!(
            base,
            anonymous_signing_key("root2", "https://a.com", &salts()).verifying_key()
        );
        assert_ne!(
            base,
            anonymous_signing_key("root", "https://a.com/", &salts()).verifying_key()
        );
        let other = Salts {
            anonymous_salt: "anon2".into(),
            ..salts()
        };
        assert_ne!(
            base,
            anonymous_signing_key("root", "https://a.com", &other).verifying_key()
        );
    }

    #[test]
    fn decrypts_stored_key() {
        let key = SigningKey::from_bytes(&[7; 32]);
        let decrypted =
            decrypt_global_signing_key("root", &stored_for("root", &key), "ecdsa").unwrap();
        assert_eq!(decrypted.to_bytes(), key.to_bytes());
    }

    #[test]
    fn rejects_corrupted_and_inconsistent_records() {
        let key = SigningKey::from_bytes(&[7; 32]);
        let stored = stored_for("root", &key);

        let bad_hex = StoredKeyPair {
            private_key_encrypted: "zz".into(),
            ..stored.clone()
        };
        assert_eq!(
            decrypt_global_signing_key("root", &bad_hex, "ecdsa").unwrap_err(),
            SignerError::InvalidHex("private_key_encrypted")
        );

        let truncated = StoredKeyPair {
            private_key_encrypted: stored.private_key_encrypted[..30].into(),
            ..stored.clone()
        };
        assert_eq!(
            decrypt_global_signing_key("root", &truncated, "ecdsa").unwrap_err(),
            SignerError::Decryption
        );

        let seed_only = StoredKeyPair {
            private_key_encrypted: encrypt("root", "ecdsa", &hex::encode(key.to_bytes())),
            ..stored.clone()
        };
        assert_eq!(
            decrypt_global_signing_key("root", &seed_only, "ecdsa").unwrap_err(),
            SignerError::InvalidPrivateKey
        );

        let raw_public_key = StoredKeyPair {
            public_key: hex::encode(key.verifying_key().as_bytes()),
            ..stored.clone()
        };
        assert_eq!(
            decrypt_global_signing_key("root", &raw_public_key, "ecdsa").unwrap_err(),
            SignerError::InvalidPublicKey
        );

        let other = SigningKey::from_bytes(&[8; 32]);
        let foreign_public_key = StoredKeyPair {
            public_key: hex::encode(der_encode(other.verifying_key().as_bytes())),
            ..stored.clone()
        };
        assert_eq!(
            decrypt_global_signing_key("root", &foreign_public_key, "ecdsa").unwrap_err(),
            SignerError::KeyMismatch
        );

        let foreign_embedded =
            [key.to_bytes().as_slice(), other.verifying_key().as_bytes()].concat();
        let bad_embedded = StoredKeyPair {
            private_key_encrypted: encrypt("root", "ecdsa", &hex::encode(foreign_embedded)),
            ..stored
        };
        assert_eq!(
            decrypt_global_signing_key("root", &bad_embedded, "ecdsa").unwrap_err(),
            SignerError::KeyMismatch
        );
    }
}
