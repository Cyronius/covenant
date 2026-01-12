# LLM Manual Tests

Manual smoke tests to validate whether frontier LLMs can generate valid Covenant code.

## How to Use

1. Open a chat with ChatGPT, Claude, Gemini, etc.
2. Copy the **System Instructions** section into the system prompt (or paste at the start of the conversation)
3. Send the **User Prompt** section as your message
4. Compare the output against **Expected Output**
5. Score using the **Validation Checklist**

## Test Files

| File | Difficulty | Tests |
|------|------------|-------|
| [01-medium-database-query.md](01-medium-database-query.md) | Medium | Database query, effects, structs, enums |
| [02-easy-pure-function.md](02-easy-pure-function.md) | Easy | Pure function, basic compute |
| [03-medium-validation-error.md](03-medium-validation-error.md) | Medium | Conditionals, error handling, union returns |
| [04-hard-transaction.md](04-hard-transaction.md) | Hard | Transactions, multiple queries, complex flow |

## What to Watch For

**Common failures:**
1. Using operators (`+`, `==`, `>`) instead of keywords (`op=add`, `op=equals`, `op=greater`)
2. Missing `as="..."` on steps
3. Missing `end` keywords
4. Wrong section order
5. Inventing syntax not in the spec

## Scoring Guide

- **Perfect**: LLM nails the syntax on first try
- **Minor issues**: Small syntax errors, easy to fix
- **Partial success**: Gets structure right, misses details
- **Significant gaps**: Fundamental misunderstanding of syntax

## Results Log

| Date | Model | Test | Score | Notes |
|------|-------|------|-------|-------|
| | | | | |
