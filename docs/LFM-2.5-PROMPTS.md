# LFM-2.5 Prompts for Covenant Code Generation

Test prompts for evaluating small local LLMs (LFM2.5-1.2B-Instruct, Llama 3.2 1B) on Covenant syntax generation.

Each prompt includes the complete grammar and sufficient context to generate the target example.

## Prompt Design Principles

Based on testing, these techniques improve accuracy:

1. **Explicit `end` tracking** - Add a rule stating every block requires matching `end`
2. **Template skeleton** - Show the nesting structure before the task
3. **One snippet per prompt** - Reduce context accumulation errors
4. **Explicit return step rule** - Functions must end with explicit return
5. **`as="..."` syntax reminder** - Always use equals sign, not space

---

## Prompt 1: Hello World

**Target**: Simple effectful function with console output

```
You are a code generator for Covenant, a machine-first programming language. Generate valid Covenant code following the grammar and rules below.

=== COVENANT GRAMMAR (EBNF) ===

(* Top Level *)
program        = { snippet } ;
snippet        = "snippet" "id" "=" STRING "kind" "=" snippet_kind { section } "end" ;
snippet_kind   = "fn" | "struct" | "enum" | "module" | "database" | "extern" | "test" | "data" ;

(* Sections - must appear in this canonical order *)
section        = effects_section | requires_section | signature_section | body_section | tests_section | metadata_section ;

(* Effects Section *)
effects_section = "effects" { effect_decl } "end" ;
effect_decl     = "effect" IDENT ;

(* Signature Section *)
signature_section = "signature" fn_signature "end" ;
fn_signature   = "fn" "name" "=" STRING { param_decl } [ returns_decl ] "end" ;
param_decl     = "param" "name" "=" STRING "type" "=" type_ref ;
returns_decl   = "returns" ( "type" "=" type_ref | "collection" "of" "=" type_ref | "union" { union_member } "end" ) ;
union_member   = "type" "=" type_ref [ "optional" ] ;

(* Body Section - SSA form, one operation per step *)
body_section   = "body" { step } "end" ;
step           = "step" "id" "=" STRING "kind" "=" step_kind step_body "as" "=" STRING "end" ;
step_kind      = "compute" | "call" | "query" | "bind" | "return" | "if" | "match" | "for" ;

(* Call Step *)
call_body      = "fn" "=" STRING { call_arg } ;
call_arg       = "arg" "name" "=" STRING ( "from" "=" STRING | "lit" "=" literal ) ;

(* Return Step *)
return_body    = "from" "=" STRING | "lit" "=" literal ;

(* Types and Literals *)
type_ref       = IDENT [ "?" | "[]" ] ;
literal        = NUMBER | STRING | "true" | "false" | "none" ;
STRING         = '"' { any_char } '"' ;

=== KEY RULES ===

1. Every snippet starts with `snippet id="..." kind="..."` and ends with `end`
2. Sections must appear in canonical order: effects, requires, signature, body, tests, metadata
3. Every step has: id, kind, body content, and `as="..."` for output binding
4. Use `as="_"` when discarding the return value (like void/Unit)
5. Function calls use fully-qualified snippet IDs: `fn="module.function_name"`
6. Effects must be declared before using effectful functions
7. No operators - use keywords (add, equals, and, or, not)
8. Double quotes only for strings

=== TASK ===

Generate a Covenant function that:
- Has snippet id "main.hello" and kind "fn"
- Declares the "console" effect (required for printing)
- Has a signature with function name "main", no parameters, returns type "Unit"
- Body calls "console.println" with argument name="message" and literal value "Hello, world!"
- Discards the return value with as="_"

Generate only the Covenant code, no explanations.
```

**Expected Output**:
```covenant
snippet id="main.hello" kind="fn"

effects
  effect console
end

signature
  fn name="main"
    returns type="Unit"
  end
end

body
  step id="s1" kind="call"
    fn="console.println"
    arg name="message" lit="Hello, world!"
    as="_"
  end
end

end
```

---

## Prompt 2: Project Queries

**Target**: Meta-programming functions that query the AST/symbol graph

```
You are a code generator for Covenant, a machine-first programming language. Generate valid Covenant code following the grammar and rules below.

=== COVENANT GRAMMAR (EBNF) ===

(* Top Level *)
program        = { snippet } ;
snippet        = "snippet" "id" "=" STRING "kind" "=" snippet_kind { section } "end" ;
snippet_kind   = "fn" | "struct" | "enum" | "module" | "database" | "extern" | "test" | "data" ;

(* Sections - canonical order *)
section        = effects_section | requires_section | signature_section | body_section | tests_section | metadata_section ;

(* Effects Section *)
effects_section = "effects" { effect_decl } "end" ;
effect_decl     = "effect" IDENT ;

(* Signature Section *)
signature_section = "signature" fn_signature "end" ;
fn_signature   = "fn" "name" "=" STRING { param_decl } [ returns_decl ] "end" ;
param_decl     = "param" "name" "=" STRING "type" "=" type_ref ;
returns_decl   = "returns" ( "type" "=" type_ref | "collection" "of" "=" type_ref | "union" { union_member } "end" ) ;

(* Body Section *)
body_section   = "body" { step } "end" ;
step           = "step" "id" "=" STRING "kind" "=" step_kind step_body "as" "=" STRING "end" ;
step_kind      = "compute" | "call" | "query" | "bind" | "return" | "if" | "match" | "for" ;

(* Query Step - Covenant dialect for querying project/AST *)
query_body     = "target" "=" STRING select_clause from_clause [ where_clause ] [ order_clause ] [ limit_clause ] ;
select_clause  = "select" ( "all" | { "field" "=" STRING } ) ;
from_clause    = "from" "=" STRING ;
where_clause   = "where" condition "end" ;
condition      = simple_condition | compound_condition ;
simple_condition = compare_op "field" "=" STRING ( "var" "=" STRING | "lit" "=" literal ) ;
compare_op     = "equals" | "not_equals" | "less" | "greater" | "contains" | "matches" ;
compound_condition = ( "and" | "or" ) { condition } "end" ;
order_clause   = "order" "by" "=" STRING "dir" "=" ( "asc" | "desc" ) ;
limit_clause   = "limit" "=" NUMBER ;

(* Return Step *)
return_body    = "from" "=" STRING ;

(* Types *)
type_ref       = IDENT [ "?" | "[]" ] ;
literal        = NUMBER | STRING | "true" | "false" | "none" | "[" [ literal { "," literal } ] "]" ;

=== KEY RULES ===

1. Every snippet: `snippet id="..." kind="..."` ... `end`
2. Sections in order: effects, requires, signature, body, tests, metadata
3. Every step has: id, kind, body, `as="..."` binding
4. The "meta" effect allows querying the project's symbol graph
5. target="project" queries the current project's AST
6. from="functions" queries function symbols, from="requirements" queries requirements, etc.
7. Compound conditions use `and ... end` or `or ... end` with nested conditions
8. Array literals use brackets: lit=[] for empty, lit=["a", "b"] for values
9. Use "contains" for checking if a field contains a value
10. Use "matches" for regex pattern matching on string fields

=== TASK ===

Generate THREE Covenant functions for querying the project's symbol graph:

FUNCTION 1: meta.find_db_functions
- Effect: meta
- Returns: FunctionInfo[]
- Query target="project", select all from="functions"
- Where: contains field="effects" lit="database"
- Return the result

FUNCTION 2: meta.find_callers
- Effect: meta
- Parameter: fn_name of type String
- Returns: FunctionInfo[]
- Query target="project", select field="called_by" from="functions"
- Where: equals field="name" var="fn_name"
- Return the result

FUNCTION 3: meta.find_dead_code
- Effect: meta
- Returns: FunctionInfo[]
- Query target="project", select all from="functions"
- Where: compound AND condition with THREE clauses:
  - equals field="called_by" lit=[]
  - equals field="is_exported" lit=false
  - equals field="is_entry_point" lit=false
- Return the result

Generate only the Covenant code for all three functions, no explanations.
```

**Expected Output**:
```covenant
snippet id="meta.find_db_functions" kind="fn"

effects
  effect meta
end

signature
  fn name="find_db_functions"
    returns collection of="FunctionInfo"
  end
end

body
  step id="s1" kind="query"
    target="project"
    select all
    from="functions"
    where
      contains field="effects" lit="database"
    end
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end

end


snippet id="meta.find_callers" kind="fn"

effects
  effect meta
end

signature
  fn name="find_callers"
    param name="fn_name" type="String"
    returns collection of="FunctionInfo"
  end
end

body
  step id="s1" kind="query"
    target="project"
    select field="called_by"
    from="functions"
    where
      equals field="name" var="fn_name"
    end
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end

end


snippet id="meta.find_dead_code" kind="fn"

effects
  effect meta
end

signature
  fn name="find_dead_code"
    returns collection of="FunctionInfo"
  end
end

body
  step id="s1" kind="query"
    target="project"
    select all
    from="functions"
    where
      and
        equals field="called_by" lit=[]
        equals field="is_exported" lit=false
        equals field="is_entry_point" lit=false
      end
    end
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end

end
```

---

## Prompt 3: Advanced SQL

**Target**: Complex SQL queries with CTEs, window functions, and dialect-specific syntax

```
You are a code generator for Covenant, a machine-first programming language. Generate valid Covenant code following the grammar and rules below.

=== COVENANT GRAMMAR (EBNF) ===

(* Top Level *)
program        = { snippet } ;
snippet        = "snippet" "id" "=" STRING "kind" "=" snippet_kind { section } "end" ;
snippet_kind   = "fn" | "struct" | "enum" | "module" | "database" | "extern" | "test" | "data" ;

(* Sections - canonical order *)
section        = effects_section | requires_section | signature_section | body_section | tests_section | metadata_section ;

(* Effects Section *)
effects_section = "effects" { effect_decl } "end" ;
effect_decl     = "effect" IDENT ;

(* Requirements Section *)
requires_section = "requires" { requirement } "end" ;
requirement      = "req" "id" "=" STRING { req_field } "end" ;
req_field        = "text" STRING | "priority" ( "critical" | "high" | "medium" | "low" ) ;

(* Signature Section *)
signature_section = "signature" fn_signature "end" ;
fn_signature   = "fn" "name" "=" STRING { param_decl } [ returns_decl ] "end" ;
param_decl     = "param" "name" "=" STRING "type" "=" type_ref ;
returns_decl   = "returns" ( "type" "=" type_ref | "collection" "of" "=" type_ref | "union" { union_member } "end" ) ;

(* Body Section *)
body_section   = "body" { step } "end" ;
step           = "step" "id" "=" STRING "kind" "=" step_kind step_body "as" "=" STRING "end" ;
step_kind      = "compute" | "call" | "query" | "bind" | "return" ;

(* Query Step with SQL Dialect *)
query_body     = [ dialect_clause ] "target" "=" STRING query_content ;
dialect_clause = "dialect" "=" STRING ;

(* SQL Dialect Query - raw SQL in body block *)
dialect_query_content = "body" RAW_SQL "end" [ params_section ] "returns" return_type_spec ;
params_section = "params" { param_binding } "end" ;
param_binding  = "param" "name" "=" STRING "from" "=" STRING ;
return_type_spec = "type" "=" type_ref | "collection" "of" "=" type_ref ;

(* Return Step *)
return_body    = "from" "=" STRING ;

(* Tests Section *)
tests_section  = "tests" { test_def } "end" ;
test_def       = "test" "id" "=" STRING "kind" "=" test_kind [ "covers" "=" STRING ] "end" ;
test_kind      = "unit" | "property" | "integration" ;

(* Types *)
type_ref       = IDENT [ "?" | "[]" ] ;

=== KEY RULES ===

1. Every snippet: `snippet id="..." kind="..."` ... `end`
2. Sections in canonical order: effects, requires, signature, body, tests, metadata
3. Every step has: id, kind, body, `as="..."` binding
4. SQL dialect queries use `dialect="postgres"` or `dialect="sqlserver"`
5. Raw SQL goes between `body` and `end` - the compiler does NOT parse this SQL
6. PostgreSQL uses :param_name for parameters
7. SQL Server uses @param_name for parameters
8. params section maps Covenant variables to SQL placeholders
9. returns clause is REQUIRED for dialect queries to specify the result type
10. The "database" effect is required for all database operations

=== TASK ===

Generate TWO Covenant functions with advanced SQL:

FUNCTION 1: analytics.high_value_customers (PostgreSQL)
- Effect: database
- Requirement: id="R-ANALYTICS-001", text="Identify customers with total spending above threshold", priority=high
- Parameters: min_revenue (Decimal), min_orders (Int)
- Returns: collection of CustomerStats
- Target: app_db
- SQL body with CTE:
  ```sql
  WITH customer_orders AS (
    SELECT
      customer_id,
      COUNT(*) as order_count,
      SUM(total) as total_revenue,
      AVG(total) as avg_order_value,
      ARRAY_AGG(DISTINCT product_id) as products
    FROM orders
    GROUP BY customer_id
  )
  SELECT
    c.id, c.email, c.name,
    co.order_count, co.total_revenue, co.avg_order_value, co.products
  FROM customer_orders co
  JOIN customers c ON c.id = co.customer_id
  WHERE co.total_revenue > :min_revenue AND co.order_count >= :min_orders
  ORDER BY co.total_revenue DESC
  ```
- Params: map min_revenue and min_orders
- Include a test covering the requirement

FUNCTION 2: analytics.sales_with_metrics (SQL Server)
- Effect: database
- Parameter: user_id (Int)
- Returns: collection of SalesMetrics
- Target: analytics_db
- SQL body with window functions:
  ```sql
  SELECT
    order_id, order_date, amount,
    ROW_NUMBER() OVER (ORDER BY order_date) as order_sequence,
    RANK() OVER (ORDER BY amount DESC) as amount_rank,
    SUM(amount) OVER (ORDER BY order_date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as running_total,
    AVG(amount) OVER (ORDER BY order_date ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) as moving_avg_3,
    LAG(amount, 1) OVER (ORDER BY order_date) as prev_amount,
    LEAD(amount, 1) OVER (ORDER BY order_date) as next_amount
  FROM orders
  WHERE user_id = @user_id
  ORDER BY order_date
  ```
- Params: map user_id

Generate only the Covenant code for both functions, no explanations.
```

**Expected Output**:
```covenant
snippet id="analytics.high_value_customers" kind="fn"

effects
  effect database
end

requires
  req id="R-ANALYTICS-001"
    text "Identify customers with total spending above threshold"
    priority high
  end
end

signature
  fn name="get_high_value_customers"
    param name="min_revenue" type="Decimal"
    param name="min_orders" type="Int"
    returns collection of="CustomerStats"
  end
end

body
  step id="s1" kind="query"
    dialect="postgres"
    target="app_db"
    body
      WITH customer_orders AS (
        SELECT
          customer_id,
          COUNT(*) as order_count,
          SUM(total) as total_revenue,
          AVG(total) as avg_order_value,
          ARRAY_AGG(DISTINCT product_id) as products
        FROM orders
        GROUP BY customer_id
      )
      SELECT
        c.id,
        c.email,
        c.name,
        co.order_count,
        co.total_revenue,
        co.avg_order_value,
        co.products
      FROM customer_orders co
      JOIN customers c ON c.id = co.customer_id
      WHERE co.total_revenue > :min_revenue
        AND co.order_count >= :min_orders
      ORDER BY co.total_revenue DESC
    end
    params
      param name="min_revenue" from="min_revenue"
      param name="min_orders" from="min_orders"
    end
    returns collection of="CustomerStats"
    as="result"
  end

  step id="s2" kind="return"
    from="result"
    as="_"
  end
end

tests
  test id="T-ANALYTICS-001" kind="unit" covers="R-ANALYTICS-001"
  end
end

end


snippet id="analytics.sales_with_metrics" kind="fn"

effects
  effect database
end

signature
  fn name="calculate_sales_metrics"
    param name="user_id" type="Int"
    returns collection of="SalesMetrics"
  end
end

body
  step id="s1" kind="query"
    dialect="sqlserver"
    target="analytics_db"
    body
      SELECT
        order_id,
        order_date,
        amount,
        ROW_NUMBER() OVER (ORDER BY order_date) as order_sequence,
        RANK() OVER (ORDER BY amount DESC) as amount_rank,
        SUM(amount) OVER (
          ORDER BY order_date
          ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
        ) as running_total,
        AVG(amount) OVER (
          ORDER BY order_date
          ROWS BETWEEN 2 PRECEDING AND CURRENT ROW
        ) as moving_avg_3,
        LAG(amount, 1) OVER (ORDER BY order_date) as prev_amount,
        LEAD(amount, 1) OVER (ORDER BY order_date) as next_amount
      FROM orders
      WHERE user_id = @user_id
      ORDER BY order_date
    end
    params
      param name="user_id" from="user_id"
    end
    returns collection of="SalesMetrics"
    as="result"
  end

  step id="s2" kind="return"
    from="result"
    as="_"
  end
end

end
```

---

## Prompt 4: Pattern Matching

**Target**: Enum definition and functions using match expressions with variant patterns and bindings

```
You are a code generator for Covenant, a machine-first programming language. Generate valid Covenant code following the grammar and rules below.

=== COVENANT GRAMMAR (EBNF) ===

(* Top Level *)
program        = { snippet } ;
snippet        = "snippet" "id" "=" STRING "kind" "=" snippet_kind { section } "end" ;
snippet_kind   = "fn" | "struct" | "enum" | "module" | "database" | "extern" | "test" | "data" ;

(* Sections - canonical order *)
section        = effects_section | requires_section | signature_section | body_section | tests_section | metadata_section ;

(* Signature Section - for enums *)
signature_section = "signature" ( fn_signature | enum_signature ) "end" ;

enum_signature = "enum" "name" "=" STRING { enum_variant } "end" ;
enum_variant   = "variant" "name" "=" STRING [ variant_fields ] "end" ;
variant_fields = { "field" "name" "=" STRING "type" "=" type_ref } ;

(* Signature Section - for functions *)
fn_signature   = "fn" "name" "=" STRING { param_decl } [ returns_decl ] "end" ;
param_decl     = "param" "name" "=" STRING "type" "=" type_ref ;
returns_decl   = "returns" "type" "=" type_ref [ "optional" ] ;

(* Body Section *)
body_section   = "body" { step } "end" ;
step           = "step" "id" "=" STRING "kind" "=" step_kind step_body "as" "=" STRING "end" ;
step_kind      = "compute" | "call" | "query" | "bind" | "return" | "if" | "match" | "for" ;

(* Match Step - pattern matching on values *)
match_body     = "on" "=" STRING { match_case } ;
match_case     = "case" pattern { step } "end" ;
pattern        = variant_pattern | wildcard_pattern ;
variant_pattern = "variant" "type" "=" STRING [ "bindings" "=" "(" { STRING } ")" ] ;
wildcard_pattern = "wildcard" ;

(* Call Step *)
call_body      = "fn" "=" STRING { call_arg } ;
call_arg       = "arg" "name" "=" STRING "from" "=" STRING ;

(* Return Step *)
return_body    = "from" "=" STRING | "lit" "=" literal ;

(* Types and Literals *)
type_ref       = IDENT [ "::" IDENT ] [ "?" | "[]" | "<" type_list ">" ] ;
type_list      = type_ref { "," type_ref } ;
literal        = NUMBER | STRING | "true" | "false" | "none" ;

=== KEY RULES ===

1. Every snippet: `snippet id="..." kind="..."` ... `end`
2. Sections in canonical order: effects, requires, signature, body, tests, metadata
3. Every step has: id, kind, body content, and `as="..."` for output binding
4. Enum snippets use kind="enum" and have signature with enum definition
5. Variant patterns use `variant type="EnumName::VariantName"`
6. To extract data from a variant, use `bindings=("var1", "var2")` - variables are bound in order
7. Use `wildcard` for catch-all/default cases
8. Match steps contain nested steps inside each case
9. Steps inside match cases use sub-IDs like "s1a", "s1b", etc.
10. The match step itself needs `as="_"` at the end (after all cases)
11. Return `lit=none` for returning the none/null value
12. Optional return types use `returns type="T" optional`

=== TASK ===

Generate THREE Covenant snippets for JSON handling with pattern matching:

SNIPPET 1: json.Json (enum definition)
- kind="enum"
- Define enum "Json" with 6 variants:
  - Null (no fields)
  - Bool (field: value of type Bool)
  - Number (field: value of type Float)
  - String (field: value of type String)
  - Array (field: items of type Json[])
  - Object (field: fields of type Map<String, Json>)

SNIPPET 2: json.type_name (function)
- kind="fn"
- Parameter: value of type Json
- Returns: String
- Match on "value" with 6 cases for each Json variant:
  - Json::Null -> return lit="null"
  - Json::Bool -> return lit="boolean"
  - Json::Number -> return lit="number"
  - Json::String -> return lit="string"
  - Json::Array -> return lit="array"
  - Json::Object -> return lit="object"
- Use step IDs: s1 for match, s1a/s1b/s1c/s1d/s1e/s1f for returns inside cases

SNIPPET 3: json.get_string (function)
- kind="fn"
- Parameter: value of type Json
- Returns: String optional
- Match on "value" with 2 cases:
  - Json::String with bindings=("s") -> return from="s"
  - wildcard -> return lit=none
- Use step IDs: s1 for match, s1a for first return, s1b for second return

Generate only the Covenant code for all three snippets, no explanations.
```

**Expected Output**:
```covenant
snippet id="json.Json" kind="enum"

signature
  enum name="Json"
    variant name="Null"
    end
    variant name="Bool"
      field name="value" type="Bool"
    end
    variant name="Number"
      field name="value" type="Float"
    end
    variant name="String"
      field name="value" type="String"
    end
    variant name="Array"
      field name="items" type="Json[]"
    end
    variant name="Object"
      field name="fields" type="Map<String, Json>"
    end
  end
end

end


snippet id="json.type_name" kind="fn"

signature
  fn name="json_type_name"
    param name="value" type="Json"
    returns type="String"
  end
end

body
  step id="s1" kind="match"
    on="value"
    case variant type="Json::Null"
      step id="s1a" kind="return"
        lit="null"
        as="_"
      end
    end
    case variant type="Json::Bool"
      step id="s1b" kind="return"
        lit="boolean"
        as="_"
      end
    end
    case variant type="Json::Number"
      step id="s1c" kind="return"
        lit="number"
        as="_"
      end
    end
    case variant type="Json::String"
      step id="s1d" kind="return"
        lit="string"
        as="_"
      end
    end
    case variant type="Json::Array"
      step id="s1e" kind="return"
        lit="array"
        as="_"
      end
    end
    case variant type="Json::Object"
      step id="s1f" kind="return"
        lit="object"
        as="_"
      end
    end
    as="_"
  end
end

end


snippet id="json.get_string" kind="fn"

signature
  fn name="get_string"
    param name="value" type="Json"
    returns type="String" optional
  end
end

body
  step id="s1" kind="match"
    on="value"
    case variant type="Json::String" bindings=("s")
      step id="s1a" kind="return"
        from="s"
        as="_"
      end
    end
    case wildcard
      step id="s1b" kind="return"
        lit=none
        as="_"
      end
    end
    as="_"
  end
end

end
```

---

## Token Estimates

| Prompt | Grammar | Instructions | Task | Total Input | Expected Output |
|--------|---------|--------------|------|-------------|-----------------|
| Hello World | ~800 | ~300 | ~150 | ~1,250 | ~150 |
| Project Queries | ~1,200 | ~400 | ~400 | ~2,000 | ~600 |
| Advanced SQL | ~1,400 | ~500 | ~800 | ~2,700 | ~1,200 |
| Pattern Matching | ~1,100 | ~500 | ~500 | ~2,100 | ~800 |

All prompts fit comfortably within a 32K context window, leaving ample room for generation.

## Usage Notes

1. **Grammar Subsetting**: Each prompt includes only the grammar rules relevant to that example. This reduces noise and focuses the model on applicable syntax.

2. **Key Rules Section**: Explicitly states the most important constraints to prevent common errors (canonical ordering, quoting, effect requirements).

3. **Structured Task**: The task description mirrors the target output structure, making it easier for the model to generate correct code.

4. **No Ambiguity**: All identifiers, types, and structure are specified exactly - no room for creative interpretation.

5. **Evaluation**: Compare generated output against expected output for:
   - Syntax correctness (parseable)
   - Structural match (same sections, steps)
   - Semantic equivalence (same behavior)
