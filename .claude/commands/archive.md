# Archive Completed Plan

Move the current completed plan from `.claude/plans/` to `.claude/implemented_plans/`.

## Instructions

1. **Find the active plan**: Look in `.claude/plans/` for the most recently modified `.md` file
2. **Verify completion**: Check if the plan has `## Status: Implemented` near the top
   - If not present, add it below the title before moving
3. **Create destination directory** if it doesn't exist: `.claude/implemented_plans/`
4. **Move the plan**: Move the file from `.claude/plans/` to `.claude/implemented_plans/`
5. **Confirm**: Report which plan was archived

## Example Output

```
Archived: platform-abstraction.md
  From: .claude/plans/platform-abstraction.md
  To:   .claude/implemented_plans/platform-abstraction.md
```

## Error Handling

- If no plans exist in `.claude/plans/`, report "No active plans to archive"
- If multiple plans exist, list them and ask which one to archive
- If the plan doesn't appear complete (no Status: Implemented), ask for confirmation before archiving
