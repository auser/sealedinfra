use std::sync::Arc;

use super::crd::FpApp;
use super::finalizer;
use crate::error::{SealedError, SealedResult};
use kube::runtime::controller::Action;
use kube::Client;
use kube::Resource;
use kube::ResourceExt;
use std::time::Duration;

pub struct ContextData {
    client: Client,
}

impl ContextData {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

enum SealedAction {
    Create,
    NoOp,
    Delete,
}

pub async fn reconcile(fp_app: Arc<FpApp>, context: Arc<ContextData>) -> SealedResult<Action> {
    let client: Client = context.client.clone();
    let namespace = fp_app.namespace().unwrap_or("default".to_string());
    let name = fp_app.name_any();

    match determine_action(&fp_app) {
        SealedAction::Create => {
            finalizer::add(client.clone(), &name, &namespace).await?;

            deploy_app(client.clone(), &fp_app, context).await?;
            Ok(Action::await_change())
        }
        SealedAction::Delete => {
            finalizer::delete(client.clone(), &name, &namespace).await?;

            delete_app(client.clone(), &fp_app, context).await?;
            Ok(Action::await_change())
        }
        SealedAction::NoOp => {
            println!("Nothing to do");
            Ok(Action::requeue(Duration::from_secs(10)))
        }
    }
}

async fn deploy_app(client: Client, fp_app: &FpApp, context: Arc<ContextData>) -> SealedResult<()> {
    Ok(())
}

async fn delete_app(client: Client, fp_app: &FpApp, context: Arc<ContextData>) -> SealedResult<()> {
    Ok(())
}

fn determine_action(fp_app: &FpApp) -> SealedAction {
    if fp_app.meta().deletion_timestamp.is_some() {
        return SealedAction::Delete;
    } else if fp_app
        .meta()
        .finalizers
        .as_ref()
        .map_or(true, |finalizers| finalizers.is_empty())
    {
        return SealedAction::Create;
    } else {
        SealedAction::NoOp
    }
}

pub fn on_error(fp_app: Arc<FpApp>, error: &SealedError, _context: Arc<ContextData>) -> Action {
    eprintln!("Reconciliation error:\n{:?}.\n{:?}", error, fp_app);
    Action::requeue(Duration::from_secs(5))
}
