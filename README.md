# Covenant

**A programming language where code is queryable data.**

Covenant is designed for a future where AI agents and humans collaborate on codebases. Traditional languages optimize for human authorship—Covenant optimizes for machine comprehension without sacrificing readability.

---

## The Problem

AI coding agents struggle with existing codebases:

- They parse **text**, not semantic structure
- They must **search and guess** instead of query
- Tool use is **probabilistic**, not contractual
- Each task starts from scratch—benefits don't compound
- Reliability degrades as codebases grow larger

We've been writing code for humans to read and machines to execute. But now machines need to *understand* code too—and grep isn't good enough.

---

## The Vision

> *Humans write contracts. AI navigates structure. Tools do the work. Every line makes the next easier.*

Covenant separates concerns across four layers:

| Layer | Purpose | Lifetime |
|-------|---------|----------|
| **Natural Language** | Human↔AI communication | Ephemeral (discarded after translation) |
| **Source Code** | Contracts, intent, constraints | Permanent artifact |
| **AST** | Queryable symbol graph | What AI agents operate against |
| **Bytecode** | WASM execution | Sandboxed, metered, deterministic |

The key insight: **source code should be as queryable as a database**. Not through text search, but through structured queries over a semantic graph.

---

## Core Principles

### Code is Data
Every symbol has computed metadata—who calls it, what it calls, what effects it has. Query your codebase like you query a database:

```covenant
find_db_functions() -> FunctionInfo[]
    import { project } from meta
{
    query project {
        select * from functions
        where effects contains "database"
    }
}
```

### Contracts, Not Comments
Types encode what a function *can* do, not just what it returns. Effects are declared as imports—capabilities, not annotations:

```covenant
get_user(id: Int) -> User | DbError
    import { app_db } from database
{
    query app_db {
        select * from users where id = id limit 1
    }
}
```

The function signature tells you everything: it takes an `Int`, returns a `User` or a `DbError`, and requires database access to `app_db`. No hidden side effects.

### Bidirectional References
The compiler computes `called_by` for every function. Find all callers without grep:

```covenant
find_auth_callers() -> FunctionInfo[]
    import { project } from meta
{
    query project {
        select * from functions
        where calls contains "authenticate"
    }
}
```

### Compounding Clarity
Each function you write makes the next one easier. Typed contracts, queryable structure, and computed metadata mean AI agents can navigate with precision instead of probability.

---

## What Makes It Different

| Traditional Languages | Covenant |
|-----------------------|----------|
| Search code with grep/regex | Query code with SQL-like syntax |
| Effects are implicit | Effects declared as imports |
| "Who calls this?" requires tooling | `called_by` computed automatically |
| Comments describe intent | Types encode intent |
| Each file is isolated text | Codebase is a queryable graph |

**Unified query syntax**: The same SQL-like syntax works for databases *and* source code. The import determines semantics:

```covenant
// Query external database (compiles to SQL)
query app_db { select * from users where is_active = true }

// Query source code (compiles to AST traversal)
query project { select * from functions where is_exported = false and called_by = [] }
```

**WASM target**: Compiles to WebAssembly for sandboxed, capability-constrained, metered execution.

---

## Documentation

| Doc | Purpose |
|-----|---------|
| [DESIGN.md](DESIGN.md) | Philosophy, four-layer model, core decisions |
| [grammar.ebnf](grammar.ebnf) | Formal syntax definition |
| [examples/](examples/) | Example `.cov` source files |
| [prior-art.md](prior-art.md) | Lessons from Austral and Koka |

---

## Status

**Design phase.** No compiler exists yet.

Current focus: finalize syntax, define AST, build parser.

---

## License

MIT
