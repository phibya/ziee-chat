use crate::database::models::PasswordService;
use rand::{rng, Rng};

/// Generate a random salt as 16-byte array
pub fn generate_salt() -> [u8; 16] {
    let mut rng = rng();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt);
    salt
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Convert hex string to bytes
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Invalid hex string length".to_string());
    }
    
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "Invalid hex string".to_string())
}

/// Hash a password with a random salt using bcrypt
pub fn hash_password(password: &str) -> Result<PasswordService, bcrypt::BcryptError> {
    let salt_bytes = generate_salt();
    let salt_string = bytes_to_hex(&salt_bytes);
    
    // Create bcrypt hash with custom salt
    let bcrypt_hash = bcrypt::hash_with_salt(password, bcrypt::DEFAULT_COST, salt_bytes)?;
    
    Ok(PasswordService {
        bcrypt: bcrypt_hash.to_string(),
        salt: salt_string,
    })
}

/// Hash a password with a provided salt using bcrypt
pub fn hash_password_with_salt(password: &str, salt: &str) -> Result<PasswordService, bcrypt::BcryptError> {
    // Decode salt from hex
    let salt_bytes = hex_to_bytes(salt).map_err(|e| bcrypt::BcryptError::InvalidHash(e))?;
    if salt_bytes.len() != 16 {
        return Err(bcrypt::BcryptError::InvalidHash("Salt must be 16 bytes".to_string()));
    }
    
    let mut salt_array = [0u8; 16];
    salt_array.copy_from_slice(&salt_bytes);
    
    let bcrypt_hash = bcrypt::hash_with_salt(password, bcrypt::DEFAULT_COST, salt_array)?;
    
    Ok(PasswordService {
        bcrypt: bcrypt_hash.to_string(),
        salt: salt.to_string(),
    })
}

/// Verify a password against a stored password service
pub fn verify_password(password: &str, password_service: &PasswordService) -> Result<bool, bcrypt::BcryptError> {
    // Re-hash the input password with the stored salt and compare with stored bcrypt hash
    let test_service = hash_password_with_salt(password, &password_service.salt)?;
    Ok(test_service.bcrypt == password_service.bcrypt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        
        assert_eq!(salt1.len(), 16);
        assert_eq!(salt2.len(), 16);
        assert_ne!(salt1, salt2); // Should be different
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = [0x41, 0x42, 0x43, 0x44];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "41424344");
        
        let decoded = hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let result = hash_password(password).unwrap();
        
        assert!(!result.bcrypt.is_empty());
        assert!(!result.salt.is_empty());
        assert_eq!(result.salt.len(), 32); // 16 bytes as hex = 32 chars
    }

    #[test]
    fn test_verify_password() {
        let password = "test_password_123";
        let password_service = hash_password(password).unwrap();
        
        // Should verify correctly with correct password
        assert!(verify_password(password, &password_service).unwrap());
        
        // Should fail with incorrect password
        assert!(!verify_password("wrong_password", &password_service).unwrap());
    }

    #[test]
    fn test_hash_with_same_salt() {
        let password = "test_password_123";
        let salt = "0123456789abcdef0123456789abcdef"; // 16 bytes as hex
        
        let service1 = hash_password_with_salt(password, salt).unwrap();
        let service2 = hash_password_with_salt(password, salt).unwrap();
        
        // Same password + same salt should produce same hash
        assert_eq!(service1.bcrypt, service2.bcrypt);
        assert_eq!(service1.salt, service2.salt);
    }
}