//! Delegation signing of the legacy AWS lambda (`sms-sender-serverless`, `GlobalSignService`),
//! ported to produce bit-identical results.
//!
//! Serves accounts with anchor < 200_000_000 only. Nothing is generated here:
//! - global delegations are signed with the Ed25519 key already stored (encrypted) in `signer_ic`;
//! - anonymous delegations are signed with the key deterministically derived from the salts.
//!
//! Authorization (caller -> root principal) and the source of the stored keys are the caller's job.

mod delegation;
mod error;
mod keys;
#[cfg(test)]
mod lambda_vectors_tests;

use candid::Principal;

pub use delegation::{expiration_ns, DelegationChain};
pub use error::SignerError;
pub use keys::{Salts, StoredKeyPair};

/// `GlobalSignService.getAnonymousDelegation`: a delegation from the anonymous key of
/// (`root_principal`, `domain`) to `session_public_key`. Empty `targets` means unrestricted.
pub fn sign_anonymous_delegation(
    root_principal: &str,
    domain: &str,
    salts: &Salts,
    session_public_key: &[u8],
    targets: &[Principal],
    expiration_ns: u64,
) -> Result<DelegationChain, SignerError> {
    let key = keys::anonymous_signing_key(root_principal, domain, salts);
    delegation::sign(
        &key,
        session_public_key,
        expiration_ns,
        optional_targets(targets),
    )
}

/// DER-encoded public key of the anonymous identity of (`root_principal`, `domain`).
pub fn anonymous_public_key(root_principal: &str, domain: &str, salts: &Salts) -> Vec<u8> {
    keys::der_encode(
        keys::anonymous_signing_key(root_principal, domain, salts)
            .verifying_key()
            .as_bytes(),
    )
}

/// `GlobalSignService.sign` for `Chain.IC` of an already registered user: a delegation from the
/// user's global key to `session_public_key`. Empty `targets` means unrestricted.
pub fn sign_global_delegation(
    root_principal: &str,
    stored: &StoredKeyPair,
    ecdsa_salt: &str,
    session_public_key: &[u8],
    targets: &[Principal],
    expiration_ns: u64,
) -> Result<DelegationChain, SignerError> {
    let key = keys::decrypt_global_signing_key(root_principal, stored, ecdsa_salt)?;
    delegation::sign(
        &key,
        session_public_key,
        expiration_ns,
        optional_targets(targets),
    )
}

// The lambda omits `targets` from the delegation when none are requested.
fn optional_targets(targets: &[Principal]) -> Option<Vec<Principal>> {
    (!targets.is_empty()).then(|| targets.to_vec())
}
