# Pressure Test: Covenant Use Cases

This document evaluates whether Covenant's current design can support various real-world use cases, identifying capabilities and limitations.

---

## Use Case 1: VS Code Extension with Covenant WASM + OpenCode

**Goal:** Run Covenant inside VS Code as a TypeScript extension, compiled to WASM, communicating with OpenCode.

### Verdict: FEASIBLE with caveats

**What works:**
- WASM compilation to `--target=browser` or `--target=node` exists
- Effect system supports `network` for API communication (to OpenCode)
- Extern bindings can wrap VS Code extension APIs via `contract="..."` metadata
- Symbol graph queries (the WIT `query` interface) enable code intelligence features
- Platform abstraction (`ExternAbstract` + `ExternImpl`) handles Node.js runtime in VS Code

**Limitations:**
- **No DOM/UI effects** - VS Code webview rendering would need extern bindings (not built-in)
- **Event-driven model** - VS Code extensions are callback-heavy; Covenant's structured concurrency (`parallel`/`race`) doesn't map directly to event listeners
- **TypeScript interop** - Calling TS functions requires defining extern snippets for each VS Code API
- **Async model mismatch** - VS Code APIs are Promise-based; Covenant uses structured concurrency which compiles differently

**Required additions:**
- Extern bindings for `vscode` namespace APIs
- An event adapter pattern (extern that bridges VS Code events → Covenant handlers)
- UI rendering would remain in TypeScript, with Covenant handling logic

---

## Use Case 2: Run DOOM in a Browser

**Goal:** Implement or port DOOM to run in a browser using Covenant compiled to WASM.

### Verdict: NOT FEASIBLE

**Blockers:**

| Requirement | Covenant Support | Impact |
|-------------|------------------|--------|
| Mutable arrays | Not supported | Frame buffer, BSP tree, sprite data need in-place updates |
| Bit manipulation | Not supported | WAD parsing, texture mapping, color palettes |
| While loops | Not supported | Game loop requires `while (running) { ... }` |
| Graphics/rendering | No effects exist | Canvas, WebGL bindings don't exist |
| Raw memory access | Not supported | Direct byte-level framebuffer manipulation |
| Real-time execution | Cost constraints | 35fps rendering would exceed query budgets |
| Mutable game state | SSA/immutability | Player position, enemies, ammo need mutable updates |

**Fundamental issue:** Covenant is designed for "machine-first IR" with deterministic, query-based execution. DOOM is a tight imperative game loop with mutable state, bit-level operations, and real-time constraints. These are architectural opposites.

---

## Use Case 3: Run DOOM as an Electron App

**Goal:** Run DOOM inside Electron using Covenant WASM.

### Verdict: NOT FEASIBLE

Electron provides a Node.js + Chromium runtime, but doesn't change Covenant's computational model. The core DOOM requirements remain unsupported:

- Mutable game state
- Bitwise operations
- Real-time rendering loop
- Direct framebuffer access

---

## Use Case 4: Full Text Editor as Electron App

**Goal:** Implement a text editor in Electron with Covenant helpers for rendering and document management.

### Verdict: PARTIALLY FEASIBLE (split architecture)

**What Covenant can handle:**
- **Document model** - Structs for lines, paragraphs, selections
- **Command system** - Pattern matching for keyboard shortcuts → actions
- **Undo/redo** - Immutable snapshots of document state (natural fit for SSA)
- **File I/O** - `effect filesystem` with extern bindings
- **Search/replace** - Query-based text operations
- **Syntax highlighting rules** - Declarative pattern matching
- **LSP integration** - `effect network` for language server communication

**What Covenant cannot handle:**
- **Rendering** - No DOM/Canvas effects; text layout, cursor blinking, scrolling need JS/TypeScript
- **Real-time input handling** - Keystroke events need callback/event model
- **Cursor/selection manipulation** - Requires mutable position state with rapid updates
- **Large file editing** - For-loop iteration over 100k+ lines would hit cost limits

**Recommended architecture:**

```
┌──────────────────────────────────────────────────────┐
│  TypeScript (Electron renderer)                       │
│  - DOM rendering, event handling, virtualized list    │
│  - Calls Covenant WASM for document operations        │
└──────────────────────────────────────────────────────┘
          │                           ▲
          ▼                           │
┌──────────────────────────────────────────────────────┐
│  Covenant WASM                                        │
│  - Document model (immutable rope/piece table)        │
│  - Command execution                                  │
│  - Undo/redo stack                                    │
│  - Syntax analysis                                    │
│  - Returns: new document state, render instructions   │
└──────────────────────────────────────────────────────┘
```

Covenant serves as the **logic engine**, not the rendering layer.

---

## Use Case 5: Covenant Compiler (Self-Hosting)

**Goal:** Implement the Covenant compiler in Covenant itself.

### Verdict: THEORETICALLY POSSIBLE but impractical

**What works:**
- **Recursive descent parsing** - Full recursion support
- **AST representation** - Structs and enums for nodes
- **Pattern matching** - `match` on AST node kinds for visitors
- **Type checking** - Covenant's type system can express its own types
- **Symbol graph construction** - Query system is designed for this
- **Effect propagation** - Can implement I2 invariant validation

**Critical limitations:**

| Limitation | Impact |
|------------|--------|
| No mutable symbol tables | Each compiler pass produces new immutable structures (memory inefficient) |
| No while loops | Iterative algorithms (fixed-point analysis, optimization passes) need workarounds |
| No bit operations | WASM binary encoding needs bit-level control |
| Cost constraints | Compiling 10k+ snippet codebases would hit query budgets |
| No raw byte output | WASM emission requires byte-level manipulation |

**The WASM emission problem is fatal:**

WASM binary format requires:
- LEB128 encoding (variable-length integers with bit manipulation)
- Byte-level section headers
- Instruction opcodes as raw bytes

Without bitwise operations and raw byte manipulation, Covenant cannot emit valid WASM binaries.

**Possible workaround:** Emit WAT (WebAssembly Text format) as strings, then use an external tool to convert WAT → WASM. This adds an external dependency and isn't true self-hosting.

---

## Summary

| Use Case | Feasibility | Key Blocker |
|----------|-------------|-------------|
| VS Code extension + OpenCode | Feasible | Event model needs adapter |
| DOOM in browser | Not feasible | Mutable state, bit ops, game loops |
| DOOM in Electron | Not feasible | Same as browser |
| Text editor (Electron) | Partial | Covenant as logic engine only |
| Self-hosting compiler | Theoretical | WASM emission needs bit ops |

---

## Design Implications

If these use cases become goals, Covenant would need:

### For games/real-time applications:
- `while` loops or unbounded iteration
- Mutable arrays and data structures
- Bitwise operations (`band`, `bor`, `bxor`, `bshl`, `bshr`)
- Raw memory access primitives

### For UI applications:
- Event/callback binding model
- DOM/Canvas effect kinds
- Mutable state for UI components

### For self-hosting:
- Byte-level output primitives
- Or: WAT text generation as alternative

**Trade-off consideration:** These additions would fundamentally change Covenant's character as a "machine-first, deterministic, query-based" language. The current design optimizes for LLM code generation and navigation—not traditional systems programming. Any extensions should be weighed against this core mission.
