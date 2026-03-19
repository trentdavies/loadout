# Future Work: `agent collect` Enhancements

Deferred features and open questions for the `equip agent collect` command.

## 1. Skill movement between plugins

`equip move <pattern> --to <plugin>` — relocate skills between plugins, updating registry provenance and re-equipping to agents that had them installed.

## 2. Plugin rename

`equip source rename <old> <new>` or similar. Could leverage the existing `reconcile_with_config()` machinery to cascade the rename through the registry.

## 3. External source matching for untracked skills

When collecting untracked skills, search all registered sources for skills with matching names and offer to link provenance rather than defaulting to `local/`. This would let users "claim" a skill that came from a known source but was installed before tracking existed.

## 4. Reconciling adopted skills back to agents

After `collect --adopt`, the skill lives in the local plugin but isn't re-equipped to the agent via the normal flow. Document (or automate) the `equip @agent <pattern>` step needed to push changes back.

## 5. Command naming

Consider alternatives to `collect`: `harvest`, `gather`, `pull`. Needs user feedback — `collect` may imply aggregation rather than the reverse-sync this command performs.
