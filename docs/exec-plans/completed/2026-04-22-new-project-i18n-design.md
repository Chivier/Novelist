# New Project Dialog — i18n + bilingual template scaffolding

**Date:** 2026-04-22
**Scope:** `NewProjectDialog.svelte`, locale files, `core/src/commands/template.rs`, new `core/src/services/template_scaffold.rs`
**Out of scope:** User-authored templates (`~/.novelist/templates/*/template.toml`), template ZIP import, Settings panel, Welcome page (already localized), command palette entry (already localized).

## Problem

The New Project dialog has three i18n leaks that show up in the current UI regardless of UI language:

1. **Category sidebar labels** (`General / Fiction / Non-fiction / Personal / Custom`) are hardcoded strings in `NewProjectDialog.svelte:41-47`.
2. **Template names and descriptions** come from Rust (`core/src/commands/template.rs` `builtin_templates()`) as hardcoded strings — a confusing mix where `Blank / Novel / Blog / Journal` are English while `长篇小说 / 短篇小说 / 剧本` are Chinese. This is "language-tied templates" rather than i18n'd labels.
3. **Descriptions** are all English regardless of UI.

Additionally the scaffolding file names are language-tied:
- `novel` generates `Chapter 1.md` / `Chapter 2.md` / `Chapter 3.md` (English).
- `long-novel` generates `大纲.md / 人物设定.md / 第一卷/第一章.md ...` (Chinese).
- `short-story` / `screenplay` similarly Chinese-only.

A Chinese-UI user who picks `novel` gets English chapter files; an English-UI user who picks `long-novel` gets Chinese planning docs.

## Goal

1. All labels in the New Project dialog follow the current UI locale.
2. Each of the 7 built-in templates exists as one conceptual card (unchanged count) but its scaffolded files + seed content switch between EN and ZH based on UI locale at creation time.
3. User-authored templates remain single-language (read from their `template.toml`) — out of scope to change.

## Non-goals

- No new locales beyond `en` and `zh-CN`. The locale detection uses a "starts with `zh`" rule so future `zh-TW`/`zh-HK` can reuse the ZH scaffolding without a code change, but ja/fr/etc. will not land in this change.
- No migration of existing projects' file names. This only affects **newly-created** projects.
- No change to user templates, template ZIP import, `save_project_as_template`.

## Architecture

### Responsibility split

| Layer | Responsibility |
|-------|----------------|
| Frontend (`app/lib/i18n/locales/*.ts`) | Display strings: template names, descriptions, category labels |
| Rust `list_templates()` | Returns stable `id`, `category`, `builtin`; `name`/`description` retained as EN fallbacks only |
| Rust `create_project_from_template(..., locale)` | Scaffolds files + seed content from a locale-keyed table |

### Data flow

```
NewProjectDialog
  ├─ mount: commands.listTemplates()
  │    → Rust returns [{id, category, builtin, name (EN fallback), description (EN fallback)}]
  ├─ render card: helper looks up t(`template.${id}.name`); falls back to tpl.name if key missing
  ├─ render category: t(`template.category.${cat}`); falls back to cat
  ├─ click Create →
  │    commands.createProjectFromTemplate(id, name, parentDir, i18n.locale)
  └─ Rust scaffold_builtin(id, name, dest, locale_enum)
       → template_scaffold::files_for(id, locale, name)
       → template_scaffold::outline_order(id, locale)
```

### Invariants

- Template `id` is stable and English: `blank / novel / long-novel / short-story / screenplay / blog / journal`. It is the anchor for i18n keys and for any persisted project metadata.
- A missing i18n key falls back to the Rust-returned `name`/`description` string. After this change those fallbacks are all English — no mixed EN/ZH rendering.
- User templates (`builtin: false`) are displayed using their `template.toml` `name`/`description` verbatim; helpers short-circuit on `!tpl.builtin`.

## Template matrix (7 × 2)

Date-only files are locale-neutral; directory names like `posts/` / `drafts/` stay in English for routing intuition.

| id | EN scaffold | ZH scaffold |
|----|-------------|-------------|
| `blank` | `.novelist/project.toml` only | same |
| `novel` | `Chapter 1.md`, `Chapter 2.md`, `Chapter 3.md` | `第一章.md`, `第二章.md`, `第三章.md` |
| `long-novel` | `Outline.md`, `Characters.md`, `Worldbuilding.md`, `Volume 1/Chapter 1.md`, `Volume 1/Chapter 2.md`, `Volume 1/Chapter 3.md`, `Volume 2/Chapter 4.md`, `Volume 2/Chapter 5.md` | `大纲.md`, `人物设定.md`, `世界观.md`, `第一卷/第一章.md`, `第一卷/第二章.md`, `第一卷/第三章.md`, `第二卷/第四章.md`, `第二卷/第五章.md` (= current behavior) |
| `short-story` | `Manuscript.md`, `Notes.md` | `正文.md`, `创作笔记.md` (= current) |
| `screenplay` | `Cast.md`, `Act 1.md`, `Act 2.md`, `Act 3.md` | `人物表.md`, `第一幕.md`, `第二幕.md`, `第三幕.md` (= current) |
| `blog` | `posts/first-post.md` + empty `drafts/` | `posts/第一篇.md` + empty `drafts/` |
| `journal` | `YYYY-MM-DD.md` with `# YYYY-MM-DD` | same |

`[outline].order` in the generated `project.toml` references the locale-matching file names.

### Seed content highlights (EN additions)

`Outline.md`:
```
# Outline

## Synopsis

## Themes

## Plotlines

### Main Plot

### Subplot
```

`Characters.md`:
```
# Characters

## Protagonist

**Name**:
**Age**:
**Personality**:
**Background**:

---

## Supporting Cast

---

## Antagonist
```

`Worldbuilding.md`:
```
# Worldbuilding

## Era

## Geography

## Society

## Key Concepts
```

`Notes.md` (short-story):
```
# Notes

## Inspiration

## Core Conflict

## Character Sketches

## Ending Ideas
```

`Cast.md`:
```
# Cast

## Main Characters

**Name**:
**Role**:
**Bio**:

---

## Supporting Characters
```

`Act 1/2/3.md`:
```
# Act N

## Scene 1

**Location**:
**Time**:

---
```

The Chinese seed content is the existing string-literal set in `template.rs` — no change.

## Frontend changes

### `app/lib/components/NewProjectDialog.svelte`

1. **Remove** the `categoryLabels: Record<string,string>` constant (L41-47).
2. **Add** three local helpers:

```ts
import { i18n, t } from '$lib/i18n';

function categoryLabel(cat: string): string {
  const key = `template.category.${cat}`;
  const translated = t(key);
  return translated === key ? cat : translated;
}

function templateName(tpl: TemplateInfo): string {
  if (!tpl.builtin) return tpl.name;
  const key = `template.${tpl.id}.name`;
  const translated = t(key);
  return translated === key ? tpl.name : translated;
}

function templateDescription(tpl: TemplateInfo): string {
  if (!tpl.builtin) return tpl.description;
  const key = `template.${tpl.id}.description`;
  const translated = t(key);
  return translated === key ? tpl.description : translated;
}
```

3. **Replace** the three usages in the template:
   - `{categoryLabels[cat] || cat}` → `{categoryLabel(cat)}`
   - `{tpl.name}` (inside the template card) → `{templateName(tpl)}`
   - `{selectedTemplate.description}` → `{templateDescription(selectedTemplate)}`

4. **Pass locale** to the Rust call:

```ts
const result = await commands.createProjectFromTemplate(
  selectedTemplate.id,
  projectName.trim(),
  parentDir.trim(),
  i18n.locale,
);
```

### `app/lib/i18n/locales/en.ts`

Add after the `newProject.*` block:

```ts
'template.category.general': 'General',
'template.category.fiction': 'Fiction',
'template.category.non-fiction': 'Non-fiction',
'template.category.personal': 'Personal',
'template.category.custom': 'Custom',

'template.blank.name': 'Blank',
'template.blank.description': 'Empty project with default settings',
'template.novel.name': 'Novel',
'template.novel.description': 'Novel project with chapter structure',
'template.long-novel.name': 'Long Novel',
'template.long-novel.description': 'Multi-volume novel with character profiles, world-building notes, and chapter folders',
'template.short-story.name': 'Short Story',
'template.short-story.description': 'Short story with a single manuscript file and planning notes',
'template.screenplay.name': 'Screenplay',
'template.screenplay.description': 'Three-act screenplay with character list and scene structure',
'template.blog.name': 'Blog',
'template.blog.description': 'Blog with posts and drafts folders',
'template.journal.name': 'Journal',
'template.journal.description': 'Daily journal with date-based files',
```

### `app/lib/i18n/locales/zh-CN.ts`

Mirror keys with Chinese strings:

```ts
'template.category.general': '通用',
'template.category.fiction': '虚构',
'template.category.non-fiction': '非虚构',
'template.category.personal': '个人',
'template.category.custom': '自定义',

'template.blank.name': '空白',
'template.blank.description': '使用默认设置的空项目',
'template.novel.name': '小说',
'template.novel.description': '按章节组织的小说项目',
'template.long-novel.name': '长篇小说',
'template.long-novel.description': '分卷长篇，含人物设定、世界观与章节目录',
'template.short-story.name': '短篇小说',
'template.short-story.description': '单文件正文加创作笔记的短篇项目',
'template.screenplay.name': '剧本',
'template.screenplay.description': '三幕剧结构，含人物表和场景分场',
'template.blog.name': '博客',
'template.blog.description': '包含 posts 与 drafts 文件夹的博客项目',
'template.journal.name': '日记',
'template.journal.description': '按日期命名的每日日记',
```

## Rust changes

### `core/src/commands/template.rs`

1. **Signature:** add `locale: String` to `create_project_from_template`. Auto-regenerated TS binding in `app/lib/ipc/commands.ts` via tauri-specta.

2. **Internal locale enum** lives in `core/src/services/template_scaffold.rs` and is imported by `template.rs`:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScaffoldLocale { En, ZhCN }

impl ScaffoldLocale {
    pub fn from_tag(tag: &str) -> Self {
        if tag.to_ascii_lowercase().starts_with("zh") {
            ScaffoldLocale::ZhCN
        } else {
            ScaffoldLocale::En
        }
    }
}
```

3. **Delegate** all scaffolding string literals to `core/src/services/template_scaffold.rs`. The top-level `scaffold_builtin` becomes:

```rust
fn scaffold_builtin(
    template_id: &str,
    project_name: &str,
    dest: &Path,
    locale: ScaffoldLocale,
) -> Result<String, AppError> {
    std::fs::create_dir_all(dest.join(".novelist"))?;

    for f in template_scaffold::files_for(template_id, locale, project_name) {
        let full = dest.join(&f.name);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full, &f.content)?;
    }

    let order = template_scaffold::outline_order(template_id, locale);
    let ptype = template_scaffold::project_type(template_id);
    let daily_goal = template_scaffold::daily_goal(template_id);
    let config = build_project_toml(project_name, ptype, &order, daily_goal);
    std::fs::write(dest.join(".novelist/project.toml"), config)?;

    Ok(dest.to_string_lossy().to_string())
}
```

4. **Normalize `builtin_templates()` fallbacks to English** (so EN fallbacks never render Chinese): `"长篇小说" → "Long Novel"`, `"短篇小说" → "Short Story"`, `"剧本" → "Screenplay"`. These strings are only displayed when the frontend's i18n key lookup misses.

### `core/src/services/template_scaffold.rs` (new)

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScaffoldLocale { En, ZhCN }

impl ScaffoldLocale {
    pub fn from_tag(tag: &str) -> Self { /* see above */ }
}

pub struct ScaffoldFile {
    pub name: String,       // relative path, may include subdirs
    pub content: String,
}

pub fn files_for(id: &str, locale: ScaffoldLocale, project_name: &str) -> Vec<ScaffoldFile>;
pub fn outline_order(id: &str, locale: ScaffoldLocale) -> Vec<String>;
pub fn project_type(id: &str) -> &'static str;   // "novel" | "blog" | "journal"
pub fn daily_goal(id: &str) -> u32;               // 2000 / 1500 / 1000
```

Implementation: internal match on `(id, locale)` returning the file list. Content uses `format!` to inject `project_name` into the first chapter for `novel` only.

### `core/src/services/mod.rs`

```rust
pub mod template_scaffold;
```

## Testing

### Rust (`core/src/commands/template.rs`)

Update existing tests to pass `"zh-CN"`:
- `test_create_long_novel_project` — already asserts Chinese file names; keep assertions, add locale arg.
- `test_create_short_story_project` — same.
- `test_create_screenplay_project` — same.
- `test_create_novel_project` — currently asserts `Chapter 1.md`. Since `novel` under ZH becomes `第一章.md`, this test needs its locale set to `"en"` to match existing assertions. Add a sibling `test_create_novel_project_zh` asserting `第一章.md`.
- `test_create_blank_project`, `test_create_duplicate_fails` — pass any locale (`"en"`).

Add:
- `test_create_long_novel_project_en` — asserts `Outline.md`, `Characters.md`, `Worldbuilding.md`, `Volume 1/Chapter 1.md`, `Volume 2/Chapter 4.md`.
- `test_create_short_story_project_en` — asserts `Manuscript.md`, `Notes.md`.
- `test_create_screenplay_project_en` — asserts `Cast.md`, `Act 1.md`, `Act 2.md`, `Act 3.md`.
- `test_create_blog_project_en` — asserts `posts/first-post.md`.
- `test_create_blog_project_zh` — asserts `posts/第一篇.md`.
- `test_locale_fallback_unknown_tag` — passes `"ja"`, expects EN scaffolding (no panic).
- `test_outline_order_locale_en` — reads generated `project.toml`, checks `[outline].order` contains `"Outline.md"` (or equivalent EN file) for `long-novel`.
- `test_outline_order_locale_zh` — mirror for ZH.

`test_save_and_delete_template` is untouched (no locale dependency).

### Frontend (`tests/unit/components/new-project-dialog.test.ts` — new)

- `categoryLabel` with known category returns the translated string.
- `categoryLabel` with unknown category returns the raw input.
- `templateName` for `builtin: false` returns `tpl.name` untouched.
- `templateName` for `builtin: true` returns the i18n value when present, or `tpl.name` when the key is missing.
- Switching `i18n.locale` re-renders the dialog with translated labels (snapshot or direct text assertion on category sidebar items).
- `commands.createProjectFromTemplate` mock receives locale as the 4th argument.

### E2E (optional, stretch)

Skip unless a low-cost opportunity emerges — frontend unit tests are sufficient for this change.

### Manual QA checklist

1. Switch UI to English → open New Project → all cards/categories/descriptions are English only. Confirm no `长篇小说 / 短篇小说 / 剧本` bleed-through.
2. Switch UI to 简体中文 → open 新建项目 → all labels are Chinese.
3. Under EN, create a `long-novel` project → file tree shows `Outline.md`, `Characters.md`, `Volume 1/Chapter 1.md`, etc.
4. Under ZH, create a `long-novel` project → file tree shows `大纲.md`, `人物设定.md`, `第一卷/第一章.md` (= current behavior).
5. Open the newly-created project → no missing-file errors; `.novelist/project.toml` parses.
6. Install a user-authored template (existing flow) → its name/description come from its `template.toml`, not i18n keys.

## Coverage impact

Existing frontend threshold (stmt 73 / branch 67 / func 75 / line 75) stays. New scaffolding branches in Rust are fully covered by the new tests above; no waiver needed.

## Risks & mitigations

| Risk | Mitigation |
|------|------------|
| tauri-specta regenerates `commands.ts` — stale binding during dev | Instructions already say rerun `pnpm tauri dev` after Rust signature changes; CLAUDE.md covers this. |
| Existing user has a project metadata referencing old Chinese file names | Out of scope — only affects new project creation. |
| i18n key typo → fallback to Rust EN string | Acceptable degradation; still coherent English. |
| Future ja/fr locale: uses EN scaffolding | Acceptable; adding locales is a follow-up change, not a regression. |
