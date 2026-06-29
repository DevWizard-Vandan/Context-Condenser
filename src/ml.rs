use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    embedding: Vec<f32>,
}

pub struct LocalEmbedder {
    client: Client,
}

impl LocalEmbedder {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn embed_chunk(&self, text: String) -> anyhow::Result<Vec<f32>> {
        let request = OllamaRequest {
            model: "nomic-embed-text".to_string(),
            prompt: text,
        };

        let response = self
            .client
            .post("http://localhost:11434/api/embeddings")
            .json(&request)
            .send()
            .await?;

        let ollama_response: OllamaResponse = response.json().await?;

        Ok(ollama_response.embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Ollama to be running"]
    async fn test_embed_chunk_ollama() {
        let embedder = LocalEmbedder::new();
        let text = "Test text for embedding";

        let embedding = embedder.embed_chunk(text.to_string()).await.unwrap();

        // nomic-embed-text outputs 768 dimensions
        assert_eq!(embedding.len(), 768);
    }
}
