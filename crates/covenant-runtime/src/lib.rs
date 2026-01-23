//! covenant-runtime: Runtime Query and Mutation Modules
//!
//! This crate provides runtime implementations of the Covenant query system:
//! - `SymbolStore`: In-memory symbol graph storage with versioning
//! - `QueryEngine`: Execute queries against the symbol store
//! - `Mutator`: Update snippets and trigger recompilation
//!
//! These modules are designed to be compiled to WASM and communicate via
//! WIT interfaces defined in `runtime/wit/covenant-runtime.wit`.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
//! │  query.wasm     │  │  symbols.wasm   │  │  app.wasm       │
//! │  (QueryEngine)  │  │  (SymbolStore)  │  │  (user code)    │
//! │                 │  │                 │  │                 │
//! │  imports:       │  │  exports:       │  │  imports:       │
//! │  - symbols      │  │  - symbols API  │  │  - query        │
//! │  exports:       │  │                 │  │  - symbols      │
//! │  - query API    │  │                 │  │  - mutation     │
//! └─────────────────┘  └─────────────────┘  └─────────────────┘
//! ```

mod error;
mod mutation;
mod query;
mod store;
mod types;

pub use error::RuntimeError;
pub use mutation::Mutator;
pub use query::{QueryEngine, QueryHandle, QueryRequest, QueryResult, QueryStatus};
pub use store::SymbolStore;
pub use types::{RuntimeSymbol, SymbolFilter};
