#!/usr/bin/env bash
#
# Build script for query examples
#
# This script compiles the query example programs to WASM:
# - project-queries: Symbol graph queries
# - knowledge-base: Knowledge base traversal
# - query-system: Embedded queries, RAG, parameterized queries
# - symbol-metadata: Symbol metadata embedding
# - relation-traversal: Relation traversal
# - query-performance-benchmark: Performance benchmark

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Building Query Examples ===${NC}\n"

# Build the compiler first
echo -e "${YELLOW}Building Covenant compiler...${NC}"
cargo build --release -p covenant-cli
echo -e "${GREEN}✓ Compiler built${NC}\n"

COVENANT="./target/release/covenant"

# Helper function to compile an example
compile_example() {
    local dir=$1
    local name=$2
    local input="./examples/${dir}/${name}.cov"
    local output="./examples/${dir}/output/${name}.wasm"

    echo -e "${YELLOW}Compiling ${dir}/${name}${NC}"
    echo "  Input:  $input"
    echo "  Output: $output"

    if [ ! -f "$input" ]; then
        echo -e "${RED}  ✗ Input file not found${NC}"
        return 1
    fi

    # Ensure output directory exists
    mkdir -p "$(dirname "$output")"

    if $COVENANT compile "$input" --output "$output" --target deno; then
        local size=$(stat -c%s "$output" 2>/dev/null || stat -f%z "$output" 2>/dev/null)
        echo -e "${GREEN}  ✓ Compiled successfully (${size} bytes)${NC}\n"
    else
        echo -e "${RED}  ✗ Compilation failed${NC}\n"
        return 1
    fi
}

# Compile examples
compile_example "project-queries" "project-queries" || true
compile_example "knowledge-base" "knowledge-base" || true
compile_example "query-system" "embedded-query" || true
compile_example "query-system" "rag-query" || true
compile_example "query-system" "parameterized-query" || true
compile_example "symbol-metadata" "symbol-metadata" || true
compile_example "relation-traversal" "relation-traversal" || true
compile_example "query-performance-benchmark" "performance-benchmark" || true

echo -e "${GREEN}=== Build Complete ===${NC}\n"

# List generated files
echo "Generated WASM files:"
find ./examples -name "*.wasm" -exec ls -lh {} \; 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'  || echo "  (none)"
echo ""

echo "To run tests:"
echo "  cd examples/project-queries && deno run --allow-read test.ts"
echo "  cd examples/knowledge-base && deno run --allow-read test.ts"
echo "  cd examples/query-system && deno run --allow-read test-embedded.ts"
echo "  cd examples/query-system && deno run --allow-read test-rag.ts"
echo "  cd examples/query-system && deno run --allow-read test-parameterized.ts"
echo "  cd examples/symbol-metadata && deno run --allow-read test.ts"
echo "  cd examples/relation-traversal && deno run --allow-read test.ts"
echo "  cd examples/query-performance-benchmark && deno run --allow-read benchmark.ts"
