use kube::{
    api::{Patch, PatchParams},
    Api, Client, Error,
};
use serde_json::{json, Value};

use crate::error::SealedOperatorResult;

use super::crd::FpApp;

pub async fn add(
    client: Client,
    name: &str,
    namespace: &str,
) -> SealedOperatorResult<FpApp, Error> {
    let api: Api<FpApp> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
      "metadata": {
        "finalizers": ["apps.fp.com/finalizer"]
      }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}

pub async fn delete(
    client: Client,
    name: &str,
    namespace: &str,
) -> SealedOperatorResult<FpApp, Error> {
    let api: Api<FpApp> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
      "metadata": {
        "finalizers": null
      }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}
