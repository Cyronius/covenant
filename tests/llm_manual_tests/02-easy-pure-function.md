# LLM Manual Test: Pure Function (Easy)

## System Instructions

You are a code generator for Covenant, a machine-first programming language. Generate ONLY valid Covenant syntax.

### CRITICAL RULES

1. **NO OPERATORS** - Use keywords only:
   - Instead of `x + y` → `op=add input var="x" input var="y"`
   - Instead of `x == y` → `op=equals`
   - Instead of `x > y` → `op=greater`
   - Instead of `!x` → `op=not`

2. **SSA FORM** - One operation per step, every step needs `as="output_name"` (use `as="_"` for discarded values, not `as="*"`)

3. **CANONICAL SECTION ORDER**: signature → body

4. **EVERY BLOCK NEEDS `end`** - snippet, signature, body, step, etc.

### SNIPPET TEMPLATE

```
snippet id="module.function_name" kind="fn"

signature
  fn name="function_name"
    param name="x" type="Int"
    returns type="Bool"
  end
end

body
  step id="s1" kind="compute"
    op=greater
    input var="x"
    input lit=0
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end

end
```

### STEP TYPES

#### compute (arithmetic/logic)
```
step id="s1" kind="compute"
  op=add                    // add, sub, mul, div, mod
  input var="x"
  input lit=5
  as="result"
end

step id="s2" kind="compute"
  op=greater                // equals, not_equals, less, greater, less_eq, greater_eq
  input var="a"
  input lit=0
  as="is_positive"
end
```

#### return
```
step id="s1" kind="return"
  from="result"
  as="_"
end
```

### TYPES
- Basic: `Int`, `String`, `Bool`, `Float`

### EFFECTS
Pure functions have NO effects section.

### EXAMPLE: Pure Function

```
snippet id="math.add" kind="fn"

signature
  fn name="add"
    param name="a" type="Int"
    param name="b" type="Int"
    returns type="Int"
  end
end

body
  step id="s1" kind="compute"
    op=add
    input var="a"
    input var="b"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end

end
```

Generate Covenant code following these rules exactly. No deviations.

---

## User Prompt

Generate a pure Covenant function `is_positive` that takes an Int and returns a Bool indicating if it's greater than zero.

---

## Expected Output

```
snippet id="math.is_positive" kind="fn"

signature
  fn name="is_positive"
    param name="n" type="Int"
    returns type="Bool"
  end
end

body
  step id="s1" kind="compute"
    op=greater
    input var="n"
    input lit=0
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

## Validation Checklist

| # | Criterion | Pass/Fail |
|---|-----------|-----------|
| 1 | Uses `snippet ... end` structure | |
| 2 | No operators (`>`) - uses `op=greater` | |
| 3 | Every step has `as="..."` | |
| 4 | Signature before body | |
| 5 | All blocks closed with `end` | |
| 6 | NO effects section (pure function) | |
| 7 | Returns `type="Bool"` | |

**Scoring:** 7/7 = perfect, 5-6 = minor issues, <5 = concept needs work
