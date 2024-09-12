use std::time::Duration;

use crate::cmd::InstallArgs;
use crate::sealed::k8s::namespace::SINamespace;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{ApiResource, DynamicObject, GroupVersionKind, Patch, PatchParams},
    discovery::{ApiCapabilities, Scope},
    runtime::wait::{await_condition, Condition},
    Api, Client, Discovery, ResourceExt,
};
use tracing::{info, trace, warn};

use crate::{
    error::{SealedError, SealedResult},
    settings::Settings,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CNPG_YAML: &str = include_str!("../../config/operators/cnpg-1.22.1.yaml");
const NGINX_YAML: &str = include_str!("../../config/operators/nginx-ingress.yaml");

pub async fn install(args: InstallArgs, config: &Settings) -> SealedResult<()> {
    info!("Installing sealed infrastructure");
    let client = connect_to_cluster(config).await?;
    let ns = SINamespace::new(&args.namespace);
    let operator_ns = SINamespace::new(&args.operator_namespace);
    Ok(())
}

async fn connect_to_cluster(config: &Settings) -> SealedResult<Client> {
    info!("Connecting to cluster...");
    let client = Client::try_default().await?;
    info!("Connected to cluster");
    Ok(client)
}

async fn install_postgres_operator(client: &Client) -> SealedResult<()> {
    info!("Installing cloud native postgres operator (TODO)");
    apply(client, CNPG_YAML, None).await?;
    info!("Waiting for cloud native postgres operator to be available...");
    let deploys: Api<Deployment> = Api::namespaced(client.clone(), "postgres-operator");
    let establish = await_condition(deploys, "postgres-operator", is_deployment_available());
    let _ = tokio::time::timeout(Duration::from_secs(120), establish).await?;
    Ok(())
}

async fn install_nginx_operator(client: &Client) -> SealedResult<()> {
    info!("Installing nginx operator (TODO)");
    apply(client, NGINX_YAML, None).await?;

    info!("Waiting for nginx operator to be available...");
    let deploys: Api<Deployment> = Api::namespaced(client.clone(), "ingress-nginx");
    let establish = await_condition(
        deploys,
        "nginx-ingress-controller",
        is_deployment_available(),
    );
    let _ = tokio::time::timeout(Duration::from_secs(120), establish).await?;

    Ok(())
}

fn is_deployment_available() -> impl Condition<Deployment> {
    |obj: Option<&Deployment>| {
        if let Some(deployment) = &obj {
            if let Some(status) = &deployment.status {
                if let Some(phase) = &status.available_replicas {
                    return phase > &1;
                }
            }
        }
        false
    }
}

async fn apply(client: &Client, yaml: &str, namespace: Option<&str>) -> SealedResult<()> {
    let ssapply = PatchParams::apply("kubectl-light").force();
    let discovery = Discovery::new(client.clone()).run().await?;
    for doc in multidoc_deserialize(yaml)? {
        let obj: DynamicObject = serde_yaml::from_value(doc)?;
        let namespace = obj.metadata.namespace.as_deref().or(namespace);
        let gvk = if let Some(tm) = &obj.types {
            GroupVersionKind::try_from(tm)?
        } else {
            return Err(SealedError::Runtime(anyhow::anyhow!(
                "cannot apply object without valid TypeMeta {:?}",
                obj
            )));
        };
        let name = obj.name_any();
        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
            let api = dynamic_api(ar, caps, client.clone(), namespace, false);
            trace!("Applying {}: \n{}", gvk.kind, serde_yaml::to_string(&obj)?);
            let data: serde_json::Value = serde_json::to_value(&obj)?;
            let _r = api.patch(&name, &ssapply, &Patch::Apply(data)).await?;
            info!("applied {} {}", gvk.kind, name);
        } else {
            warn!("Cannot apply document for unknown {:?}", gvk);
        }
    }

    Ok(())
}

pub fn multidoc_deserialize(data: &str) -> SealedResult<Vec<serde_yaml::Value>> {
    use serde::Deserialize;
    let mut docs = vec![];
    for de in serde_yaml::Deserializer::from_str(data) {
        docs.push(serde_yaml::Value::deserialize(de)?);
    }
    Ok(docs)
}

fn dynamic_api(
    ar: ApiResource,
    caps: ApiCapabilities,
    client: Client,
    ns: Option<&str>,
    all: bool,
) -> Api<DynamicObject> {
    if caps.scope == Scope::Cluster || all {
        Api::all_with(client, &ar)
    } else if let Some(namespace) = ns {
        Api::namespaced_with(client, namespace, &ar)
    } else {
        Api::default_namespaced_with(client, &ar)
    }
}
