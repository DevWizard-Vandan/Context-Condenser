#![deny(unsafe_code)]

mod graph;
mod ml;
mod parser;

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(chat_completions));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionRequest {
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionResponse {
    status: String,
    pruned_content: String,
    compression_ratio: f64,
}

async fn chat_completions(Json(payload): Json<ChatCompletionRequest>) -> impl IntoResponse {
    info!("Received chat completion request");

    // Extract content from the last message
    let content = payload
        .messages
        .last()
        .map(|msg| msg.content.as_str())
        .unwrap_or("");

    let original_len = content.len();
    info!("Original content length: {} bytes", original_len);

    // Parse code into logical blocks
    let blocks = parser::extract_logical_blocks(content, "rust");
    let original_block_count = blocks.len();
    info!("Extracted {} logical blocks", original_block_count);

    // Concurrently embed all blocks
    let embedder = ml::LocalEmbedder::new();
    let embed_futures: Vec<_> = blocks
        .iter()
        .map(|block| embedder.embed_chunk(block.text.to_string()))
        .collect();

    let embeddings = match futures::future::join_all(embed_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(emb) => emb,
        Err(e) => {
            info!("Embedding error: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatCompletionResponse {
                    status: format!("embedding error: {}", e),
                    pruned_content: String::new(),
                    compression_ratio: 0.0,
                }),
            );
        }
    };

    // Prune redundant blocks
    let pruned_blocks = graph::SemanticGraph::prune_redundant_blocks(blocks, embeddings, 0.85);
    info!(
        "Pruned to {} blocks (removed {})",
        pruned_blocks.len(),
        original_block_count - pruned_blocks.len()
    );

    // Reconstruct pruned content
    let pruned_content: String = pruned_blocks.iter().map(|block| block.text).collect();
    let pruned_len = pruned_content.len();

    // Calculate compression metrics
    let compression_ratio = if original_len > 0 {
        (1.0 - (pruned_len as f64 / original_len as f64)) * 100.0
    } else {
        0.0
    };

    info!(
        "COMPRESSION SUCCESS: Saved {:.2}% of context ({} -> {} bytes)",
        compression_ratio, original_len, pruned_len
    );

    let response = ChatCompletionResponse {
        status: "success".to_string(),
        pruned_content,
        compression_ratio,
    };

    (StatusCode::OK, Json(response))
}
