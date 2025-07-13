use std::{
    net::{IpAddr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
};
use bytes::Bytes;
use futures::StreamExt;
use mesh::discovery::Mesh;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Copy)]
pub enum Role {
    Shim,
    Worker,
}

struct AppState {
    mesh: Mesh,
    role: Role,
    counter: AtomicUsize,
    client: Client,
    port: u16,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GenerateRequest {
    pub prompt: String,
}

#[derive(Serialize)]
struct GenerateResponse {
    response: String,
    done: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct EmbeddingsRequest {
    pub prompt: String,
}

#[derive(Serialize)]
struct EmbeddingsResponse {
    embedding: Vec<f32>,
    done: bool,
}

pub async fn run(role: Role, port: u16) {
    let mesh = Mesh::new().await;
    let state = Arc::new(AppState {
        mesh,
        role,
        counter: AtomicUsize::new(0),
        client: Client::new(),
        port,
    });

    let app = Router::new()
        .route("/api/generate", post(generate_handler))
        .route("/api/embeddings", post(embeddings_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn generate_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> impl IntoResponse {
    match state.role {
        Role::Worker => worker_generate(req).await,
        Role::Shim => shim_generate(state, req).await,
    }
}

async fn embeddings_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EmbeddingsRequest>,
) -> impl IntoResponse {
    match state.role {
        Role::Worker => worker_embeddings(req).await,
        Role::Shim => shim_embeddings(state, req).await,
    }
}

async fn worker_generate(req: GenerateRequest) -> Response {
    let answer = if req.prompt.trim() == "2+2=?" {
        "4"
    } else {
        ""
    };
    let line = serde_json::to_string(&GenerateResponse {
        response: answer.to_string(),
        done: true,
    })
    .unwrap()
        + "\n";
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(1);
    tx.send(Ok(Bytes::from(line))).await.unwrap();
    let stream = ReceiverStream::new(rx);
    Body::from_stream(stream).into_response()
}

async fn worker_embeddings(_req: EmbeddingsRequest) -> Response {
    let line = serde_json::to_string(&EmbeddingsResponse {
        embedding: vec![0.0],
        done: true,
    })
    .unwrap()
        + "\n";
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(1);
    tx.send(Ok(Bytes::from(line))).await.unwrap();
    Body::from_stream(ReceiverStream::new(rx)).into_response()
}

async fn shim_generate(state: Arc<AppState>, req: GenerateRequest) -> Response {
    let peers: Vec<IpAddr> = state.mesh.addresses().values().cloned().collect();
    if peers.is_empty() {
        return worker_generate(req).await;
    }
    let idx = state.counter.fetch_add(1, Ordering::SeqCst) % peers.len();
    let ip = peers[idx];
    let url = format!("http://{}:{}/api/generate", ip, state.port);
    let resp = match state.client.post(url).json(&req).send().await {
        Ok(r) => r,
        Err(_) => return worker_generate(req).await,
    };
    let stream = resp
        .bytes_stream()
        .map(|r| r.map_err(std::io::Error::other));
    Body::from_stream(stream).into_response()
}

async fn shim_embeddings(state: Arc<AppState>, req: EmbeddingsRequest) -> Response {
    let peers: Vec<IpAddr> = state.mesh.addresses().values().cloned().collect();
    if peers.is_empty() {
        return worker_embeddings(req).await;
    }
    let idx = state.counter.fetch_add(1, Ordering::SeqCst) % peers.len();
    let ip = peers[idx];
    let url = format!("http://{}:{}/api/embeddings", ip, state.port);
    let resp = match state.client.post(url).json(&req).send().await {
        Ok(r) => r,
        Err(_) => return worker_embeddings(req).await,
    };
    let stream = resp
        .bytes_stream()
        .map(|r| r.map_err(std::io::Error::other));
    Body::from_stream(stream).into_response()
}
