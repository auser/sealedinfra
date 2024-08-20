use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deployment {
    pub metadata: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub replicas: u32,
    pub image: String,
    pub env: BTreeMap<String, String>,
}

impl Default for Deployment {
    fn default() -> Self {
        Self {
            metadata: BTreeMap::new(),
            labels: BTreeMap::new(),
            replicas: 1,
            image: String::new(),
            env: BTreeMap::new(),
        }
    }
}

impl Deployment {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn metadata(mut self, metadata: BTreeMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }
}
