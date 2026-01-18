//! Mutation Module - Update snippets and trigger recompilation
//!
//! This module implements the `mutation` interface from WIT.
//! It provides operations to modify Covenant source code at runtime.

use crate::store::SymbolStore;
use crate::types::RuntimeSymbol;
use serde::{Deserialize, Serialize};

/// Result of a mutation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub new_version: u64,
}

impl MutationResult {
    /// Create a successful result
    pub fn ok(version: u64) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            new_version: version,
        }
    }

    /// Create a failed result with errors
    pub fn err(errors: Vec<String>) -> Self {
        Self {
            success: false,
            errors,
            warnings: Vec::new(),
            new_version: 0,
        }
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

/// Result of a compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub success: bool,
    pub errors: Vec<String>,
    /// WASM binary (if successful)
    pub wasm: Option<Vec<u8>>,
}

impl CompileResult {
    /// Create a successful result with WASM binary
    pub fn ok(wasm: Vec<u8>) -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            wasm: Some(wasm),
        }
    }

    /// Create a failed result
    pub fn err(errors: Vec<String>) -> Self {
        Self {
            success: false,
            errors,
            wasm: None,
        }
    }
}

/// Mutator that handles snippet updates and recompilation
pub struct Mutator {
    // In a real implementation, this would hold a reference to the compiler
    // For now, we just provide the interface structure
}

impl Mutator {
    /// Create a new mutator
    pub fn new() -> Self {
        Self {}
    }

    /// Parse and validate a snippet without modifying the symbol store
    ///
    /// This is useful for IDE validation before committing changes.
    pub fn parse_snippet(&self, source: &str) -> MutationResult {
        // In a real implementation, this would:
        // 1. Call the Covenant lexer
        // 2. Call the Covenant parser
        // 3. Return parse errors if any

        // For now, just do basic validation
        if source.trim().is_empty() {
            return MutationResult::err(vec!["Empty source".into()]);
        }

        if !source.contains("snippet") {
            return MutationResult::err(vec!["Source must contain 'snippet' declaration".into()]);
        }

        // Check for basic snippet structure
        if !source.contains("id=") {
            return MutationResult::err(vec!["Snippet must have an 'id' attribute".into()]);
        }

        if !source.contains("kind=") {
            return MutationResult::err(vec!["Snippet must have a 'kind' attribute".into()]);
        }

        if !source.contains("end") {
            return MutationResult::err(vec!["Snippet must be terminated with 'end'".into()]);
        }

        MutationResult::ok(0)
    }

    /// Update a snippet in the symbol store
    ///
    /// This parses the source, validates it, and updates the symbol graph.
    pub fn update_snippet(&self, store: &mut SymbolStore, id: &str, source: &str) -> MutationResult {
        // First, parse and validate
        let parse_result = self.parse_snippet(source);
        if !parse_result.success {
            return parse_result;
        }

        // In a real implementation, this would:
        // 1. Parse the source to AST
        // 2. Extract symbol information
        // 3. Update the symbol store

        // For now, create a placeholder symbol
        let mut symbol = RuntimeSymbol::new(id, "fn");
        symbol.file = "<runtime>".into();

        // Extract some basic info from source (simplified)
        if source.contains("effect database") {
            symbol.effects.push("database".into());
            symbol.effect_closure.push("database".into());
        }
        if source.contains("effect network") {
            symbol.effects.push("network".into());
            symbol.effect_closure.push("network".into());
        }

        let version = store.upsert(symbol);
        store.recompute_backward_refs();

        MutationResult::ok(version)
    }

    /// Delete a snippet from the symbol store
    pub fn delete_snippet(&self, store: &mut SymbolStore, id: &str) -> bool {
        let deleted = store.delete(id);
        if deleted {
            store.recompute_backward_refs();
        }
        deleted
    }

    /// Compile a single snippet to WASM
    ///
    /// This compiles just the specified snippet, assuming all dependencies
    /// are already compiled.
    pub fn compile_snippet(&self, _store: &SymbolStore, id: &str) -> CompileResult {
        // In a real implementation, this would:
        // 1. Get the snippet from the store
        // 2. Run it through the type checker
        // 3. Run it through the optimizer
        // 4. Emit WASM

        // For now, return a placeholder
        CompileResult::err(vec![format!(
            "Compilation not yet implemented for snippet '{}'",
            id
        )])
    }

    /// Recompile a snippet (update + compile in one operation)
    pub fn recompile_snippet(
        &self,
        store: &mut SymbolStore,
        id: &str,
        source: &str,
    ) -> CompileResult {
        // First update
        let update_result = self.update_snippet(store, id, source);
        if !update_result.success {
            return CompileResult::err(update_result.errors);
        }

        // Then compile
        self.compile_snippet(store, id)
    }
}

impl Default for Mutator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_source() {
        let mutator = Mutator::new();
        let result = mutator.parse_snippet("");
        assert!(!result.success);
        assert!(result.errors.iter().any(|e| e.contains("Empty")));
    }

    #[test]
    fn test_parse_valid_snippet() {
        let mutator = Mutator::new();
        let source = r#"
            snippet id="test.foo" kind="fn"
            end
        "#;
        let result = mutator.parse_snippet(source);
        assert!(result.success, "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_missing_id() {
        let mutator = Mutator::new();
        let source = r#"
            snippet kind="fn"
            end
        "#;
        let result = mutator.parse_snippet(source);
        assert!(!result.success);
        assert!(result.errors.iter().any(|e| e.contains("id")));
    }

    #[test]
    fn test_update_snippet() {
        let mut store = SymbolStore::new();
        let mutator = Mutator::new();

        let source = r#"
            snippet id="test.foo" kind="fn"
            effects
              effect database
            end
            end
        "#;

        let result = mutator.update_snippet(&mut store, "test.foo", source);
        assert!(result.success, "Errors: {:?}", result.errors);
        assert!(store.contains("test.foo"));

        let symbol = store.get("test.foo").unwrap();
        assert!(symbol.effect_closure.contains(&"database".to_string()));
    }

    #[test]
    fn test_delete_snippet() {
        let mut store = SymbolStore::new();
        let mutator = Mutator::new();

        // Add a snippet first
        store.upsert(RuntimeSymbol::new("test.foo", "fn"));
        assert!(store.contains("test.foo"));

        // Delete it
        assert!(mutator.delete_snippet(&mut store, "test.foo"));
        assert!(!store.contains("test.foo"));

        // Deleting again returns false
        assert!(!mutator.delete_snippet(&mut store, "test.foo"));
    }
}
