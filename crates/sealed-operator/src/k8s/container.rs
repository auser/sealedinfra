use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SIContainer {
    pub name: String,
    pub image: String,
    pub env: Option<BTreeMap<String, String>>,
    pub command: Option<Vec<String>>,
}

impl SIContainer {
    pub fn new(name: &str, image: &str) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
            env: None,
            command: None,
        }
    }
}

// impl Default for SIContainer {
//     fn default() -> Self {
//         Self {
//             name: String::new(),
//             image: String::new(),
//             env: None,
//             command: None,
//         }
//     }
// }
