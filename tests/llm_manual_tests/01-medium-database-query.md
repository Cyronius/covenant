# LLM Manual Test: Database Query Function

## System Instructions

You are a code generator for Covenant, a machine-first programming language. Generate ONLY valid Covenant syntax.

### CRITICAL RULES

1. **NO OPERATORS** - Use keywords only:
   - Instead of `x + y` → `op=add input var="x" input var="y"`
   - Instead of `x == y` → `op=equals`
   - Instead of `x && y` → `op=and`
   - Instead of `!x` → `op=not`

2. **SSA FORM** - One operation per step, every step needs `as="output_name"` (use `as="_"` for discarded values, not `as="*"`)

3. **CANONICAL SECTION ORDER** (must follow exactly):
   effects → requires → types → signature → body → tests → metadata

4. **EVERY BLOCK NEEDS `end`** - snippet, signature, body, step, if, match, etc.

### SNIPPET TEMPLATE

```
snippet id="module.function_name" kind="fn"

effects
  effect database
  effect network
end

signature
  fn name="function_name"
    param name="x" type="Int"
    param name="y" type="String"
    returns union
      type="Result"
      type="Error"
    end
  end
end

body
  step id="s1" kind="compute"
    op=add
    input var="x"
    input lit=5
    as="sum"
  end
  step id="s2" kind="return"
    from="sum"
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
  op=equals                 // equals, not_equals, less, greater, less_eq, greater_eq
  input var="a"
  input var="b"
  as="is_equal"
end

step id="s3" kind="compute"
  op=and                    // and, or, not
  input var="flag1"
  input var="flag2"
  as="both_true"
end
```

#### call (function invocation)
```
step id="s1" kind="call"
  fn="validate_email"
  arg name="email" from="user_email"
  arg name="strict" lit=true
  as="is_valid"
end
```

#### bind (variable binding)
```
step id="s1" kind="bind"
  from="some_value"
  as="bound_var"
end

step id="s2" kind="bind"
  lit=42
  as="answer"
end
```

#### return
```
step id="s1" kind="return"
  from="result"
  as="_"
end

step id="s2" kind="return"
  struct type="User"
    field name="id" from="user_id"
    field name="email" from="email"
  end
  as="_"
end

step id="s3" kind="return"
  variant type="Error::NotFound"
  end
  as="_"
end
```

#### if (conditional)
```
step id="s1" kind="if"
  condition="is_valid"
  then
    step id="s1a" kind="return"
      lit=true
      as="_"
    end
  end
  else
    step id="s1b" kind="return"
      lit=false
      as="_"
    end
  end
  as="_"
end
```

#### match (pattern matching)
```
step id="s1" kind="match"
  on="result"
  case variant type="Some" bindings=("value")
    step id="s1a" kind="bind"
      from="value"
      as="unwrapped"
    end
  end
  case variant type="None"
    step id="s1b" kind="return"
      variant type="Error::NotFound"
      end
      as="_"
    end
  end
  as="_"
end
```

#### query (database)
```
step id="s1" kind="query"
  target="app_db"
  select all
  from="users"
  where
    equals field="id" var="user_id"
  end
  limit=1
  as="result"
end
```

#### insert
```
step id="s1" kind="insert"
  into="app_db.users"
  set field="name" from="name"
  set field="email" from="email"
  as="new_user"
end
```

### TYPES
- Basic: `Int`, `String`, `Bool`, `Float`, `DateTime`
- Optional: `type="User" optional`
- Collection: `collection of="User"`
- Union: `returns union type="Success" type="Error" end`

### EFFECTS
Functions with side effects MUST declare them:
```
effects
  effect database
  effect network
  effect filesystem
end
```
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

### EXAMPLE: Database Query with Error Handling

```
snippet id="db.get_user_by_id" kind="fn"

effects
  effect database
end

signature
  fn name="get_user_by_id"
    param name="id" type="Int"
    returns union
      type="User" optional
      type="DbError"
    end
  end
end

body
  step id="s1" kind="query"
    target="app_db"
    select all
    from="users"
    where
      equals field="id" var="id"
    end
    limit=1
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

Generate a Covenant function called `get_active_users_by_role` that:

1. Takes two parameters: `role` (String) and `min_login_count` (Int)
2. Queries the database for users where:
   - status equals "active"
   - role matches the input role
   - login_count is greater than min_login_count
3. Returns a collection of User or a DbError

Include:
- The User struct definition
- The DbError enum with variants: ConnectionFailed, QueryFailed, Timeout
- The function with proper effects declared

---

## Expected Output

```
snippet id="users.User" kind="struct"

signature
  struct name="User"
    field name="id" type="Int"
    field name="name" type="String"
    field name="email" type="String"
    field name="role" type="String"
    field name="status" type="String"
    field name="login_count" type="Int"
  end
end

end


snippet id="users.DbError" kind="enum"

signature
  enum name="DbError"
    variant name="ConnectionFailed"
      field name="message" type="String"
    end
    variant name="QueryFailed"
      field name="message" type="String"
    end
    variant name="Timeout"
    end
  end
end

end


snippet id="users.get_active_users_by_role" kind="fn"

effects
  effect database
end

signature
  fn name="get_active_users_by_role"
    param name="role" type="String"
    param name="min_login_count" type="Int"
    returns union
      collection of="User"
      type="DbError"
    end
  end
end

body
  step id="s1" kind="query"
    target="app_db"
    select all
    from="users"
    where
      and
        equals field="status" lit="active"
        equals field="role" var="role"
        greater field="login_count" var="min_login_count"
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

## Validation Checklist

| # | Criterion | Pass/Fail |
|---|-----------|-----------|
| 1 | Uses `snippet ... end` structure | |
| 2 | No operators (`+`, `==`, `&&`) - only keywords | |
| 3 | Every step has `as="..."` | |
| 4 | Sections in correct order (effects → signature → body) | |
| 5 | All blocks properly closed with `end` | |
| 6 | Effects section present for database function | |
| 7 | Query uses keyword syntax (`equals field=...`) | |
| 8 | Step IDs are unique (`s1`, `s2`, etc.) | |
| 9 | Return type matches signature | |

**Scoring:** 9/9 = perfect, 7-8 = minor issues, 5-6 = needs better examples, <5 = concept needs work
