# Query System Examples

This directory contains examples demonstrating Covenant's embedded query system - the ability to compile documentation and data into WASM modules and query them at runtime.

## Examples

| File | Description |
|------|-------------|
| `doc-ingestion.cov` | Tool for processing external documentation into Covenant data nodes |
| `embedded-query.cov` | Basic embedded queries against `kind="data"` snippets |
| `rag-query.cov` | Full RAG (Retrieval-Augmented Generation) system with keyword search and relation traversal |
| `parameterized-query.cov` | Queries with runtime string parameters (dynamic search) |

## Progression

1. **Start with `embedded-query.cov`** - Learn the basics of embedding data and querying with `target="project"`
2. **Explore `rag-query.cov`** - See a complete RAG system with multiple query patterns and relations
3. **Try `parameterized-query.cov`** - Learn how to pass runtime parameters for dynamic queries

## Building

From the project root:

```bash
# Compile all query examples
covenant compile examples/query-system/embedded-query.cov -o examples/query-system/output/embedded-query.wasm
covenant compile examples/query-system/rag-query.cov -o examples/query-system/output/rag-query.wasm
covenant compile examples/query-system/parameterized-query.cov -o examples/query-system/output/parameterized-query.wasm
```

## Testing

From the `examples/query-system/` directory:

```bash
deno run --allow-read test-embedded.ts
deno run --allow-read test-rag.ts
deno run --allow-read test-parameterized.ts
```

## Key Concepts

### Embedding Data

Data is embedded using `kind="data"` snippets:

```covenant
snippet id="docs.hello" kind="data"
  content
    """
    Your documentation content here...
    """
  end
end
```

### Querying Embedded Data

Query functions use `target="project"` to query embedded data:

```covenant
step id="s1" kind="query"
  target="project"
  select all
  from="snippets"
  where
    contains field="content" lit="search term"
  end
  as="results"
end
```

### Parameterized Queries

For runtime parameters, use `var="param_name"` instead of `lit="value"`:

```covenant
snippet id="query.search" kind="fn"
  signature
    fn name="search"
      param name="term" type="String"
      returns type="Any"
    end
  end

  body
    step id="s1" kind="query"
      target="project"
      select all
      from="snippets"
      where
        contains field="content" var="term"
      end
      as="results"
    end
    ...
  end
end
```

Call from TypeScript:

```typescript
const runner = new CovenantQueryRunner();
await runner.load("./output/parameterized-query.wasm");

const results = runner.queryWithString("search", "user input here");
const nodes = runner.getQueryResultNodes(results);
```
