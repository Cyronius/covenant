# LLM Manual Test: Transaction with Multiple Queries (Hard)

## System Instructions

You are a code generator for Covenant, a machine-first programming language. Generate ONLY valid Covenant syntax.

### CRITICAL RULES

1. **NO OPERATORS** - Use keywords only:
   - Instead of `x + y` → `op=add input var="x" input var="y"`
   - Instead of `x - y` → `op=sub`
   - Instead of `x >= y` → `op=greater_eq`
   - Instead of `x < y` → `op=less`

2. **SSA FORM** - One operation per step, every step needs `as="output_name"` (use `as="_"` for discarded values, not `as="*"`)

3. **CANONICAL SECTION ORDER**: effects → signature → body

4. **EVERY BLOCK NEEDS `end`** - snippet, signature, body, step, if, match, transaction, etc.

### SNIPPET TEMPLATE

```
snippet id="module.function_name" kind="fn"

effects
  effect database
end

signature
  fn name="function_name"
    param name="id" type="Int"
    param name="amount" type="Int"
    returns union
      type="Result"
      type="Error"
    end
  end
end

body
  step id="s1" kind="query"
    target="app_db"
    select all
    from="accounts"
    where
      equals field="id" var="id"
    end
    limit=1
    as="account"
  end
  step id="s2" kind="return"
    from="account"
    as="_"
  end
end

end
```

### STEP TYPES

#### query
```
step id="s1" kind="query"
  target="app_db"
  select all
  from="accounts"
  where
    equals field="id" var="account_id"
  end
  limit=1
  as="account"
end
```

#### match (for optional/union unwrapping)
```
step id="s1" kind="match"
  on="query_result"
  case variant type="Some" bindings=("account")
    step id="s1a" kind="bind"
      from="account"
      as="found_account"
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

#### compute
```
step id="s1" kind="compute"
  op=less
  input var="balance"
  input var="amount"
  as="insufficient"
end

step id="s2" kind="compute"
  op=sub
  input var="balance"
  input var="amount"
  as="new_balance"
end
```

#### if
```
step id="s1" kind="if"
  condition="insufficient"
  then
    step id="s1a" kind="return"
      variant type="TransferError::InsufficientFunds"
      end
      as="_"
    end
  end
  as="_"
end
```

#### update
```
step id="s1" kind="update"
  target="app_db.accounts"
  set field="balance" from="new_balance"
  where
    equals field="id" var="account_id"
  end
  as="updated"
end
```

#### transaction
```
step id="s1" kind="transaction"
  step id="s1a" kind="update"
    target="app_db.accounts"
    set field="balance" from="sender_new_balance"
    where
      equals field="id" var="sender_id"
    end
    as="_"
  end
  step id="s1b" kind="update"
    target="app_db.accounts"
    set field="balance" from="recipient_new_balance"
    where
      equals field="id" var="recipient_id"
    end
    as="_"
  end
  as="tx_result"
end
```

### TYPES
- Basic: `Int`, `String`, `Bool`
- Struct fields: `field name="x" type="Int"`
- Union: `returns union type="Success" type="Error" end`

### EFFECTS
```
effects
  effect database
end
```

### EXAMPLE: Struct Definition

```
snippet id="banking.Account" kind="struct"

signature
  struct name="Account"
    field name="id" type="Int"
    field name="owner_name" type="String"
    field name="balance" type="Int"
  end
end

end
```

### EXAMPLE: Enum Definition

```
snippet id="banking.TransferError" kind="enum"

signature
  enum name="TransferError"
    variant name="InsufficientFunds"
    end
    variant name="AccountNotFound"
      field name="account_id" type="Int"
    end
  end
end

end
```

Generate Covenant code following these rules exactly. No deviations.

---

## User Prompt

Generate a Covenant function `transfer_funds` that:
- Takes sender_id (Int), recipient_id (Int), and amount (Int)
- Queries both accounts from the database
- Validates the sender has sufficient balance
- Updates both accounts (subtract from sender, add to recipient)
- Returns TransferResult (with new balances) or TransferError

Include:
- Account struct (id, owner_name, balance)
- TransferResult struct (sender_balance, recipient_balance)
- TransferError enum with variants: InsufficientFunds, AccountNotFound, TransactionFailed
- Use a transaction block for the updates

---

## Expected Output

```
snippet id="banking.Account" kind="struct"

signature
  struct name="Account"
    field name="id" type="Int"
    field name="owner_name" type="String"
    field name="balance" type="Int"
  end
end

end


snippet id="banking.TransferResult" kind="struct"

signature
  struct name="TransferResult"
    field name="sender_balance" type="Int"
    field name="recipient_balance" type="Int"
  end
end

end


snippet id="banking.TransferError" kind="enum"

signature
  enum name="TransferError"
    variant name="InsufficientFunds"
    end
    variant name="AccountNotFound"
      field name="account_id" type="Int"
    end
    variant name="TransactionFailed"
      field name="message" type="String"
    end
  end
end

end


snippet id="banking.transfer_funds" kind="fn"

effects
  effect database
end

signature
  fn name="transfer_funds"
    param name="sender_id" type="Int"
    param name="recipient_id" type="Int"
    param name="amount" type="Int"
    returns union
      type="TransferResult"
      type="TransferError"
    end
  end
end

body
  // Query sender account
  step id="s1" kind="query"
    target="app_db"
    select all
    from="accounts"
    where
      equals field="id" var="sender_id"
    end
    limit=1
    as="sender_result"
  end
  step id="s2" kind="match"
    on="sender_result"
    case variant type="Some" bindings=("account")
      step id="s2a" kind="bind"
        from="account"
        as="sender"
      end
    end
    case variant type="None"
      step id="s2b" kind="return"
        variant type="TransferError::AccountNotFound"
          field name="account_id" from="sender_id"
        end
        as="_"
      end
    end
    as="_"
  end

  // Query recipient account
  step id="s3" kind="query"
    target="app_db"
    select all
    from="accounts"
    where
      equals field="id" var="recipient_id"
    end
    limit=1
    as="recipient_result"
  end
  step id="s4" kind="match"
    on="recipient_result"
    case variant type="Some" bindings=("account")
      step id="s4a" kind="bind"
        from="account"
        as="recipient"
      end
    end
    case variant type="None"
      step id="s4b" kind="return"
        variant type="TransferError::AccountNotFound"
          field name="account_id" from="recipient_id"
        end
        as="_"
      end
    end
    as="_"
  end

  // Check sufficient balance
  step id="s5" kind="compute"
    op=less
    input var="sender.balance"
    input var="amount"
    as="insufficient"
  end
  step id="s6" kind="if"
    condition="insufficient"
    then
      step id="s6a" kind="return"
        variant type="TransferError::InsufficientFunds"
        end
        as="_"
      end
    end
    as="_"
  end

  // Calculate new balances
  step id="s7" kind="compute"
    op=sub
    input var="sender.balance"
    input var="amount"
    as="sender_new_balance"
  end
  step id="s8" kind="compute"
    op=add
    input var="recipient.balance"
    input var="amount"
    as="recipient_new_balance"
  end

  // Update both in transaction
  step id="s9" kind="transaction"
    step id="s9a" kind="update"
      target="app_db.accounts"
      set field="balance" from="sender_new_balance"
      where
        equals field="id" var="sender_id"
      end
      as="_"
    end
    step id="s9b" kind="update"
      target="app_db.accounts"
      set field="balance" from="recipient_new_balance"
      where
        equals field="id" var="recipient_id"
      end
      as="_"
    end
    as="_"
  end

  // Return result
  step id="s10" kind="return"
    struct type="TransferResult"
      field name="sender_balance" from="sender_new_balance"
      field name="recipient_balance" from="recipient_new_balance"
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
| 2 | No operators - uses `op=sub`, `op=add`, `op=less` | |
| 3 | Every step has `as="..."` | |
| 4 | Effects section declares database | |
| 5 | All blocks closed with `end` | |
| 6 | Queries both accounts | |
| 7 | Match handles None case with error return | |
| 8 | Validates sufficient balance before transfer | |
| 9 | Uses transaction block for updates | |
| 10 | Returns struct with new balances | |
| 11 | All three type definitions present | |

**Scoring:** 11/11 = perfect, 9-10 = minor issues, 7-8 = partial success, <7 = significant gaps
