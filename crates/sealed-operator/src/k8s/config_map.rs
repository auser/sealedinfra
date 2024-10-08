use std::collections::BTreeMap;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SIConfigMap {
    pub name: String,
    pub metadata: BTreeMap<String, String>,
    pub owner_references: Option<Vec<OwnerReference>>,
    pub data: Option<BTreeMap<String, String>>,
}

impl SIConfigMap {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            metadata: BTreeMap::new(),
            owner_references: None,
            data: None,
        }
    }
}

impl Default for SIConfigMap {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            metadata: BTreeMap::new(),
            owner_references: None,
            data: None,
        }
    }
}
