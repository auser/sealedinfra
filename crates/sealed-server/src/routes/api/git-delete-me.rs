use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use gix::create::Kind;
use serde_json::json;
use tokio::fs;

use crate::error::{SealedError, SealedResult};
use crate::server::app_state::SharedAppState;

pub fn routes(app_state: SharedAppState) -> Router<SharedAppState> {
    Router::new()
        .route("/git/:repo_name/*path", post(handle_git_request))
        .with_state(app_state)
}

#[derive(utoipa::OpenApi)]
#[openapi(info(title = "Git API", version = "0.1.0", description = "API for Git"))]
pub struct GitOpenApi;

async fn handle_git_request(
    State(app_state): State<SharedAppState>,
    Path((repo_name, path)): Path<(String, String)>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, StatusCode> {
    let repo_path = PathBuf::from(format!("./repos/{}", repo_name));

    if !repo_path.exists() {
        fs::create_dir_all(&repo_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        gix::create::into(
            repo_path.clone(),
            Kind::Bare,
            gix::create::Options::default(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    match path.as_str() {
        "git-receive-pack" => handle_push(&repo_path, &body).await,
        "git-upload-pack" => handle_fetch(&repo_path, &body).await,
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn handle_push(repo_path: &PathBuf, body: &[u8]) -> Result<impl IntoResponse, StatusCode> {
    let repo = gix::open(repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // This is a simplified push handler. In a real-world scenario, you'd need to parse the pack file
    // and apply the changes to the repository.
    let mut pack = gix::pack::Bundle::new(body.to_vec().into());
    let mut db = repo.objects.write();
    pack.decode_entries(&mut db, |_, _| {})
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update refs (this is a simplified version, you'd need to parse the actual ref updates from the request)
    let mut refs = repo
        .refs
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(id) = pack.entries().last() {
        refs.set_symbolic_ref(
            gix::refs::FullName::try_from("refs/heads/main").unwrap(),
            id.clone().into(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Push successful" })),
    ))
}

async fn handle_fetch(repo_path: &PathBuf, body: &[u8]) -> Result<impl IntoResponse, StatusCode> {
    let repo = gix::open(repo_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // This is a simplified fetch handler. In a real-world scenario, you'd need to parse the fetch request
    // and send back the appropriate pack file.
    let head = repo
        .head()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .try_into_peeled()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut pack = Vec::new();
    let mut encoder = gix::pack::data::Encoder::new(gix::pack::data::Version::V2, &mut pack);

    encoder
        .write_object(&repo.objects, head.into(), &mut gix::pack::cache::Never)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    encoder
        .finish()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, pack))
}
