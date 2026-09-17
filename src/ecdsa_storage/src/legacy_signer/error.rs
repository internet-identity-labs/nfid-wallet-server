use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignerError {
    EmptySessionKey,
    /// Expiration does not fit into u64 nanoseconds.
    ExpirationOverflow,
    /// The stored field is not valid hex.
    InvalidHex(&'static str),
    /// AES-256-ECB decryption failed (wrong root principal or salt, or corrupted data).
    Decryption,
    /// The decrypted private key is not a hex-encoded 64-byte tweetnacl secret key.
    InvalidPrivateKey,
    /// The stored public key is not a DER-encoded Ed25519 key.
    InvalidPublicKey,
    /// The stored public key, the one embedded in the secret key and the derived one differ.
    KeyMismatch,
}

impl fmt::Display for SignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignerError::EmptySessionKey => write!(f, "Empty session public key"),
            SignerError::ExpirationOverflow => write!(f, "Delegation expiration overflow"),
            SignerError::InvalidHex(field) => write!(f, "Invalid hex in {field}"),
            SignerError::Decryption => write!(f, "Decryption error"),
            SignerError::InvalidPrivateKey => write!(f, "Invalid private key"),
            SignerError::InvalidPublicKey => write!(f, "Invalid public key"),
            SignerError::KeyMismatch => write!(f, "Stored key pair is inconsistent"),
        }
    }
}

impl std::error::Error for SignerError {}
