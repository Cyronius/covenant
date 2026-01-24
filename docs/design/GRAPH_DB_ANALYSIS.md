# Graph Database Pressure Test: Analysis

## Overview

This document analyzes Covenant's suitability as a graph database, using the project's own documentation (~40 markdown files, ~23K lines) as the test dataset. Data would be organized into `kind="data"` snippets with typed relations, then queried/traversed via compiled WASM functions.

---

## Data Model Mapping

| Graph DB Concept | Covenant Equivalent |
|-----------------|---------------------|
| Node | `snippet id="..." kind="data"` |
| Node label/type | `kind` + metadata `tags` |
| Node properties | `content` section + `schema` fields |
| Edge | `rel to="target" type=relation_type` |
| Edge type | The 16 fixed relation types |
| Edge properties | Not supported |
| Node summary/index | `note "..."` annotations |
| Subgraph | `contains`/`contained_by` hierarchy |
| Graph traversal | `step kind="traverse" follow type=...` |
| Pattern matching | `step kind="query" target="project" where ...` |

---

## What Works Well

### 1. Snippet-as-Node is Natural

Each data snippet is self-contained with identity, content, metadata, and edges:

```
snippet id="docs.auth_overview" kind="data"
  note "High-level overview of the authentication system"
  content
    """..."""
  end
  relations
    rel to="auth.login" type=describes
    rel to="docs.security_model" type=elaborates_on
  end
  metadata
    author="system"
    tags=["auth", "security"]
  end
end
```

### 2. Bidirectional Relations are Automatic

Declaring `rel to="B" type=describes` on node A automatically produces the inverse `described_by` edge on node B. This is maintained as compiler invariant I5.

### 3. Notes as Metadata Summaries

The `note` field serves as a queryable summary without reading full content -- ideal for fast graph traversal decisions.

### 4. Hierarchical Organization via Contains

`contains`/`contained_by` naturally models document structure (file → sections → subsections).

### 5. Schema-Optional Content

Data nodes can be schema-free (markdown content) or schema-validated (structured records). Both are queryable.

### 6. Query + Traverse Separation

Covenant distinguishes between:
- **Query** (filter/search over all nodes): `step kind="query" target="project" from="snippets" where ...`
- **Traverse** (walk edges from a starting node): `step kind="traverse" from="node_id" follow type=contains depth=3`

This mirrors how graph databases separate index lookups from graph walks.

---

## Critical Implementation Gaps

### Gap 1: Relation Type is Discarded (CRITICAL)

**Location:** Parser reads `type=describes` but AST stores only `RelationKind::To`/`From`, discarding the type string.

**Impact:** Every edge becomes semantically identical. You can't distinguish `describes` from `contains` from `contrasts_with`. The entire graph model collapses to an undirected adjacency list.

**Fix:** Add `rel_type: Option<String>` to `RelationDecl` in the AST, preserve it through parsing and symbol extraction.

**Files:**
- `crates/covenant-ast/src/snippet.rs` (struct definition)
- `crates/covenant-parser/src/parser.rs` (~line 4100, discards the type)
- `crates/covenant-symbols/src/extractor.rs` (~line 318, uses placeholder)

### Gap 2: No Codegen for Project Queries (CRITICAL)

**Location:** `compile_query_step` in `crates/covenant-codegen/src/snippet_wasm.rs`

**Impact:** `target="project"` queries are compiled as if they were external SQL database queries. The generated code calls `db_execute_query` which expects a SQL database connection -- meaningless for querying the project's own snippet graph.

**Fix:** Detect `target="project"` and generate calls to a new host import (`__cov_project_query`) that queries the in-memory/persisted node store.

### Gap 3: Traverse Step Has No Codegen (CRITICAL)

**Location:** `compile_step` in codegen returns `CodegenError::UnsupportedExpression` for `StepKind::Traverse`

**Impact:** Graph traversal functions (examples 19 & 20) parse but cannot compile. The core graph database operation doesn't work.

**Fix:** Implement codegen that marshals traverse parameters and calls a `__cov_project_traverse` host import.

### Gap 4: No WASM Host API for Graph Operations (CRITICAL)

**Impact:** Even if codegen produced correct calls, the runtime has no functions to serve graph queries. The Deno/WASI host doesn't implement any project-query imports.

**Fix:** Define and implement host functions:
- `__cov_project_query(query_json_ptr, len) -> result_ptr`
- `__cov_project_traverse(start_id_ptr, len, rel_type_ptr, len, depth, direction) -> result_ptr`

### Gap 5: Storage Ingestion is a Stub (HIGH)

**Location:** `StorageSync.sync_file` in `crates/covenant-storage/src/sync.rs` (line 81-99)

**Impact:** The function reads a `.cov` file but doesn't parse or store anything. There's no path from authored data files to the storage layer.

**Fix:** Implement the parse → extract → store pipeline, including inverse relation computation.

### Gap 6: RuntimeSymbol Lacks Data Fields (HIGH)

**Impact:** `QueryEngine` operates on `RuntimeSymbol` which has no `content`, `notes`, or typed `relations` fields. The query engine can filter by kind and effects but not by the data-specific fields that matter for graph queries.

**Fix:** Extend `RuntimeSymbol` with content, notes, relations, and metadata fields.

### Gap 7: No Content Text Search (MEDIUM)

**Impact:** `contains field="content" var="topic"` appears in query examples but has no implementation. Content search is essential for finding relevant nodes by keyword.

**Fix:** Implement substring/word-boundary matching in the query engine. Consider full-text indexing for performance.

### Gap 8: No Indexing (MEDIUM)

**Impact:** All queries are O(n) linear scans over the node store. Acceptable for ~200 nodes (the doc dataset), but would degrade for larger graphs (1000+ nodes).

**Fix:** Add indexes for: kind, relation target, metadata tags, content terms.

---

## Language Spec Gaps

These are not bugs but design limitations that may matter for real graph database use:

### Fixed Relation Vocabulary

Only 16 types are allowed:
```
contains, contained_by, next, previous,
describes, described_by, elaborates_on, contrasts_with, example_of,
supersedes, precedes, version_of,
causes, caused_by, motivates, enables,
related_to, depends_on, implements, implemented_by
```

For generic graph data (e.g., social networks, knowledge graphs), this is restrictive. Consider allowing user-defined relation types.

### No Edge Properties

Relations carry only a type. You can't attach:
- Weight/confidence scores
- Timestamps
- Source attribution
- Any other edge metadata

Consider: `rel to="x" type=describes confidence=0.9 source="auto-generated"`

### No Aggregate Queries

No `select count(*)`, `group by`, `sum`, `avg`. For analytics over the graph ("how many nodes have the database effect?"), you'd need to fetch all and count client-side.

### No Path-Returning Traversal

`traverse` returns the set of reachable nodes, not the paths taken. For explaining *how* two nodes are connected (which is often the interesting question), you'd need path information.

### Content Search Semantics Undefined

What does `contains field="content" lit="auth"` match?
- Substring? (`authentication` matches)
- Word boundary? (`auth` only matches as a whole word)
- Case-insensitive?
- Regex?

This needs a clear specification.

### No Limit on Traverse Results

Unbounded traversal on a large connected graph could return the entire database. A `limit=N` option on traverse would be useful.

### No Weighted/Scored Results

Query results are unordered (or lexicographic by ID). There's no mechanism for relevance scoring, which matters for content search.

---

## Feasibility Assessment

### For the Doc Dataset (~200 nodes)

**Feasible with implementation work.** The language design fully supports this use case. With the gaps fixed (Phases 1-3 of the implementation plan), you could:

1. Chunk each markdown file into heading-based snippets
2. Establish `contains` hierarchy and `related_to` cross-references
3. Query by topic (content search)
4. Traverse the doc hierarchy
5. Find code described by specific docs (and vice versa)

### For a General-Purpose Graph Database

**Partially feasible.** The fixed relation vocabulary and lack of edge properties are limiting for arbitrary graph schemas. The model works best for:

- Documentation/knowledge bases
- Code + documentation unification
- Requirements traceability
- Hierarchical taxonomies

It's less suitable for:
- Social graphs (need custom edge types with properties)
- Recommendation engines (need weighted edges and scoring)
- Temporal graphs (need timestamped edges)
- Property graphs in general

### Performance Ceiling

Without indexing (Gap 8), expect:
- Query: O(n) per query, where n = total nodes
- Traverse: O(V + E) per traversal in the subgraph
- Content search: O(n * avg_content_length) per search

For 200 nodes this is fine (sub-millisecond). For 10K+ nodes, indexing becomes necessary.

---

## Architecture: Data as a Compiled Program

### How Data Lives at Runtime

Data snippets don't compile to *executable code* -- they're static content. But query/traverse functions compile to WASM that queries the data at runtime. The architecture is:

```
.cov files (data + query functions)
       ↓ compile
┌──────────────────────────────────┐
│          app.wasm                 │
│  ┌─────────────────────────┐     │
│  │ Compiled query functions │     │
│  │ (traverse, find_by_topic)│     │
│  └─────────┬───────────────┘     │
│            │ calls host imports   │
│            ▼                      │
│  __cov_project_query(...)         │
│  __cov_project_traverse(...)      │
└──────────────────────────────────┘
       ↑ host provides
┌──────────────────────────────────┐
│      Deno/Node Host Runtime       │
│  ┌─────────────────────────┐     │
│  │ Graph Store (in-memory   │     │
│  │ or persisted via redb)   │     │
│  │                          │     │
│  │ Loaded from .cov files   │     │
│  │ at startup               │     │
│  └──────────────────────────┘     │
└──────────────────────────────────┘
```

**Key insight:** The data is loaded by the host before WASM execution begins. The host ingests the `.cov` data files, builds the graph store, then instantiates the WASM module with imports that query that store. The WASM module is purely the query logic.

### Alternative: Embedded Data Segment

For self-contained deployment, data could be embedded in the WASM data segment:

```
compile --embed-data data/*.cov → app.wasm
```

The compiled WASM would contain serialized nodes in its data segment, loaded into the graph store on `_start`. This eliminates the need for separate data files at runtime but increases binary size.

---

## Architecture: Runtime Codegen (Ad-hoc Queries)

### The Problem

Static query functions are compiled ahead-of-time. But a graph database also needs ad-hoc queries -- queries constructed at runtime from user input, natural language, or dynamic conditions.

### The Solution: Host-Side Compiler

The Deno/Node host already has access to the Covenant compiler. The flow for ad-hoc queries:

```
User input (string or natural language)
       ↓
WASM calls host import:
  __cov_compile_and_query(source_ptr, source_len) -> result_ptr
       ↓
Host-side:
  1. Parse the Covenant query source string
  2. Validate it (type check, effect check)
  3. Execute it directly against the graph store
  4. Return results to the WASM caller
       ↓
Results available to the calling function
```

**No WASM-to-WASM compilation needed** -- the host can interpret the query AST directly against the graph store without generating a second WASM module. This is much simpler and faster than JIT compilation.

### Covenant Source Example

```
snippet id="graph.adhoc_query" kind="fn"

effects
  effect meta
end

signature
  fn name="adhoc_query"
    param name="query_source" type="String"
    returns collection of="Node"
  end
end

body
  // Pass Covenant query source to the host for compilation + execution
  step id="s1" kind="call"
    fn="meta.compile_and_query"
    arg name="source" from="query_source"
    as="results"
  end
  step id="s2" kind="return"
    from="results"
    as="_"
  end
end

end
```

### Natural Language Path

For natural language → query translation:

```
"Find all docs related to authentication"
       ↓ (LLM generates Covenant query)
"select all from=\"snippets\" where contains field=\"content\" lit=\"auth\""
       ↓ (host compiles and executes)
[matching nodes]
```

This could be a two-step host import:
1. `__cov_nl_to_query(nl_ptr, nl_len) -> query_source_ptr` (LLM translation)
2. `__cov_compile_and_query(query_source_ptr, len) -> result_ptr` (compile + execute)

Or a single combined import:
- `__cov_nl_query(nl_ptr, nl_len) -> result_ptr`

### The `meta` Effect Gates This

Ad-hoc query compilation requires the `meta` effect, ensuring:
- Only explicitly authorized functions can run dynamic queries
- The effect propagates transitively (callers must also declare `meta`)
- Pure functions can never trigger compilation

### What's Already Designed (WIT)

The existing WIT interface already specifies the pieces:

```wit
// mutation interface (from covenant-runtime.wit)
parse-snippet: func(source: string) -> mutation-result;
compile-snippet: func(id: string) -> compile-result;
```

For graph queries, we'd add:

```wit
interface graph-query {
    // Compile and execute a Covenant query string against the graph store
    compile-and-query: func(source: string) -> result<query-result, query-error>;

    // Execute a pre-parsed query (for repeated queries)
    execute-parsed: func(query-id: u64) -> result<query-result, query-error>;

    // Parse and cache a query for repeated execution
    prepare-query: func(source: string) -> result<u64, query-error>;
}
```

---

## Architecture: Standard Query Interface (Cross-Module)

### The Problem

If multiple Covenant data modules exist (docs, requirements, team data, etc.), each compiled to its own WASM, how do TypeScript frontends or other WASM modules query them uniformly?

### The Solution: Exported Function Convention

Every "document module" exports a standard set of functions:

```
// Standard exports for any Covenant document module:

cov_query(query_json_ptr: i32, query_json_len: i32) -> i64  // fat ptr to result JSON
cov_traverse(start_id_ptr: i32, start_id_len: i32,
             rel_type_ptr: i32, rel_type_len: i32,
             depth: i32, direction: i32) -> i64              // fat ptr to result JSON
cov_get_node(id_ptr: i32, id_len: i32) -> i64               // fat ptr to node JSON
cov_list_nodes(filter_json_ptr: i32, filter_json_len: i32) -> i64
cov_get_schema() -> i64                                      // fat ptr to schema JSON
```

### TypeScript Integration

A TypeScript wrapper around any conforming module:

```typescript
// covenant-graph-client.ts

interface CovGraphModule {
  memory: WebAssembly.Memory;
  cov_query(ptr: number, len: number): bigint;
  cov_traverse(startPtr: number, startLen: number,
               relPtr: number, relLen: number,
               depth: number, direction: number): bigint;
  cov_get_node(ptr: number, len: number): bigint;
  cov_list_nodes(filterPtr: number, filterLen: number): bigint;
  cov_get_schema(): bigint;
}

class CovGraphClient {
  private module: CovGraphModule;

  async query(where: object, options?: QueryOptions): Promise<Node[]> {
    const queryJson = JSON.stringify({ where, ...options });
    const [ptr, len] = this.writeString(queryJson);
    const resultFatPtr = this.module.cov_query(ptr, len);
    return JSON.parse(this.readFatPtr(resultFatPtr));
  }

  async traverse(startId: string, relType: string,
                 depth?: number, direction?: 'outgoing'|'incoming'|'both'): Promise<Node[]> {
    // ... marshal and call cov_traverse
  }

  async getNode(id: string): Promise<Node | null> {
    // ... marshal and call cov_get_node
  }
}
```

### Usage from a TypeScript Frontend

```typescript
import { CovGraphClient } from '@covenant/graph-client';

// Load any conforming document module
const docs = await CovGraphClient.load('./docs.wasm');
const reqs = await CovGraphClient.load('./requirements.wasm');

// Same API regardless of which module
const authDocs = await docs.query({
  kind: 'data',
  content_contains: 'authentication'
});

const tree = await docs.traverse('kb.root', 'contains', Infinity, 'outgoing');

// Cross-module: find requirements related to docs
const relatedReqs = await reqs.query({
  kind: 'data',
  rel_to: { target: 'auth.login', type: 'implements' }
});
```

### Schema Discovery

`cov_get_schema()` returns metadata about the module's data:

```json
{
  "node_count": 247,
  "relation_types": ["contains", "describes", "elaborates_on", "related_to"],
  "metadata_keys": ["author", "tags", "source_file"],
  "queryable_fields": ["id", "kind", "content", "notes", "relations", "metadata"],
  "supports_content_search": true,
  "supports_traverse": true
}
```

This enables generic graph browsers to discover what a module contains without prior knowledge.

### Multiple Modules, One Query

For querying across multiple document modules (federated queries):

```typescript
// Federated graph query across multiple compiled modules
const federation = new CovGraphFederation([docs, reqs, teamData]);

// Searches all modules, merges results
const results = await federation.query({ content_contains: 'security' });

// Cross-module traversal (follows relations that reference nodes in other modules)
const crossRefs = await federation.traverse('docs.auth_overview', 'describes', 2, 'outgoing');
```

This works because snippet IDs are globally unique and relations reference snippet IDs regardless of which module they live in.

---

## Architecture: Data Compiled Into WASM

The data is compiled directly into the WASM binary. One `.wasm` file = the complete database + query engine. No host-side graph store needed for reads.

```
.cov data files + .cov query functions
       ↓ covenant compile
┌──────────────────────────────────────────────────┐
│                    app.wasm                        │
│                                                    │
│  ┌────────────────────────────────────────────┐   │
│  │ Data Segment                                │   │
│  │ (serialized graph: nodes, content, edges)   │   │
│  └────────────────────────────────────────────┘   │
│                                                    │
│  ┌────────────────────────────────────────────┐   │
│  │ Graph Access Interface (internal)           │   │
│  │ _gai_node_count() -> i32                    │   │
│  │ _gai_get_node(idx) -> fat_ptr               │   │
│  │ _gai_get_content(idx) -> fat_ptr            │   │
│  │ _gai_get_relations(idx) -> fat_ptr          │   │
│  │ _gai_find_by_id(id_ptr, len) -> idx         │   │
│  └────────────────────────────────────────────┘   │
│                                                    │
│  ┌────────────────────────────────────────────┐   │
│  │ Compiled Query/Traverse Functions           │   │
│  │ (call GAI to access data)                   │   │
│  └────────────────────────────────────────────┘   │
│                                                    │
│  ┌────────────────────────────────────────────┐   │
│  │ Standard Exports                            │   │
│  │ cov_query, cov_traverse, cov_get_node, ... │   │
│  └────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────┘
```

### Graph Access Interface (GAI)

The key architectural insight is an **internal abstraction layer** between query logic and data representation. Query functions never access the data segment directly -- they call GAI functions. This means the underlying representation can change without affecting query code.

**GAI functions (internal, not exported):**

```
_gai_node_count() -> i32
_gai_get_node_id(idx: i32) -> fat_ptr          // String ID
_gai_get_node_kind(idx: i32) -> i32            // Enum
_gai_get_node_content(idx: i32) -> fat_ptr     // Content text
_gai_get_node_notes(idx: i32) -> fat_ptr       // Notes array
_gai_get_node_metadata(idx: i32, key_ptr, key_len) -> fat_ptr
_gai_get_outgoing_rels(idx: i32) -> fat_ptr    // [(target_idx, rel_type)]
_gai_get_incoming_rels(idx: i32) -> fat_ptr    // [(source_idx, rel_type)]
_gai_find_by_id(id_ptr: i32, id_len: i32) -> i32  // -1 if not found
_gai_content_contains(idx: i32, term_ptr, term_len) -> i32  // bool
```

### Internal Representation Options

The GAI abstraction means the compiler can choose the best representation without affecting the query interface:

#### Option A: Static Arrays + Offset Tables (Recommended Initial)

```
Data Segment Layout:
  [String Pool]       "kb.root\0docs.design\0auth...\0..."
  [Node ID Table]     [(offset, len), (offset, len), ...]  // into string pool
  [Content Table]     [(offset, len), ...]                  // into string pool
  [Notes Table]       [(offset, len, count), ...]           // array of note ptrs
  [Relations Table]   [(from_idx, to_idx, type_enum), ...]  // flat edge list
  [Adjacency Index]   [(out_start, out_count, in_start, in_count), ...]  // per node
  [ID Hash Table]     [hash -> node_idx]                    // for O(1) lookup by ID
```

| Property | Rating | Notes |
|----------|--------|-------|
| Cold start | Excellent | Zero deserialization; data usable immediately |
| Memory | Excellent | No runtime copy; data segment IS the database |
| Query speed | Good | Direct array access, O(1) by index |
| ID lookup | Good | Hash table in data segment |
| Content search | Fair | Linear scan of content table (no full-text index) |
| Compiler complexity | Medium | Must compute layouts, hash tables, adjacency lists |
| Flexibility | Via GAI | Representation hidden behind GAI functions |

#### Option B: Serialized Graph (JSON/MessagePack)

```
Data Segment:
  [Single serialized blob: { nodes: [...], relations: [...] }]

On _start():
  Deserialize blob → allocate typed structs in linear memory
  Build hash table for ID lookup
  Build adjacency lists
```

| Property | Rating | Notes |
|----------|--------|-------|
| Cold start | Poor | Must deserialize entire graph before queries work |
| Memory | Poor | 2x: data segment + deserialized copy |
| Query speed | Good | Same as A after initialization |
| ID lookup | Good | Hash table built at init |
| Content search | Fair | Same linear scan |
| Compiler complexity | Low | Just serialize to JSON |
| Flexibility | Via GAI | Same abstraction |

#### Option C: Hybrid (Static structure, lazy content)

```
Data Segment:
  [Node metadata arrays]  // IDs, kinds, relation indices -- always loaded
  [Content offsets]        // Pointers into content pool
  [Content pool]           // Actual text, accessed on demand

Query functions access metadata directly, content lazily via offset.
```

**Recommendation:** Start with **Option A** (static arrays). The GAI abstraction means switching to B or C later is a compiler change, not a language change.

---

## Architecture: Multi-Module Partitioning

For large datasets, a single WASM module may be too large. The data can be split across multiple modules:

```
┌──────────────────────────┐
│  query-engine.wasm        │
│  (query/traverse logic)   │
│  Imports: data_module.*   │
└────────────┬─────────────┘
             │ loads dynamically
    ┌────────┼────────┐
    ▼        ▼        ▼
┌────────┐ ┌────────┐ ┌────────┐
│data-a  │ │data-b  │ │data-c  │
│.wasm   │ │.wasm   │ │.wasm   │
│        │ │        │ │        │
│Nodes   │ │Nodes   │ │Nodes   │
│0-99    │ │100-199 │ │200-299 │
└────────┘ └────────┘ └────────┘
```

### How It Works (Host-Mediated)

WASM modules can't directly instantiate other WASM modules. The host mediates:

1. Query engine calls host import: `__cov_load_partition("data-b.wasm")`
2. Host instantiates `data-b.wasm` via `WebAssembly.instantiate()`
3. Host reads data-b's exported GAI functions
4. Host provides results back to query engine
5. When done, host can drop the data module reference (GC'd)

```typescript
// Host-side (Deno):
const partitions: Map<string, WebAssembly.Instance> = new Map();

const imports = {
  covenant_graph: {
    load_partition: (namePtr: number, nameLen: number): number => {
      const name = readStr(namePtr, nameLen);
      if (!partitions.has(name)) {
        const bytes = Deno.readFileSync(`${name}.wasm`);
        const { instance } = WebAssembly.instantiateSync(bytes, {});
        partitions.set(name, instance);
      }
      return partitions.get(name)!.exports._gai_node_count() as number;
    },
    query_partition: (partitionId, queryPtr, queryLen): bigint => {
      // Execute query against specific partition's GAI
    },
    unload_partition: (namePtr, nameLen) => {
      partitions.delete(readStr(namePtr, nameLen));
      // GC will reclaim the module's memory
    }
  }
};
```

### Partition Schemes

| Scheme | How | Best For |
|--------|-----|----------|
| **By namespace prefix** | `kb.design.*` in partition A, `kb.faq.*` in B | Hierarchical data with locality |
| **By relation cluster** | Graph partitioning algorithm (minimize cross-edges) | Traversal-heavy workloads |
| **By size** | N nodes per partition | Predictable memory usage |
| **By access pattern** | Hot data in one partition, cold in others | Read-heavy with skewed access |
| **By topic/tag** | All "auth" tagged nodes together | Topic-focused queries |

### Trade-offs

| Aspect | Single Module | Multi-Module |
|--------|---------------|--------------|
| Startup | Load one file | Load engine + relevant partitions |
| Memory peak | All data in memory | Only loaded partitions |
| Cross-partition queries | N/A (all local) | Require loading multiple partitions |
| Cross-partition traversal | N/A | Must detect cross-partition edges |
| Deployment | One file | Multiple files |
| Complexity | Simple | Host-mediated communication |

### Cross-Partition Edges

When node A (partition 1) has a relation to node B (partition 2):
- The relation is stored in partition 1 with a **global ID** reference
- Query engine detects the target is in a different partition
- Loads partition 2 (or uses cached instance)
- Resolves the reference

This works because snippet IDs are globally unique. The partition scheme just needs a routing function: `partition_for(snippet_id) -> partition_name`.

### Recommendation

Start with **single-module** (all data in one WASM). Add partitioning only when:
- Data exceeds ~10MB compiled (WASM modules load synchronously in many runtimes)
- Memory pressure requires loading subsets
- The partition boundaries are natural (e.g., separate knowledge domains)

The GAI abstraction makes this transition seamless -- query functions don't change, only the GAI implementation switches from "read local data segment" to "call host to query partition."

---

## Summary of Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Where data lives at runtime | Embedded in WASM data segment | Self-contained module; zero cold-start for reads; one file = complete DB |
| Data access abstraction | Graph Access Interface (GAI) -- internal WASM functions | Decouples query logic from representation; can swap offset tables for serialized format later |
| Initial representation | Static arrays + offset tables | Zero deserialization; direct memory access; minimal memory overhead |
| Query execution | Compiled loops over GAI (no host imports for reads) | Self-contained; works without host runtime |
| Runtime codegen (ad-hoc) | Host-side compiler interprets query AST directly | Only needed for dynamic queries; `meta` effect gates access |
| Standard query interface | Exported function convention (`cov_query`, `cov_traverse`, etc.) | Works with current runtime; TypeScript can call directly |
| Large datasets | Multi-module partitioning via host-mediated loading | GAI abstraction makes it transparent; partition schemes by prefix/cluster/size |
| Cross-module federation | Globally unique snippet IDs + same exported interface | Enables querying multiple modules with one client |

---

## Recommendations

1. **Fix Gap 1 first** -- Without relation types, nothing else works
2. **Implement Phases 1-3** of the implementation plan to get end-to-end functionality
3. **Build the doc dataset** (Phase 4) as concrete validation
4. **Define the standard exported function convention** -- This enables TypeScript integration from day one
5. **Implement host-side query interpretation** for ad-hoc queries (simpler than JIT)
6. **Consider relaxing the relation vocabulary** -- either allow arbitrary strings or add a mechanism for declaring custom types
7. **Define content search semantics** explicitly in the spec
8. **Defer edge properties, aggregates, and natural language** -- useful but not blocking for initial validation
