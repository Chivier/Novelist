# Plans

Last updated: 2026-05-27

## Active Plan Surface

- Memory and huge-file optimization remains in
  [exec-plans/active/2026-04-12-memory-optimization.md](exec-plans/active/2026-04-12-memory-optimization.md).
- Broader maintenance items are tracked in
  [exec-plans/tech-debt-tracker.md](exec-plans/tech-debt-tracker.md).

## Completed Plan Archive

Completed implementation drafts are archived in
[exec-plans/completed/](exec-plans/completed/). Workflow-generated
Superpowers specs and plans can remain under `docs/superpowers/`; when they
ship, link them from a release note and copy lasting decisions into product
specs or design docs.

Use those files as historical evidence only. For new work, start with:

1. Product behavior in `docs/product-specs/`.
2. Durable design constraints in `docs/design-docs/`.
3. A focused execution plan under `docs/exec-plans/active/`.
4. A completed-plan move or deletion once the feature ships.

## Plan Hygiene

- Do not leave stale implementation drafts in the primary docs surface.
- A completed plan should either be deleted or moved to `completed/`.
- A design decision that still matters belongs in `design-docs/`, not in an
  old plan.
