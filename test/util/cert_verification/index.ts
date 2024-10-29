import {
  Cbor,
  Certificate,
  HashTree,
  reconstruct,
  compare,
  HttpAgent,
  lookup_path,
} from '@dfinity/agent';
import { Principal } from '@dfinity/principal';
import { PipeArrayBuffer, lebDecode } from '@dfinity/candid';
import { CertificateTimeError, CertificateVerificationError } from './error';
import * as crypto from "crypto";
import {LookupResultFound} from "@dfinity/agent/lib/esm/certificate";

export interface VerifyCertificationParams {
  canisterId: Principal;
  encodedCertificate: ArrayBuffer;
  encodedTree: ArrayBuffer;
  rootKey: ArrayBuffer;
  maxCertificateTimeOffsetMs: number;
}

export async function verifyCertifiedResponse(certificate: Uint8Array | number[], witness: Uint8Array | number[], principal: string, canisterId: string, newOwnedString: string) {
  const agent = new HttpAgent({host: "http://127.0.0.1:8000"});
  await agent.fetchRootKey();
  const tree = await verifyCertification({
      canisterId: Principal.fromText(canisterId),
      encodedCertificate: new Uint8Array(certificate).buffer,
      encodedTree: new Uint8Array(witness).buffer,
      rootKey: agent.rootKey,
      maxCertificateTimeOffsetMs: 50000,
  });

  const treeHash = lookup_path([principal], tree) as LookupResultFound;
  if (!treeHash) {
      throw new Error('Response not found in tree');
  }
  const sha256Result = crypto.createHash('sha256').update(newOwnedString).digest();
  const byteArray = new Uint8Array(sha256Result);
  if (!equal(byteArray, treeHash.value as ArrayBuffer)) {
      throw new Error('Response hash does not match');
  }
}

export async function verifyCertification({
  canisterId,
  encodedCertificate,
  encodedTree,
  rootKey,
  maxCertificateTimeOffsetMs,
}: VerifyCertificationParams): Promise<HashTree> {
  const nowMs = Date.now();
  const certificate = await Certificate.create({
    certificate: encodedCertificate,
    canisterId,
    rootKey,
  });
  const tree = Cbor.decode<HashTree>(encodedTree);

  validateCertificateTime(certificate, maxCertificateTimeOffsetMs, nowMs);
  await validateTree(tree, certificate, canisterId);

  return tree;
}

function validateCertificateTime(
  certificate: Certificate,
  maxCertificateTimeOffsetMs: number,
  nowMs: number,
): void {
    let a = certificate.lookup(['time'])  as LookupResultFound
  const certificateTimeNs = lebDecode(
    new PipeArrayBuffer(a.value as ArrayBuffer),
  );
  const certificateTimeMs = Number(certificateTimeNs / BigInt(1_000_000));

  if (certificateTimeMs - maxCertificateTimeOffsetMs > nowMs) {
    throw new CertificateTimeError(
      `Invalid certificate: time ${certificateTimeMs} is too far in the future (current time: ${nowMs})`,
    );
  }

  if (certificateTimeMs + maxCertificateTimeOffsetMs < nowMs) {
    throw new CertificateTimeError(
      `Invalid certificate: time ${certificateTimeMs} is too far in the past (current time: ${nowMs})`,
    );
  }
}

async function validateTree(
  tree: HashTree,
  certificate: Certificate,
  canisterId: Principal,
): Promise<void> {
  const treeRootHash = await reconstruct(tree);
  const certifiedData = certificate.lookup([
    'canister',
    canisterId.toUint8Array(),
    'certified_data',
  ]) as LookupResultFound;

  if (!certifiedData) {
    throw new CertificateVerificationError(
      'Could not find certified data in the certificate.',
    );
  }

  if (!equal(certifiedData.value as ArrayBuffer, treeRootHash)) {
    throw new CertificateVerificationError(
      'Tree root hash did not match the certified data in the certificate.',
    );
  }
}

function equal(a: ArrayBuffer, b: ArrayBuffer): boolean {
  return compare(a, b) === 0;
}
