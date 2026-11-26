use candid::{CandidType, Deserialize, Principal};
use ic_cbor::{parse_cbor_principals_array, CertificateToCbor, HashTreeToCbor};
use ic_certificate_verification::VerifyCertificate;
use ic_certification::hash_tree::SubtreeLookupResult;
use ic_certification::{Certificate, HashTree, LookupResult};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

/// IC mainnet root public key, DER-encoded (`IC_ROOT_KEY` in agent-js).
pub const MAINNET_ROOT_KEY_DER: &str = "308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100814c0e6ec71fab583b08bd81373c255c3c371b2e84863c98a4f1e08b74235d14fb5d9c0cd546d9685f913a0c0b2cc5341583bf4b4392e467db96d65b9bb4cb717112f8472e0d5a4d14505ffd7484b01291091c5f87b98883463f98091a0baaae";

/// Maximum difference between the certificate time and the canister time.
pub const MAX_CERTIFICATE_TIME_OFFSET_NS: u128 = 5 * 60 * 1_000_000_000;

/// Response of the Identity Manager `get_root_certified`, requested by the caller itself.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CertifiedRoot {
    pub root: String,
    pub certificate: ByteBuf,
    pub witness: ByteBuf,
}

/// Checks that `certified` proves `caller` is an access point of the account `certified.root`:
/// the certificate is signed by the IC for the Identity Manager and is fresh, the witness matches its
/// certified data, and the witness maps `caller` to sha256(root).
///
/// The Identity Manager builds this witness only for the access point calling `get_root_certified`,
/// after its 2FA check, so this gives the same guarantee as calling the Identity Manager as the user.
pub fn verify_certified_root(
    im_canister: &Principal,
    root_key_der: &[u8],
    caller: &Principal,
    certified: &CertifiedRoot,
    now_ns: u128,
) -> Result<(), String> {
    let certificate = Certificate::from_cbor(&certified.certificate)
        .map_err(|e| format!("Invalid certificate: {e}"))?;
    verify_certificate(&certificate, im_canister, root_key_der, now_ns)
        .map_err(|e| format!("Certificate verification failed: {e}"))?;

    let certified_data_path: [&[u8]; 3] = [b"canister", im_canister.as_slice(), b"certified_data"];
    let LookupResult::Found(certified_data) = certificate.tree.lookup_path(&certified_data_path)
    else {
        return Err("Certificate has no certified data of the Identity Manager".into());
    };

    let witness =
        HashTree::from_cbor(&certified.witness).map_err(|e| format!("Invalid witness: {e}"))?;
    if witness.digest().as_slice() != certified_data {
        return Err("Witness does not match the certified data".into());
    }

    let caller_text = caller.to_text();
    let LookupResult::Found(root_hash) = witness.lookup_path(&[caller_text.as_bytes()]) else {
        return Err("Caller is not certified by the Identity Manager".into());
    };
    if root_hash != Sha256::digest(certified.root.as_bytes()).as_slice() {
        return Err("Certified root does not match".into());
    }
    Ok(())
}

/// `VerifyCertificate::verify` of ic-certificate-verification 2.6 expects the subnet delegation to list
/// canister ranges under `/subnet/<subnet_id>/canister_ranges`, while API v3 (used by @icp-sdk/core)
/// returns them under `/canister_ranges/<subnet_id>/<shard>`. The newer crate versions that support it
/// pull wasm-bindgen into the canister, so the delegation is resolved here and both BLS signatures are
/// still checked by the crate: the delegation with the root key, the certificate with the subnet key.
fn verify_certificate(
    certificate: &Certificate,
    canister: &Principal,
    root_key_der: &[u8],
    now_ns: u128,
) -> Result<(), String> {
    let signing_key = match &certificate.delegation {
        None => root_key_der.to_vec(),
        Some(delegation) => {
            let delegation_certificate = Certificate::from_cbor(&delegation.certificate)
                .map_err(|e| format!("Invalid delegation: {e}"))?;
            if delegation_certificate.delegation.is_some() {
                return Err("Nested delegations are not allowed".into());
            }
            // Only the signature: delegations are not reissued regularly, so their time is not checked.
            let delegation_time = certificate_time(&delegation_certificate)?;
            delegation_certificate
                .verify(canister.as_slice(), root_key_der, &delegation_time, &0)
                .map_err(|e| format!("Invalid delegation: {e}"))?;

            let ranges = canister_ranges(&delegation_certificate, &delegation.subnet_id)?;
            if !ranges
                .iter()
                .any(|(start, end)| canister >= start && canister <= end)
            {
                return Err("The subnet delegation does not cover the canister".into());
            }

            let public_key_path: [&[u8]; 3] = [b"subnet", &delegation.subnet_id, b"public_key"];
            let LookupResult::Found(subnet_key) =
                delegation_certificate.tree.lookup_path(&public_key_path)
            else {
                return Err("The subnet delegation has no public key".into());
            };
            subnet_key.to_vec()
        }
    };

    Certificate {
        tree: certificate.tree.clone(),
        signature: certificate.signature.clone(),
        delegation: None,
    }
    .verify(
        canister.as_slice(),
        &signing_key,
        &now_ns,
        &MAX_CERTIFICATE_TIME_OFFSET_NS,
    )
    .map_err(|e| e.to_string())
}

fn canister_ranges(
    delegation_certificate: &Certificate,
    subnet_id: &[u8],
) -> Result<Vec<(Principal, Principal)>, String> {
    let parse = |cbor: &[u8]| {
        parse_cbor_principals_array(cbor).map_err(|e| format!("Invalid canister ranges: {e}"))
    };
    let new_path: [&[u8]; 2] = [b"canister_ranges", subnet_id];
    let ranges = match delegation_certificate.tree.lookup_subtree(&new_path) {
        SubtreeLookupResult::Found(shards) => {
            let mut ranges = Vec::new();
            for path in shards.list_paths() {
                let Some(shard) = path.first() else { continue };
                if let LookupResult::Found(cbor) = shards.lookup_path([shard.as_bytes()]) {
                    ranges.extend(parse(cbor)?);
                }
            }
            ranges
        }
        _ => {
            let old_path: [&[u8]; 3] = [b"subnet", subnet_id, b"canister_ranges"];
            match delegation_certificate.tree.lookup_path(&old_path) {
                LookupResult::Found(cbor) => parse(cbor)?,
                _ => return Err("The subnet delegation has no canister ranges".into()),
            }
        }
    };
    Ok(ranges)
}

fn certificate_time(certificate: &Certificate) -> Result<u128, String> {
    let LookupResult::Found(mut encoded) = certificate.tree.lookup_path([b"time".as_slice()])
    else {
        return Err("Certificate has no time".into());
    };
    let mut time = 0u128;
    for shift in (0..).step_by(7) {
        let (&byte, rest) = encoded.split_first().ok_or("Invalid certificate time")?;
        encoded = rest;
        if shift > 63 {
            return Err("Invalid certificate time".into());
        }
        time |= u128::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(time);
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize as SerdeDeserialize;

    /// Real responses of the dev Identity Manager: fetched by agent-js 2 through API v2
    /// (scripts/fetch-certified-root-fixture.mjs) and by @icp-sdk/core 5 through API v3, whose subnet
    /// delegation lists canister ranges under `/canister_ranges/<subnet>` instead of `/subnet/<subnet>`.
    const FIXTURES: [(&str, &str); 2] = [
        (
            "api_v2",
            include_str!("../tests/fixtures/im_dev_certified_root_api_v2.json"),
        ),
        (
            "api_v3",
            include_str!("../tests/fixtures/im_dev_certified_root_api_v3.json"),
        ),
    ];

    #[derive(SerdeDeserialize)]
    struct Fixture {
        im_canister: String,
        caller: String,
        root: String,
        certificate: String,
        witness: String,
    }

    struct Case {
        im_canister: Principal,
        caller: Principal,
        certified: CertifiedRoot,
        certificate_time_ns: u128,
        root_key: Vec<u8>,
    }

    fn cases() -> impl Iterator<Item = (&'static str, Case)> {
        FIXTURES.iter().map(|(name, json)| (*name, case(json)))
    }

    fn case(json: &str) -> Case {
        let fixture: Fixture = serde_json::from_str(json).unwrap();
        let certificate = hex::decode(&fixture.certificate).unwrap();
        let parsed = Certificate::from_cbor(&certificate).unwrap();
        Case {
            im_canister: Principal::from_text(&fixture.im_canister).unwrap(),
            caller: Principal::from_text(&fixture.caller).unwrap(),
            certified: CertifiedRoot {
                root: fixture.root,
                certificate: ByteBuf::from(certificate),
                witness: ByteBuf::from(hex::decode(&fixture.witness).unwrap()),
            },
            certificate_time_ns: certificate_time(&parsed).unwrap(),
            root_key: hex::decode(MAINNET_ROOT_KEY_DER).unwrap(),
        }
    }

    fn verify(c: &Case, now_ns: u128) -> Result<(), String> {
        verify_certified_root(&c.im_canister, &c.root_key, &c.caller, &c.certified, now_ns)
    }

    #[test]
    fn fixtures_cover_both_canister_range_formats() {
        let formats: Vec<(&str, bool)> = FIXTURES
            .iter()
            .map(|(name, json)| {
                let fixture: Fixture = serde_json::from_str(json).unwrap();
                let certificate =
                    Certificate::from_cbor(&hex::decode(fixture.certificate).unwrap()).unwrap();
                let delegation = certificate.delegation.expect("subnet delegation");
                let delegation = Certificate::from_cbor(&delegation.certificate).unwrap();
                let new_path: [&[u8]; 1] = [b"canister_ranges"];
                let has_new_format = matches!(
                    delegation.tree.lookup_subtree(&new_path),
                    SubtreeLookupResult::Found(_)
                );
                (*name, has_new_format)
            })
            .collect();
        assert_eq!(formats, vec![("api_v2", false), ("api_v3", true)]);
    }

    #[test]
    fn accepts_real_identity_manager_response() {
        for (name, c) in cases() {
            assert_eq!(verify(&c, c.certificate_time_ns), Ok(()), "{name}");
            assert_eq!(
                verify(&c, c.certificate_time_ns + MAX_CERTIFICATE_TIME_OFFSET_NS),
                Ok(()),
                "{name}"
            );
        }
    }

    #[test]
    fn rejects_stale_or_future_certificate() {
        for (name, c) in cases() {
            let late = c.certificate_time_ns + MAX_CERTIFICATE_TIME_OFFSET_NS + 1_000_000_000;
            let error = verify(&c, late).unwrap_err();
            assert!(error.contains("too far in the past"), "{name}: {error}");
            let early = c.certificate_time_ns - MAX_CERTIFICATE_TIME_OFFSET_NS - 1_000_000_000;
            let error = verify(&c, early).unwrap_err();
            assert!(error.contains("too far in the future"), "{name}: {error}");
        }
    }

    #[test]
    fn rejects_other_caller() {
        for (name, mut c) in cases() {
            c.caller = Principal::from_text("2vxsx-fae").unwrap();
            assert_eq!(
                verify(&c, c.certificate_time_ns),
                Err("Caller is not certified by the Identity Manager".into()),
                "{name}"
            );
        }
    }

    #[test]
    fn rejects_forged_root() {
        for (name, mut c) in cases() {
            c.certified.root = "aaaaa-aa".into();
            assert_eq!(
                verify(&c, c.certificate_time_ns),
                Err("Certified root does not match".into()),
                "{name}"
            );
        }
    }

    #[test]
    fn rejects_tampered_witness() {
        for (name, mut c) in cases() {
            let last = c.certified.witness.len() - 1;
            c.certified.witness[last] ^= 1;
            assert_eq!(
                verify(&c, c.certificate_time_ns),
                Err("Witness does not match the certified data".into()),
                "{name}"
            );
        }
    }

    #[test]
    fn rejects_tampered_certificate_and_wrong_root_key() {
        for (name, mut c) in cases() {
            c.root_key[50] ^= 1;
            let error = verify(&c, c.certificate_time_ns).unwrap_err();
            assert!(
                error.starts_with("Certificate verification failed"),
                "{name}: {error}"
            );
        }
        for (name, mut c) in cases() {
            let last = c.certified.certificate.len() - 1;
            c.certified.certificate[last] ^= 1;
            assert!(verify(&c, c.certificate_time_ns).is_err(), "{name}");
        }
    }

    #[test]
    fn rejects_certificate_for_another_canister() {
        for (name, mut c) in cases() {
            // The legacy IC signer lives on another subnet, outside the delegation's canister ranges.
            c.im_canister = Principal::from_text("nux62-yqaaa-aaaak-ae2pq-cai").unwrap();
            assert!(verify(&c, c.certificate_time_ns).is_err(), "{name}");
        }
    }
}
