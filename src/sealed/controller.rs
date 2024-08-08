use anyhow::{Context, Result};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Service};
use kube::{
    api::{Api, DeleteParams, ListParams, PostParams, WatchEvent},
    Client, Resource,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tokio;

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    name: String,
    image: String,
    dependencies: Vec<String>,
    environment: Option<Vec<String>>,
    env_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    apps: HashMap<String, AppConfig>,
}

struct K8sController {
    client: Client,
    config: Config,
}

impl K8sController {
    async fn new(config_path: &str) -> Result<Self> {
        let config_str = fs::read_to_string(config_path).context("Failed to read config file")?;
        let config: Config =
            serde_yaml::from_str(&config_str).context("Failed to parse config file")?;
        let client = Client::try_default().await?;
        Ok(Self { client, config })
    }

    async fn deploy_apps(&self) -> Result<()> {
        for (_, app) in &self.config.apps {
            self.deploy_app(app).await?;
        }
        Ok(())
    }

    async fn deploy_app(&self, app: &AppConfig) -> Result<()> {
        // Deploy dependencies first
        for dep in &app.dependencies {
            if let Some(dep_app) = self.config.apps.get(dep) {
                self.deploy_app(dep_app).await?;
            }
        }

        println!("Deploying {}", app.name);

        // Create ConfigMap if env_file is specified
        if let Some(env_file) = &app.env_file {
            self.create_config_map(app, env_file).await?;
        }

        // Create Deployment
        self.create_deployment(app).await?;

        // Create Service
        self.create_service(app).await?;

        println!("Successfully deployed {}", app.name);
        Ok(())
    }

    async fn create_config_map(&self, app: &AppConfig, env_file: &str) -> Result<()> {
        let cm_api: Api<ConfigMap> = Api::namespaced(self.client.clone(), "default");
        let cm_data = fs::read_to_string(env_file)?;
        let cm = ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some(format!("{}-config", app.name)),
                ..Default::default()
            },
            data: Some(HashMap::from_iter(vec![("env".to_string(), cm_data)])),
            ..Default::default()
        };
        cm_api.create(&PostParams::default(), &cm).await?;
        Ok(())
    }

    async fn create_deployment(&self, app: &AppConfig) -> Result<()> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), "default");
        let deployment = self.generate_deployment(app)?;
        deployments
            .create(&PostParams::default(), &deployment)
            .await?;
        Ok(())
    }

    fn generate_deployment(&self, app: &AppConfig) -> Result<Deployment> {
        // Generate Deployment resource based on app config
        // This is a simplified version. You'll need to flesh this out with more details.
        let mut env = vec![];
        if let Some(env_vars) = &app.environment {
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

        let deployment = Deployment {
            metadata: kube::api::ObjectMeta {
                name: Some(app.name.clone()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
                replicas: Some(1),
                selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                    match_labels: Some(HashMap::from_iter(vec![(
                        "app".to_string(),
                        app.name.clone(),
                    )])),
                    ..Default::default()
                },
                template: k8s_openapi::api::core::v1::PodTemplateSpec {
                    metadata: Some(kube::api::ObjectMeta {
                        labels: Some(HashMap::from_iter(vec![(
                            "app".to_string(),
                            app.name.clone(),
                        )])),
                        ..Default::default()
                    }),
                    spec: Some(k8s_openapi::api::core::v1::PodSpec {
                        containers: vec![k8s_openapi::api::core::v1::Container {
                            name: app.name.clone(),
                            image: Some(app.image.clone()),
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

    async fn create_service(&self, app: &AppConfig) -> Result<()> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), "default");
        let service = self.generate_service(app)?;
        services.create(&PostParams::default(), &service).await?;
        Ok(())
    }

    fn generate_service(&self, app: &AppConfig) -> Result<Service> {
        // Generate Service resource based on app config
        // This is a simplified version. You'll need to adjust based on your needs.
        let service = Service {
            metadata: kube::api::ObjectMeta {
                name: Some(app.name.clone()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
                selector: Some(HashMap::from_iter(vec![(
                    "app".to_string(),
                    app.name.clone(),
                )])),
                ports: Some(vec![k8s_openapi::api::core::v1::ServicePort {
                    port: 80,
                    target_port: Some(
                        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(80),
                    ),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        Ok(service)
    }

    async fn watch_resources(&self) -> Result<()> {
        let deployments: Api<Deployment> = Api::all(self.client.clone());
        let lp = ListParams::default();

        let mut stream = deployments.watch(&lp, "0").await?.boxed();

        while let Some(status) = stream.next().await {
            match status {
                Ok(WatchEvent::Added(dep)) => println!(
                    "Deployment added: {}",
                    dep.metadata.name.unwrap_or_default()
                ),
                Ok(WatchEvent::Modified(dep)) => println!(
                    "Deployment modified: {}",
                    dep.metadata.name.unwrap_or_default()
                ),
                Ok(WatchEvent::Deleted(dep)) => println!(
                    "Deployment deleted: {}",
                    dep.metadata.name.unwrap_or_default()
                ),
                Ok(WatchEvent::Error(err)) => println!("Error: {}", err),
                Err(e) => println!("Watch error: {}", e),
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let controller = K8sController::new("config.yaml").await?;
    controller.deploy_apps().await?;
    controller.watch_resources().await?;
    Ok(())
}
