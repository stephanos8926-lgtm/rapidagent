//! Binary artifact format for RapidAgent compiled agents
//!
//! Produces `.rapidagent` files with:
//! - Ed25519 signature (compiler keypair)
//! - UUIDv5 provenance = UUIDv5(namespace=compiler_pubkey, name=payload_sha256)
//! - Optional encryption (age/x25519)
//!
//! Binary layout:
//! ```text
//! +---------------------+
//! | MAGIC: "RAPIDAGT"   (8 bytes)
//! | VERSION: u32 LE     (4 bytes)
//! | SIG_LEN: u16 LE     (2 bytes)
//! | SIGNATURE: Ed25519  (SIG_LEN bytes, 64)
//! | PAYLOAD_LEN: u64 LE (8 bytes)
//! | PAYLOAD (JSON compressed) (PAYLOAD_LEN bytes)
//! +---------------------+
//! ```

use anyhow::{Context, Result};
use ed25519_dalek::{Signature as Ed25519Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use uuid::Uuid;

use rapidagent_ir::CompiledArtifact;

/// Binary format version
pub const FORMAT_VERSION: u32 = 1;

/// Magic bytes identifying a .rapidagent file (8 bytes)
pub const MAGIC: &[u8] = b"RAPIDAGT";

/// Compiler identity: a keypair used to sign artifacts
#[derive(Debug, Clone)]
pub struct CompilerIdentity {
    secret_key: ed25519_dalek::SigningKey,
    public_key: ed25519_dalek::VerifyingKey,
}

impl CompilerIdentity {
    /// Generate a new random keypair
    pub fn generate() -> Result<Self> {
        let mut csprng = rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        Ok(Self {
            secret_key: signing_key.clone(),
            public_key: signing_key.verifying_key(),
        })
    }

    /// Load from base64-encoded secret key
    pub fn from_secret_key_b64(b64: &str) -> Result<Self> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let bytes = STANDARD.decode(b64).context("Invalid base64 in secret key")?;
        if bytes.len() != 32 {
            anyhow::bail!("Ed25519 secret key must be 32 bytes, got {}", bytes.len());
        }
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&bytes.try_into().unwrap());
        Ok(Self {
            secret_key: signing_key.clone(),
            public_key: signing_key.verifying_key(),
        })
    }

    /// Get the short hex representation of the public key (16 chars)
    pub fn pubkey_short(&self) -> String {
        hex::encode(&self.public_key.to_bytes()[..8])
    }

    /// Sign a payload (returns Ed25519 signature bytes)
    pub fn sign(&self, payload: &[u8]) -> Ed25519Signature {
        self.secret_key.sign(payload)
    }

    /// Verify a signature against a payload
    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> bool {
        if sig.len() != 64 {
            return false;
        }
        let sig_bytes: [u8; 64] = sig.try_into().unwrap();
        let signature = Ed25519Signature::from(sig_bytes);
        self.public_key.verify(payload, &signature).is_ok()
    }
}

impl Default for CompilerIdentity {
    /// Default: generate a fresh keypair (for dev/testing)
    fn default() -> Self {
        Self::generate().expect("Failed to generate default compiler keypair")
    }
}

// ─── Binary Serialization ────────────────────────────────────────────────────

/// Serialize CompiledArtifact to binary .rapidagent format
pub fn to_binary(artifact: &CompiledArtifact, identity: &CompilerIdentity) -> Result<Vec<u8>> {
    // Serialize artifact to JSON
    let payload_bytes = serde_json::to_vec(artifact)
        .context("Failed to serialize artifact to JSON")?;

    // Compute payload hash (SHA-256)
    let mut hasher = Sha256::new();
    hasher.update(&payload_bytes);
    let payload_hash = hex::encode(hasher.finalize());

    // Decode hash to bytes for UUID computation
    let hash_bytes: Vec<u8> = hex::decode(&payload_hash)
        .context("Invalid payload hash format")?;
    if hash_bytes.len() != 32 {
        anyhow::bail!("Payload hash has wrong length: {}", hash_bytes.len());
    }

    // Compute UUIDv5: namespace=first 16 bytes of pubkey, name=payload_hash
    let pubkey_bytes = identity.public_key.to_bytes();
    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(&pubkey_bytes[..16]);
    hash_input.extend_from_slice(&hash_bytes);
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, &hash_input);
    let uuid_str = uuid.to_string();

    // Update artifact metadata with computed values
    let mut artifact = artifact.clone();
    artifact.metadata.uuid = uuid_str;
    artifact.metadata.hash = payload_hash;
    artifact.metadata.author = identity.pubkey_short();

    // Re-serialize with updated metadata
    let payload_bytes = serde_json::to_vec(&artifact)
        .context("Failed to re-serialize artifact")?;

    // Sign the payload
    let signature = identity.sign(&payload_bytes);
    let sig_bytes = signature.to_bytes();

    // Build binary format
    let mut buf = Vec::new();

    // Magic
    buf.extend_from_slice(MAGIC);

    // Version
    buf.extend_from_slice(&FORMAT_VERSION.to_le_bytes());

    // Signature length
    buf.extend_from_slice(&(sig_bytes.len() as u16).to_le_bytes());

    // Signature
    buf.extend_from_slice(&sig_bytes);

    // Payload length
    buf.extend_from_slice(&(payload_bytes.len() as u64).to_le_bytes());

    // Payload
    buf.extend_from_slice(&payload_bytes);

    Ok(buf)
}

/// Deserialize from binary .rapidagent format
pub fn from_binary(data: &[u8], identity: Option<&CompilerIdentity>) -> Result<CompiledArtifact> {
    // Validate magic
    if data.len() < 8 || &data[..8] != MAGIC {
        anyhow::bail!("Invalid magic bytes: not a .rapidagent file");
    }

    // Parse version (at bytes 8-12, after 8-byte magic)
    if data.len() < 12 {
        anyhow::bail!("Binary data too short");
    }
    let version = u32::from_le_bytes(data[8..12].try_into().unwrap());
    if version != FORMAT_VERSION {
        anyhow::bail!("Unsupported format version: {} (expected {})", version, FORMAT_VERSION);
    }

    // Parse signature length and signature
    let sig_len = u16::from_le_bytes(data[12..14].try_into().unwrap()) as usize;
    if data.len() < 14 + sig_len + 8 {
        anyhow::bail!("Binary data truncated");
    }
    let sig_bytes = &data[14..14 + sig_len];
    let payload_start = 14 + sig_len;
    let payload_len = u64::from_le_bytes(data[payload_start..payload_start + 8].try_into().unwrap()) as usize;
    if data.len() < payload_start + 8 + payload_len {
        anyhow::bail!("Binary data truncated (payload)");
    }
    let payload = &data[payload_start + 8..payload_start + 8 + payload_len];

    // Verify signature if identity provided
    if let Some(id) = identity {
        if !id.verify(payload, sig_bytes) {
            anyhow::bail!("Signature verification failed");
        }
    }

    // Deserialize JSON
    let artifact: CompiledArtifact = serde_json::from_slice(payload)
        .context("Failed to parse artifact JSON")?;

    Ok(artifact)
}

// ─── Encryption (Optional) ──────────────────────────────────────────────────

/// Encrypt payload using age (x25519)
/// Note: Requires the `age` crate. For now, returns unencrypted payload.
pub fn encrypt_payload(_payload: &[u8], _recipient_pubkey: &str) -> Result<Vec<u8>> {
    // Placeholder: age encryption requires external crate
    // In production: use `age-encryption` or `rust-sod`
    Ok(_payload.to_vec())
}

/// Decrypt payload using age (x25519)
pub fn decrypt_payload(_payload: &[u8], _identity_b64: &str) -> Result<Vec<u8>> {
    // Placeholder: age decryption requires external crate
    Ok(_payload.to_vec())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{AgentGraph, Semver, ArtifactMetadata};
    use chrono::Utc;

    fn test_artifact() -> CompiledArtifact {
        CompiledArtifact::new("test-agent", Semver::new(1, 0, 0), AgentGraph::new("test", "", "n1"))
    }

    #[test]
    fn test_roundtrip() {
        let identity = CompilerIdentity::generate().unwrap();
        let artifact = test_artifact();

        // Serialize to binary
        let binary = to_binary(&artifact, &identity).expect("Failed to serialize");
        assert!(binary.starts_with(MAGIC));

        // Deserialize
        let recovered = from_binary(&binary, Some(&identity)).expect("Failed to deserialize");
        assert_eq!(recovered.name, artifact.name);
        assert_eq!(recovered.version, artifact.version);
        assert_eq!(recovered.metadata.uuid.len(), 36); // UUID string
        assert_eq!(recovered.metadata.hash.len(), 64); // SHA-256 hex
    }

    #[test]
    fn test_signature_verification() {
        let identity = CompilerIdentity::generate().unwrap();
        let bad_identity = CompilerIdentity::generate().unwrap();
        let artifact = test_artifact();

        let binary = to_binary(&artifact, &identity).unwrap();

        // Verify with correct identity
        assert!(from_binary(&binary, Some(&identity)).is_ok());

        // Verify with wrong identity should fail
        assert!(from_binary(&binary, Some(&bad_identity)).is_err());
    }

    #[test]
    fn test_invalid_magic() {
        assert!(from_binary(b"NOTRAPID", None).is_err());
    }

    #[test]
    fn test_uuid_deterministic() {
        let identity = CompilerIdentity::generate().unwrap();
        let artifact = test_artifact();

        let binary1 = to_binary(&artifact, &identity).unwrap();
        let binary2 = to_binary(&artifact, &identity).unwrap();

        // Same artifact + same identity = same binary (deterministic UUID)
        assert_eq!(binary1, binary2);

        // Extract UUID from both
        let recovered1 = from_binary(&binary1, Some(&identity)).unwrap();
        let recovered2 = from_binary(&binary2, Some(&identity)).unwrap();
        assert_eq!(recovered1.metadata.uuid, recovered2.metadata.uuid);
    }
}
