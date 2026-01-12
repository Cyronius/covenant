# LLM Manual Test: Validation with Error Handling (Medium)

## System Instructions

You are a code generator for Covenant, a machine-first programming language. Generate ONLY valid Covenant syntax.

### CRITICAL RULES

1. **NO OPERATORS** - Use keywords only:
   - Instead of `x + y` → `op=add input var="x" input var="y"`
   - Instead of `x == y` → `op=equals`
   - Instead of `x && y` → `op=and`
   - `op=contains` for string/collection containment

2. **SSA FORM** - One operation per step, every step needs `as="output_name"` (use `as="_"` for discarded values, not `as="*"`)

3. **CANONICAL SECTION ORDER**: signature → body

4. **EVERY BLOCK NEEDS `end`** - snippet, signature, body, step, if, etc.

### SNIPPET TEMPLATE

```
snippet id="module.function_name" kind="fn"

signature
  fn name="function_name"
    param name="input" type="String"
    returns union
      type="String"
      type="ValidationError"
    end
  end
end

body
  step id="s1" kind="compute"
    op=contains
    input var="input"
    input lit="@"
    as="is_valid"
  end
  step id="s2" kind="if"
    condition="is_valid"
    then
      step id="s2a" kind="return"
        from="input"
        as="_"
      end
    end
    else
      step id="s2b" kind="return"
        variant type="ValidationError::InvalidFormat"
          field name="message" lit="Invalid"
        end
        as="_"
      end
    end
    as="_"
  end
end

end
```

### STEP TYPES

#### compute
```
step id="s1" kind="compute"
  op=contains
  input var="email"
  input lit="@"
  as="has_at"
end
```

#### if (conditional)
```
step id="s1" kind="if"
  condition="is_valid"
  then
    step id="s1a" kind="return"
      from="value"
      as="_"
    end
  end
  else
    step id="s1b" kind="return"
      variant type="Error::Invalid"
        field name="message" lit="error message"
      end
      as="_"
    end
  end
  as="_"
end
```

#### return with variant (enum)
```
step id="s1" kind="return"
  variant type="ValidationError::InvalidFormat"
    field name="message" lit="Missing @ symbol"
  end
  as="_"
end
```

### TYPES
- Basic: `Int`, `String`, `Bool`
- Union: `returns union type="Success" type="Error" end`

### EXAMPLE: Enum Definition

```
snippet id="validation.ValidationError" kind="enum"

signature
  enum name="ValidationError"
    variant name="InvalidFormat"
      field name="message" type="String"
    end
  end
end

end
```

Generate Covenant code following these rules exactly. No deviations.

---

## User Prompt

Generate a Covenant function `validate_email` that takes a String, checks if it contains "@", and returns either the email String or a ValidationError.

Include the ValidationError enum with a variant called InvalidFormat that has a message field.

---

## Expected Output

```
snippet id="validation.ValidationError" kind="enum"

signature
  enum name="ValidationError"
    variant name="InvalidFormat"
      field name="message" type="String"
    end
  end
end

end


snippet id="validation.validate_email" kind="fn"

signature
  fn name="validate_email"
    param name="email" type="String"
    returns union
      type="String"
      type="ValidationError"
    end
  end
end

body
  step id="s1" kind="compute"
    op=contains
    input var="email"
    input lit="@"
    as="has_at"
  end
  step id="s2" kind="if"
    condition="has_at"
    then
      step id="s2a" kind="return"
        from="email"
        as="_"
      end
    end
    else
      step id="s2b" kind="return"
        variant type="ValidationError::InvalidFormat"
          field name="message" lit="Email must contain @"
        end
        as="_"
      end
    end
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
| 2 | No operators - uses `op=contains` | |
| 3 | Every step has `as="..."` | |
| 4 | Union return type declared correctly | |
| 5 | All blocks closed with `end` | |
| 6 | If/then/else structure correct | |
| 7 | Variant return syntax correct | |
| 8 | Enum defined with variant and field | |

**Scoring:** 8/8 = perfect, 6-7 = minor issues, <6 = concept needs work
