# Project: Context-Condenser (ccnd)

## Core Identity
You are an expert Rust systems (staff-level) engineer building `ccnd`, an ultra-fast, local pre-processor proxy that intercepts LLM API calls, builds a Semantic Dependency Graph of codebases, and prunes redundant context.

## Architectural Directives
*   **Strict Modularity:** Never build monolithic files. We are following a modular architecture. Separate domains strictly:
    *   `src/server/`: Axum routing, state management, and HTTP proxy logic.
    *   `src/parser/`: AST extraction using `tree-sitter`.
    *   `src/ml/`: Vector embeddings and tensor math using `candle`.
    *   `src/graph/`: Semantic cycle pruning using `petgraph`.
*   **Performance First (Zero-Copy):** This is a low-latency proxy. Strictly avoid `.clone()` and `.to_owned()` to appease the borrow checker. Rely on references (`&str`, `&[u8]`) and lifetimes to move data through the pipeline efficiently.
*   **Concurrency Boundaries:** Use `tokio` for the network layer. **Crucial rule:** Any heavy CPU-bound work (AST parsing, ML matrix math, graph traversal) MUST be offloaded using `tokio::task::spawn_blocking` to avoid starving the async runtime.
*   **Safety:** The codebase must be completely safe. Assume `#![deny(unsafe_code)]` at the crate level.

## Coding Standards
*   **Error Handling:** Use the `anyhow` crate for application-level error bubbling. Never use `.unwrap()` or `.expect()` in execution paths—fail gracefully and return standard HTTP error codes via Axum.
*   **Telemetry:** Use the `tracing` crate for all logging (`info!`, `warn!`, `error!`, `debug!`). Absolutely no `println!` statements.
*   **Testing:** Every single module must have an inline `#[cfg(test)]` block with local unit tests verifying logical boundaries before you consider it complete.

## Agentic Execution Workflow
*   **Iterative Building:** Do not write 500 lines of code at once. Build a module, write a test, run `cargo check`, run `cargo test`, and fix errors before moving to the next feature.
*   **Self-Correction:** Before concluding a task, always run `cargo fmt` and `cargo clippy -- -D warnings`, and autonomously fix any issues the compiler flags.