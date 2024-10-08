use std::sync::Arc;

use crate::controller::SIController;
use crate::error::SealedOperatorError;
use crate::error::SealedOperatorResult;

use super::crd::FpApp;
use super::finalizer;
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

pub async fn reconcile(
    fp_app: Arc<FpApp>,
    context: Arc<ContextData>,
) -> SealedOperatorResult<Action> {
    let client: Client = context.client.clone();
    let namespace = fp_app.namespace().unwrap_or("default".to_string());
    let name = fp_app.name_any();

    let arc_client = Arc::new(client.clone());

    let si_controller = SIController::new(arc_client.clone(), fp_app.clone()).await?;

    match determine_action(&fp_app) {
        SealedAction::Create => {
            finalizer::add(client.clone(), &name, &namespace).await?;

            si_controller.deploy_app().await?;
            Ok(Action::await_change())
        }
        SealedAction::Delete => {
            finalizer::delete(client.clone(), &name, &namespace).await?;

            si_controller.delete_app().await?;
            Ok(Action::await_change())
        }
        SealedAction::NoOp => {
            println!("Nothing to do");
            Ok(Action::requeue(Duration::from_secs(10)))
        }
    }
}

fn determine_action(fp_app: &FpApp) -> SealedAction {
    if fp_app.meta().deletion_timestamp.is_some() {
        SealedAction::Delete
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

pub fn on_error(
    fp_app: Arc<FpApp>,
    error: &SealedOperatorError,
    _context: Arc<ContextData>,
) -> Action {
    eprintln!("Reconciliation error:\n{:?}.\n{:?}", error, fp_app);
    Action::requeue(Duration::from_secs(5))
}
