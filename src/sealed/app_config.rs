use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Service;
use k8s_openapi::api::{apps::v1::Deployment, core::v1::ServicePort};
use serde::{Deserialize, Serialize};

use crate::error::SealedResult;

use super::helpers::image_or_from_language;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub image: Option<String>,
    pub language: Option<String>,
    pub dependencies: Vec<String>,
    pub environment: Option<Vec<String>>,
    pub env_file: Option<String>,
    pub replicas: Option<i32>,
    pub labels: Option<BTreeMap<String, String>>,
    pub ports: Option<Vec<i32>>,
}

impl AppConfig {
    /// The function `into_deployment` converts an `AppConfig` into a `Deployment` object for
    /// Kubernetes.
    ///
    /// Arguments:
    ///
    /// * `app`: The `into_deployment` function takes an `AppConfig` struct as a parameter. The
    /// `AppConfig` struct likely contains configuration details for an application to be deployed, such
    /// as its name, environment variables, image, and replicas.
    ///
    /// Returns:
    ///
    /// The function `into_deployment` returns a `SealedResult<Deployment>`, where `Deployment` is a
    /// Kubernetes deployment object.
    pub fn into_deployment(&self) -> SealedResult<Deployment> {
        let mut env = vec![];
        if let Some(env_vars) = &self.environment {
            for env_var in env_vars {
                let parts: Vec<&str> = env_var.splitn(2, '=').collect();
                if parts.len() == 2 {
                    env.push(k8s_openapi::api::core::v1::EnvVar {
                        name: parts[0].to_string(),
                        value: Some(parts[1].to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        let image = image_or_from_language(self.image.clone(), &self.name);
        let metadata = self.generate_metadata();

        let replicas = self.replicas.unwrap_or(1);
        let selector = k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
            match_labels: Some(self.generate_labels()),
            ..Default::default()
        };

        let deployment = Deployment {
            metadata,
            spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
                replicas: Some(replicas),
                selector,
                template: k8s_openapi::api::core::v1::PodTemplateSpec {
                    metadata: Some(kube::api::ObjectMeta {
                        labels: Some(self.generate_labels()),
                        ..Default::default()
                    }),
                    spec: Some(k8s_openapi::api::core::v1::PodSpec {
                        containers: vec![k8s_openapi::api::core::v1::Container {
                            name: self.name.clone(),
                            image: Some(image),
                            env: Some(env),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        Ok(deployment)
    }

    fn generate_labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::from_iter(vec![("app".to_string(), self.name.clone())]);
        if let Some(defined_labels) = &self.labels {
            labels.extend(defined_labels.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        labels
    }

    fn generate_metadata(&self) -> k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
        k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
            name: Some(self.name.clone()),
            ..Default::default()
        }
    }

    pub fn into_service(&self) -> SealedResult<Service> {
        let ports: Vec<ServicePort> = self
            .ports
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|port| k8s_openapi::api::core::v1::ServicePort {
                port: *port,
                target_port: Some(
                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(*port),
                ),
                ..Default::default()
            })
            .collect();

        let service = Service {
            metadata: self.generate_metadata(),
            spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
                selector: Some(self.generate_labels()),
                ports: Some(ports),
                ..Default::default()
            }),
            ..Default::default()
        };
        Ok(service)
    }
}
