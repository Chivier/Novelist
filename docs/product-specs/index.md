# Product Specs Index

Last updated: 2026-04-25

Product specs describe intended user behavior. They are separate from
implementation plans so the product story remains readable after the code
ships.

## Current Product Surface

| Area | Status | Notes |
|---|---|---|
| WYSIWYG Markdown editor | Implemented | CodeMirror 6 decorations, CJK-aware helpers, large-file modes. |
| Project workspace | Implemented | Folder-first model with file tree, tabs, recent projects, smart new-file naming. |
| Deep writing modes | Implemented | Zen, focus, typewriter, outline, split view. |
| Templates | Implemented | Bundled templates, caret anchors, new-file and insert workflows. |
| Visual thinking | Implemented | Mindmap overlay, `.canvas`, `.kanban`. |
| Export | Implemented | Pandoc-backed HTML, PDF, DOCX, EPUB. |
| Sync and snapshots | Partial/active | Snapshot and WebDAV plumbing exist; reliability and UX need release-grade validation. |
| Image hosting | Draft (v0.2.4) | See [2026-05-06-image-host.md](2026-05-06-image-host.md). Qiniu / Aliyun OSS / S3 / R2 / imgur / sm.ms / Custom. |
| Publishing | Draft (v0.2.4) | See [2026-05-06-publish.md](2026-05-06-publish.md). Ghost / WordPress / WP.com / Medium. |
| New-user onboarding | Draft | See [new-user-onboarding.md](new-user-onboarding.md). |

## Specs

- [New-user onboarding](new-user-onboarding.md)
- [Competitive analysis](competitive-analysis.md)
- [Image hosting (v0.2.4)](2026-05-06-image-host.md)
- [Publishing (v0.2.4)](2026-05-06-publish.md)
- [File renaming & filename macros (v0.2.4)](2026-05-07-v0.2.4-rename-and-macros.md)
