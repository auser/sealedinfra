use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Container;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SIDeployment {
    pub name: String,
    pub metadata: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub replicas: u32,
    pub image: String,
    pub env: BTreeMap<String, String>,
    pub containers: BTreeMap<String, Container>,
}

impl Default for SIDeployment {
    fn default() -> Self {
        Self {
            name: String::new(),
            metadata: BTreeMap::new(),
            labels: BTreeMap::new(),
            replicas: 1,
            image: String::new(),
            env: BTreeMap::new(),
            containers: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let deployment = SIDeployment::default();
        assert_eq!(deployment.name, String::new());
    }
}
