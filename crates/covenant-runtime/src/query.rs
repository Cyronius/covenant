//! Query Engine - Execute queries against the symbol store
//!
//! This module implements the `query` interface from WIT.
//! It supports both synchronous and asynchronous query execution.

use crate::error::RuntimeError;
use crate::store::SymbolStore;
use crate::types::{RuntimeSymbol, SymbolFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Opaque handle for async queries
pub type QueryHandle = u64;

/// Status of an async query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStatus {
    Pending,
    Complete,
    Error,
    Cancelled,
}

/// Query request matching the WIT interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    /// What to select: "all" or comma-separated field names
    pub select_clause: String,

    /// Type to query: "functions", "structs", "requirements", "tests", "steps"
    pub from_type: String,

    /// Optional where clause as JSON
    pub where_clause: Option<String>,

    /// Optional ordering: "field:asc" or "field:desc"
    pub order_by: Option<String>,

    /// Optional limit on results
    pub limit: Option<u32>,

    /// Optional offset for pagination
    pub offset: Option<u32>,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Matching symbols
    pub symbols: Vec<RuntimeSymbol>,

    /// Version at which query was executed
    pub version: u64,

    /// Whether more results exist (for pagination)
    pub has_more: bool,
}

/// Internal representation of an async query
struct AsyncQuery {
    request: QueryRequest,
    status: QueryStatus,
    result: Option<Result<QueryResult, RuntimeError>>,
}

/// Query engine that executes queries against a symbol store
pub struct QueryEngine {
    /// Next handle ID
    next_handle: AtomicU64,

    /// Pending async queries
    pending: HashMap<QueryHandle, AsyncQuery>,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(1),
            pending: HashMap::new(),
        }
    }

    /// Execute a query synchronously
    pub fn execute(&self, store: &SymbolStore, request: &QueryRequest) -> Result<QueryResult, RuntimeError> {
        // Parse the from_type to determine what kind of symbols to query
        let kind_filter = match request.from_type.as_str() {
            "functions" => Some("fn".to_string()),
            "structs" => Some("struct".to_string()),
            "enums" => Some("enum".to_string()),
            "modules" => Some("module".to_string()),
            "databases" => Some("database".to_string()),
            "externs" => Some("extern".to_string()),
            "all" | "*" => None,
            other => return Err(RuntimeError::InvalidQuery(format!("Unknown from_type: {}", other))),
        };

        // Build the filter
        let mut filter = SymbolFilter::default();
        filter.kind = kind_filter;

        // Parse where clause if present
        if let Some(ref where_json) = request.where_clause {
            self.apply_where_clause(&mut filter, where_json)?;
        }

        // Execute the query
        let mut symbols: Vec<RuntimeSymbol> = store
            .list(&filter)
            .into_iter()
            .cloned()
            .collect();

        // Apply ordering
        if let Some(ref order) = request.order_by {
            self.apply_ordering(&mut symbols, order)?;
        } else {
            // Default: lexicographic by ID (deterministic)
            symbols.sort_by(|a, b| a.id.cmp(&b.id));
        }

        // Apply pagination
        let total_count = symbols.len();
        let offset = request.offset.unwrap_or(0) as usize;
        let limit = request.limit.map(|l| l as usize);

        let symbols = if offset > 0 || limit.is_some() {
            let start = offset.min(symbols.len());
            let end = limit.map(|l| (start + l).min(symbols.len())).unwrap_or(symbols.len());
            symbols[start..end].to_vec()
        } else {
            symbols
        };

        let has_more = offset + symbols.len() < total_count;

        Ok(QueryResult {
            symbols,
            version: store.version(),
            has_more,
        })
    }

    /// Start an async query and return a handle
    pub fn start_query(&mut self, request: QueryRequest) -> QueryHandle {
        let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);

        self.pending.insert(handle, AsyncQuery {
            request,
            status: QueryStatus::Pending,
            result: None,
        });

        handle
    }

    /// Poll the status of an async query
    pub fn poll_query(&self, handle: QueryHandle) -> QueryStatus {
        self.pending
            .get(&handle)
            .map(|q| q.status)
            .unwrap_or(QueryStatus::Error)
    }

    /// Execute a pending async query
    ///
    /// In a real WASM environment, this would be called by the runtime
    /// when the query is ready to be processed.
    pub fn process_query(&mut self, handle: QueryHandle, store: &SymbolStore) {
        // First, get the request if the query is pending
        let request = match self.pending.get(&handle) {
            Some(query) if query.status == QueryStatus::Pending => {
                query.request.clone()
            }
            _ => return,
        };

        // Execute the query
        let result = self.execute(store, &request);
        let status = if result.is_ok() {
            QueryStatus::Complete
        } else {
            QueryStatus::Error
        };

        // Update the query
        if let Some(query) = self.pending.get_mut(&handle) {
            query.result = Some(result);
            query.status = status;
        }
    }

    /// Get the result of an async query (if complete)
    pub fn get_result(&mut self, handle: QueryHandle) -> Option<Result<QueryResult, String>> {
        self.pending.get(&handle).and_then(|q| {
            q.result.as_ref().map(|r| {
                r.clone().map_err(|e| e.to_string())
            })
        })
    }

    /// Cancel an async query
    pub fn cancel_query(&mut self, handle: QueryHandle) {
        if let Some(query) = self.pending.get_mut(&handle) {
            if query.status == QueryStatus::Pending {
                query.status = QueryStatus::Cancelled;
                query.result = Some(Err(RuntimeError::QueryCancelled));
            }
        }
    }

    /// Clean up completed queries
    pub fn cleanup_completed(&mut self) {
        self.pending.retain(|_, q| q.status == QueryStatus::Pending);
    }

    /// Parse and apply a where clause to a filter
    fn apply_where_clause(&self, filter: &mut SymbolFilter, where_json: &str) -> Result<(), RuntimeError> {
        // Parse the where clause JSON
        // Expected format: {"field": "value"} or {"contains": {"field": "effects", "value": "database"}}
        let parsed: serde_json::Value = serde_json::from_str(where_json)
            .map_err(|e| RuntimeError::InvalidQuery(format!("Invalid where clause JSON: {}", e)))?;

        if let Some(obj) = parsed.as_object() {
            // Simple equality checks
            if let Some(effect) = obj.get("has_effect").and_then(|v| v.as_str()) {
                filter.has_effect = Some(effect.to_string());
            }
            if let Some(calls) = obj.get("calls").and_then(|v| v.as_str()) {
                filter.calls_fn = Some(calls.to_string());
            }
            if let Some(called_by) = obj.get("called_by").and_then(|v| v.as_str()) {
                filter.called_by_fn = Some(called_by.to_string());
            }
            if let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) {
                filter.kind = Some(kind.to_string());
            }

            // Handle "contains" operator for effects
            if let Some(contains) = obj.get("contains").and_then(|v| v.as_object()) {
                let field = contains.get("field").and_then(|v| v.as_str());
                let value = contains.get("value").and_then(|v| v.as_str());

                if let (Some("effects"), Some(effect_value)) = (field, value) {
                    filter.has_effect = Some(effect_value.to_string());
                }
            }
        }

        Ok(())
    }

    /// Apply ordering to results
    fn apply_ordering(&self, symbols: &mut [RuntimeSymbol], order: &str) -> Result<(), RuntimeError> {
        let parts: Vec<&str> = order.split(':').collect();
        let field = parts.get(0).copied().unwrap_or("id");
        let dir = parts.get(1).copied().unwrap_or("asc");

        let ascending = match dir {
            "asc" => true,
            "desc" => false,
            _ => return Err(RuntimeError::InvalidQuery(format!("Invalid order direction: {}", dir))),
        };

        match field {
            "id" => {
                if ascending {
                    symbols.sort_by(|a, b| a.id.cmp(&b.id));
                } else {
                    symbols.sort_by(|a, b| b.id.cmp(&a.id));
                }
            }
            "kind" => {
                if ascending {
                    symbols.sort_by(|a, b| a.kind.cmp(&b.kind));
                } else {
                    symbols.sort_by(|a, b| b.kind.cmp(&a.kind));
                }
            }
            "file" => {
                if ascending {
                    symbols.sort_by(|a, b| a.file.cmp(&b.file));
                } else {
                    symbols.sort_by(|a, b| b.file.cmp(&a.file));
                }
            }
            "line" => {
                if ascending {
                    symbols.sort_by(|a, b| a.line.cmp(&b.line));
                } else {
                    symbols.sort_by(|a, b| b.line.cmp(&a.line));
                }
            }
            _ => return Err(RuntimeError::InvalidQuery(format!("Unknown order field: {}", field))),
        }

        Ok(())
    }
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_store() -> SymbolStore {
        let mut store = SymbolStore::new();

        let mut fn1 = RuntimeSymbol::new("auth.login", "fn");
        fn1.effect_closure = vec!["database".into(), "network".into()];
        fn1.calls = vec!["db.query".into()];
        store.upsert(fn1);

        let mut fn2 = RuntimeSymbol::new("db.query", "fn");
        fn2.effect_closure = vec!["database".into()];
        store.upsert(fn2);

        let fn3 = RuntimeSymbol::new("math.add", "fn");
        store.upsert(fn3);

        let struct1 = RuntimeSymbol::new("types.User", "struct");
        store.upsert(struct1);

        store
    }

    #[test]
    fn test_query_all_functions() {
        let store = setup_store();
        let engine = QueryEngine::new();

        let request = QueryRequest {
            select_clause: "all".into(),
            from_type: "functions".into(),
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
        };

        let result = engine.execute(&store, &request).unwrap();
        assert_eq!(result.symbols.len(), 3);
        assert!(result.symbols.iter().all(|s| s.kind == "fn"));
    }

    #[test]
    fn test_query_with_effect_filter() {
        let store = setup_store();
        let engine = QueryEngine::new();

        let request = QueryRequest {
            select_clause: "all".into(),
            from_type: "functions".into(),
            where_clause: Some(r#"{"has_effect": "database"}"#.into()),
            order_by: None,
            limit: None,
            offset: None,
        };

        let result = engine.execute(&store, &request).unwrap();
        assert_eq!(result.symbols.len(), 2);
        assert!(result.symbols.iter().all(|s| s.effect_closure.contains(&"database".to_string())));
    }

    #[test]
    fn test_query_with_ordering() {
        let store = setup_store();
        let engine = QueryEngine::new();

        let request = QueryRequest {
            select_clause: "all".into(),
            from_type: "functions".into(),
            where_clause: None,
            order_by: Some("id:desc".into()),
            limit: None,
            offset: None,
        };

        let result = engine.execute(&store, &request).unwrap();
        assert_eq!(result.symbols[0].id, "math.add");
        assert_eq!(result.symbols[1].id, "db.query");
        assert_eq!(result.symbols[2].id, "auth.login");
    }

    #[test]
    fn test_query_with_pagination() {
        let store = setup_store();
        let engine = QueryEngine::new();

        let request = QueryRequest {
            select_clause: "all".into(),
            from_type: "functions".into(),
            where_clause: None,
            order_by: Some("id:asc".into()),
            limit: Some(2),
            offset: Some(0),
        };

        let result = engine.execute(&store, &request).unwrap();
        assert_eq!(result.symbols.len(), 2);
        assert!(result.has_more);
        assert_eq!(result.symbols[0].id, "auth.login");
        assert_eq!(result.symbols[1].id, "db.query");
    }

    #[test]
    fn test_async_query() {
        let store = setup_store();
        let mut engine = QueryEngine::new();

        let request = QueryRequest {
            select_clause: "all".into(),
            from_type: "all".into(),
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
        };

        // Start async query
        let handle = engine.start_query(request);
        assert_eq!(engine.poll_query(handle), QueryStatus::Pending);

        // Process the query
        engine.process_query(handle, &store);
        assert_eq!(engine.poll_query(handle), QueryStatus::Complete);

        // Get result
        let result = engine.get_result(handle).unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().symbols.len(), 4);
    }

    #[test]
    fn test_cancel_query() {
        let mut engine = QueryEngine::new();

        let request = QueryRequest {
            select_clause: "all".into(),
            from_type: "all".into(),
            where_clause: None,
            order_by: None,
            limit: None,
            offset: None,
        };

        let handle = engine.start_query(request);
        engine.cancel_query(handle);

        assert_eq!(engine.poll_query(handle), QueryStatus::Cancelled);
    }
}
