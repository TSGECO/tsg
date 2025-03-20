use anyhow::Result;
use anyhow::anyhow;
use regex::Regex;
use sha2::{Digest, Sha256};

/// Convert a string to a hash-based identifier using SHA-256.
///
/// # Arguments
///
/// * `input_string` - The string to convert
/// * `length` - The desired length of the output identifier (default: Some(16))
///             If None, returns the full hash
///
/// # Returns
///
/// A valid identifier string derived from the SHA-256 hash
///
/// # Examples
///
/// ```
/// use tsg_core::graph::to_hash_identifier;
///
/// let hash = to_hash_identifier("Hello World!", Some(16)).unwrap();
/// assert_eq!(hash, "af83b1657ff1fc53");
/// ```
pub fn to_hash_identifier(input_string: &str, length: Option<usize>) -> Result<String> {
    // Validate input
    let length = match length {
        Some(len) => {
            if len == 0 {
                return Err(anyhow!("Length must be positive"));
            }
            len
        }
        None => 64, // Full SHA-256 hash length
    };

    // Create SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(input_string.as_bytes());
    let hash_bytes = hasher.finalize();

    // Convert to hex string
    let hash_hex = format!("{:x}", hash_bytes);

    // Take specified length of hash
    let mut result = if length < hash_hex.len() {
        hash_hex[..length].to_string()
    } else {
        hash_hex
    };

    // Ensure the identifier starts with a letter (prefix with 'a' if it starts with a number)
    let re = Regex::new(r"^[0-9]").unwrap();
    if re.is_match(&result) {
        // Replace first char with 'a'
        result.replace_range(0..1, "a");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_hash() {
        let result = to_hash_identifier("Hello World!", Some(16)).unwrap();
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_starts_with_letter() {
        let result = to_hash_identifier("Test123", Some(8)).unwrap();
        assert!(result.chars().next().unwrap().is_alphabetic());
    }

    #[test]
    fn test_full_hash() {
        let result = to_hash_identifier("Full hash test", None).unwrap();
        assert_eq!(result.len(), 64); // Full SHA-256 hash length
    }

    #[test]
    fn test_invalid_length() {
        let result = to_hash_identifier("Invalid", Some(0));
        assert!(result.is_err());
    }
}
