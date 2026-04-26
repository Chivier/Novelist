# Product Sense

Last updated: 2026-04-25

## Read Of The Project

Novelist is not a prototype anymore. It has a real editor, real project
workflows, real tests, and release packaging. The current work is about
hardening, reducing drift, and making the product easier to extend without
turning it into a heavy IDE.

## Target User

The primary user is a novelist or long-form writer who wants:

- Plain files they can trust.
- A focused WYSIWYG Markdown surface.
- CJK-aware writing behavior.
- Visual planning tools without leaving the project.
- AI-assisted customization through source edits rather than a large config
  UI.

## Product Strengths

- Clear differentiation: lightweight desktop writing, WYSIWYG Markdown,
  plugin support, templates, and CJK support in one app.
- Strong local-first stance.
- Useful writer features already implemented rather than merely planned.
- Tests and CI are credible enough to support rapid iteration.

## Product Risks

- The app has many core surfaces, so feature discoverability can become
  noisy.
- Sync and conflict flows need extra confidence before they become a trust
  anchor.
- Historical docs can mislead contributors if old plans are treated as
  current architecture.

## Near-Term Product Priorities

1. Keep editor reliability and CJK correctness ahead of new feature breadth.
2. Harden onboarding and first project creation.
3. Make templates, mindmap, canvas, and kanban discoverable without crowding
   writing.
4. Treat sync as reliability work, not just settings UI.
