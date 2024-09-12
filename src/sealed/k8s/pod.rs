use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ServicePort;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SIPod {
    pub metadata: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub replicas: u32,
    pub ports: BTreeMap<usize, ServicePort>,
    pub env: BTreeMap<String, String>,
}

impl SIPod {
    pub fn new(name: &str) -> Self {
        Self {
            metadata: BTreeMap::new(),
            labels: BTreeMap::new(),
            replicas: 1,
            ports: BTreeMap::new(),
            env: BTreeMap::new(),
        }
    }
}
