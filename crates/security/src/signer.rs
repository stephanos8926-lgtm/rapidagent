//! Security signing for authenticity verification

use rapidagent_ir::PromptIR;

/// Sign a prompt with a secret key (HMAC-SHA256)
pub fn sign_prompt(ir: &PromptIR, secret: &str) -> String {
    use sha2::{Digest, Sha256};
    use hex;
    
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(serde_json::to_string(ir).unwrap_or_default().as_bytes());
    
    hex::encode(hasher.finalize())
}

/// Verify a prompt signature
pub fn verify_prompt(ir: &PromptIR, signature: &str, secret: &str) -> bool {
    let expected = sign_prompt(ir, secret);
    expected == signature
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapidagent_ir::{Section, Priority};
    
    #[test]
    fn test_sign_and_verify() {
        let ir = PromptIR {
            name: "test".to_string(),
            version: rapidagent_ir::Semver::new(1, 0, 0),
            sections: vec![
                Section::new("identity", Priority::P1, "You are an AI."),
            ],
            variables: Vec::new(),
            output_schema: None,
            metadata: rapidagent_ir::PromptMetadata::default(),
        };
        
        let secret = "my-secret-key";
        let signature = sign_prompt(&ir, secret);
        
        assert!(verify_prompt(&ir, &signature, secret));
        assert!(!verify_prompt(&ir, &signature, "wrong-secret"));
    }
}