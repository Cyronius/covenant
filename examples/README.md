# Covenant Examples

Example programs demonstrating Covenant language features.

## Running Examples

```bash
# Check syntax
covenant check examples/<dir>/<file>.cov

# Compile to WASM
covenant build examples/<dir>/<file>.cov -o output.wasm

# Run (Deno target)
covenant run examples/<dir>/<file>.cov
```

## Examples by Category

### Language Fundamentals

| Directory | Description |
|-----------|-------------|
| [syntax-fundamentals/](syntax-fundamentals/) | Core syntax: hello-world, pure functions, pattern matching, higher-order functions, text operations, regex |
| [json/](json/) | JSON parsing, building, and validation |
| [error-handling/](error-handling/) | Union types, validation, and error recovery patterns |

### Effects & I/O

| Directory | Description |
|-----------|-------------|
| [file-io/](file-io/) | File read/write with filesystem effect |
| [http-server/](http-server/) | HTTP server with pure routing logic |
| [multiple-effects/](multiple-effects/) | Combining console, filesystem, and network effects |
| [effect-granularity/](effect-granularity/) | Effect inheritance through call chains |
| [structured-concurrency/](structured-concurrency/) | Parallel and race patterns without async/await |
| [cross-platform-storage/](cross-platform-storage/) | Key-value and document storage (browser/Node/WASI) |
| [platform-abstraction/](platform-abstraction/) | Platform-specific implementations via extern-abstract |

### External Bindings (FFI)

| Directory | Description |
|-----------|-------------|
| [extern-bindings/](extern-bindings/) | Declaring extern bindings to wrap JavaScript/npm libraries |
| [using-bindings/](using-bindings/) | Using extern bindings in Covenant functions |

### Database & Queries

| Directory | Description |
|-----------|-------------|
| [database-access/](database-access/) | Basic database queries and inserts |
| [database-module/](database-module/) | Database schema and CRUD operations |
| [database-dialects/](database-dialects/) | Postgres, SQL Server, MySQL dialect examples |
| [advanced-sql/](advanced-sql/) | CTEs, window functions, recursive queries, JSON ops |

### Query System

| Directory | Description |
|-----------|-------------|
| [query-system/](query-system/) | Comprehensive query examples: data nodes, document ingestion, embedded queries, parameterized queries, RAG retrieval, knowledge base, relation traversal, symbol metadata, performance benchmarks |
| [project-queries/](project-queries/) | Meta-queries on the symbol graph (find callers, dead code, etc.) |

### AST & Metaprogramming

| Directory | Description |
|-----------|-------------|
| [ast-mutations/](ast-mutations/) | Refactoring operations: rename, add, prune |
