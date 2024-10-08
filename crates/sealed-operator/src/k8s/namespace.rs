use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SINamespace {
    pub name: String,
}

impl SINamespace {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Default for SINamespace {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let namespace = SINamespace::default();
        assert_eq!(namespace.name, "default");
    }
}
