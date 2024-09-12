use anyhow::{Context, Result};
use async_recursion::async_recursion;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Service};
use kube::Client;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use tokio;

use crate::error::SealedResult;

use super::app_config::AppConfig;
use super::operator::crd::FpApp;

struct K8sController {
    client: Client,
    app: AppConfig,
    fp_app: FpApp,
}

impl K8sController {
    async fn new(client: Client, app: AppConfig, fp_app: FpApp) -> Result<Self> {
        Ok(Self {
            client,
            app,
            fp_app,
        })
    }

    async fn deploy_apps(&self) -> Result<()> {
        for (_, app) in &self.config.apps {
            self.deploy_app(app).await?;
        }
        Ok(())
    }

    #[async_recursion]
    async fn deploy_app(&self, app: &AppConfig) -> SealedResult<()> {
        // Deploy dependencies first
        for dep in &app.dependencies {
            if let Some(dep_app) = self.config.apps.get(dep) {
                Box::pin(self.deploy_app(dep_app)).await?;
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
            data: Some(BTreeMap::from_iter(vec![("env".to_string(), cm_data)])),
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

    fn generate_deployment(&self, app: &AppConfig) -> SealedResult<Deployment> {
        app.into_deployment()
    }

    async fn create_service(&self, app: &AppConfig) -> Result<()> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), "default");
        let service = self.generate_service(app)?;
        services.create(&PostParams::default(), &service).await?;
        Ok(())
    }

    fn generate_service(&self, app: &AppConfig) -> SealedResult<Service> {
        app.into_service()
    }
}

fn image_or_from_language(image: Option<String>, language: &str) -> String {
    match image {
        Some(image) => image,
        None => match language {
            "python" => "python:3.12".to_string(),
            "node" => "node:20".to_string(),
            "rust" => "rust".to_string(),
            _ => "alpine:latest".to_string(),
        },
    }
}
