use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ServicePort;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SIService {
    pub metadata: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub replicas: u32,
    pub ports: BTreeMap<usize, ServicePort>,
    pub env: BTreeMap<String, String>,
}

impl SIService {
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
impl Default for SIService {
    fn default() -> Self {
        Self {
            metadata: BTreeMap::new(),
            labels: BTreeMap::new(),
            replicas: 1,
            ports: BTreeMap::new(),
            env: BTreeMap::new(),
        }
    }
}

// let ports: Vec<ServicePort> = self
// .ports
// .as_ref()
// .unwrap_or(&vec![])
// .iter()
// .map(|port| k8s_openapi::api::core::v1::ServicePort {
//     port: *port,
//     target_port: Some(
//         k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(*port),
//     ),
//     ..Default::default()
// })
// .collect();

// let service = Service {
// metadata: self.generate_metadata(),
// spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
//     selector: Some(self.generate_labels()),
//     ports: Some(ports),
//     ..Default::default()
// }),
// ..Default::default()
// };
// Ok(service)
