use std::fmt;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use bon::Builder;
use bstr::{BStr, BString, ByteSlice};

/// Represents an optional attribute
#[derive(Debug, Clone, Builder, Default)]
#[builder(on(BString, into))]
pub struct Attribute {
    pub tag: BString,
    #[builder(default = 'Z')]
    pub attribute_type: char,
    pub value: BString,
}

impl FromStr for Attribute {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Format: tag:type:value
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid attribute format: {}", s));
        }

        let tag = parts[0].into();
        let attr_type = parts[1]
            .chars()
            .next()
            .ok_or_else(|| anyhow!("Empty attribute type"))?;
        let value = parts[2].into();

        // Validate attribute type
        match attr_type {
            'i' | 'f' | 'Z' | 'J' | 'H' | 'B' => (),
            _ => return Err(anyhow!("Unknown attribute type: {}", attr_type)),
        }

        Ok(Attribute {
            tag,
            attribute_type: attr_type,
            value,
        })
    }
}

impl Attribute {
    /// Get the integer value if the attribute type is 'i'
    pub fn as_int(&self) -> Result<i32> {
        if self.attribute_type != 'i' {
            return Err(anyhow!("Attribute is not an integer type"));
        }
        self.value
            .to_str()?
            .parse()
            .map_err(|e| anyhow!("Failed to parse integer: {}", e))
    }

    /// Get the float value if the attribute type is 'f'
    pub fn as_float(&self) -> Result<f32> {
        if self.attribute_type != 'f' {
            return Err(anyhow!("Attribute is not a float type"));
        }
        self.value
            .to_str()?
            .parse()
            .map_err(|e| anyhow!("Failed to parse float: {}", e))
    }

    /// Get the string value if the attribute type is 'Z'
    pub fn as_string(&self) -> Result<&BStr> {
        if self.attribute_type != 'Z' {
            return Err(anyhow!("Attribute is not a string type"));
        }
        Ok(self.value.as_bstr())
    }

    /// Get the JSON value if the attribute type is 'J'
    pub fn as_json(&self) -> Result<serde_json::Value> {
        if self.attribute_type != 'J' {
            return Err(anyhow!("Attribute is not a JSON type"));
        }
        serde_json::from_str(self.value.to_str()?)
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e))
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.tag, self.attribute_type, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_attribute_from_str_valid() {
        let attr = Attribute::from_str("ptc:i:1").unwrap();
        assert_eq!(attr.tag, "ptc");
        assert_eq!(attr.attribute_type, 'i');
        assert_eq!(attr.value, "1");

        let attr = Attribute::from_str("ptf:f:0.5").unwrap();
        assert_eq!(attr.tag, "ptf");
        assert_eq!(attr.attribute_type, 'f');
        assert_eq!(attr.value, "0.5");

        let attr = Attribute::from_str("name:Z:test_value").unwrap();
        assert_eq!(attr.tag, "name");
        assert_eq!(attr.attribute_type, 'Z');
        assert_eq!(attr.value, "test_value");

        let attr = Attribute::from_str("data:J:{\"key\":\"value\"}").unwrap();
        assert_eq!(attr.tag, "data");
        assert_eq!(attr.attribute_type, 'J');
        assert_eq!(attr.value, "{\"key\":\"value\"}");
    }

    #[test]
    fn test_attribute_from_str_invalid_format() {
        let result = Attribute::from_str("ptc:i");
        assert!(result.is_err());

        let result = Attribute::from_str("ptc");
        assert!(result.is_err());
    }

    #[test]
    fn test_attribute_from_str_invalid_type() {
        let result = Attribute::from_str("ptc:x:1");
        assert!(result.is_err());
    }

    #[test]
    fn test_attribute_as_int() {
        let attr = Attribute {
            tag: "ptc".into(),
            attribute_type: 'i',
            value: "42".into(),
        };
        assert_eq!(attr.as_int().unwrap(), 42);

        let attr = Attribute {
            tag: "ptc".into(),
            attribute_type: 'f',
            value: "42".into(),
        };
        assert!(attr.as_int().is_err());

        let attr = Attribute {
            tag: "ptc".into(),
            attribute_type: 'i',
            value: "not_a_number".into(),
        };
        assert!(attr.as_int().is_err());
    }

    #[test]
    fn test_attribute_as_float() {
        let attr = Attribute {
            tag: "ptf".into(),
            attribute_type: 'f',
            value: "3.14".into(),
        };
        assert_eq!(attr.as_float().unwrap(), 3.14);

        let attr = Attribute {
            tag: "ptf".into(),
            attribute_type: 'i',
            value: "3.14".into(),
        };
        assert!(attr.as_float().is_err());

        let attr = Attribute {
            tag: "ptf".into(),
            attribute_type: 'f',
            value: "not_a_number".into(),
        };
        assert!(attr.as_float().is_err());
    }

    #[test]
    fn test_attribute_as_string() {
        let attr = Attribute {
            tag: "name".into(),
            attribute_type: 'Z',
            value: "test_value".into(),
        };
        assert_eq!(attr.as_string().unwrap(), "test_value");

        let attr = Attribute {
            tag: "name".into(),
            attribute_type: 'i',
            value: "test_value".into(),
        };
        assert!(attr.as_string().is_err());
    }

    #[test]
    fn test_attribute_as_json() {
        let attr = Attribute {
            tag: "data".into(),
            attribute_type: 'J',
            value: "{\"key\":\"value\"}".into(),
        };
        let json = attr.as_json().unwrap();
        assert_eq!(json["key"], "value");

        let attr = Attribute {
            tag: "data".into(),
            attribute_type: 'Z',
            value: "{\"key\":\"value\"}".into(),
        };
        assert!(attr.as_json().is_err());

        let attr = Attribute {
            tag: "data".into(),
            attribute_type: 'J',
            value: "invalid_json".into(),
        };
        assert!(attr.as_json().is_err());
    }

    #[test]
    fn test_attribute_display() {
        let attr = Attribute {
            tag: "ptc".into(),
            attribute_type: 'i',
            value: "1".into(),
        };
        assert_eq!(attr.to_string(), "ptc:i:1");

        let attr = Attribute {
            tag: "ptf".into(),
            attribute_type: 'f',
            value: "0.5".into(),
        };
        assert_eq!(attr.to_string(), "ptf:f:0.5");
    }
}
