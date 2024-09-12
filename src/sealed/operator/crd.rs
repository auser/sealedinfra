use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "fp.com",
    version = "v1",
    kind = "FpApp",
    plural = "fpapps",
    derive = "PartialEq",
    namespaced
)]
pub struct FpAppSpec {
    pub replicas: i32,
    pub version: String,
    pub pgadmin: Option<bool>,
    pub development: Option<bool>,
    pub testing: Option<bool>,
}
