pub mod crd;
mod finalizer;
mod reconcile;

use crd::FpApp;
use futures::StreamExt;

use kube::{runtime::Controller, Api, Client};
use reconcile::ContextData;
use sealed_common::util::tracing::setup_tracing;
use std::sync::Arc;

use crate::error::SealedOperatorResult;

pub async fn operator() -> SealedOperatorResult<()> {
    setup_tracing(None).await;

    let kubernetes_client = Client::try_default().await?;

    let crd_api: Api<FpApp> = Api::all(kubernetes_client.clone());
    let context: Arc<ContextData> = Arc::new(ContextData::new(kubernetes_client.clone()));

    Controller::new(crd_api.clone(), Default::default())
        .run(reconcile::reconcile, reconcile::on_error, context)
        .for_each(|recon_result| async move {
            match recon_result {
                Ok((echo_resource, _action)) => {
                    println!("Reconciliation successful. Resource: {:?}", echo_resource)
                }
                Err(err) => {
                    eprintln!("Reconciliation error: {:?}", err);
                }
            }
        })
        .await;

    Ok(())
}
