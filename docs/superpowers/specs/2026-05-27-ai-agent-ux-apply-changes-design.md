# AI Agent UX and Apply Changes Design

Date: 2026-05-27
Status: draft for user review

## Context

Novelist currently ships two native AI side panels:

- AI Talk, an OpenAI-compatible chat and rewrite panel.
- AI Agent, a Claude CLI backed agent panel with sessions, tool cards, plan/act mode, and project-scoped persistence.

The foundation is useful, but the user experience exposes too much of the prompt machinery. Editor selection can silently become context, `@` mentions are static tokens rather than searchable context insertion, the narrow right panel has too many large text buttons, and AI Agent has no review-first way to apply generated changes.

Cursor is the reference point for two behaviors:

- `@` references should search and preview files/folders/context before they become attached chips.
- Apply should be a separate integration step. The chat or agent proposes code/text changes; an Apply layer integrates them into files and lets the user review diffs before accepting.

For Novelist, Apply must fit a writing app. The default should be review-first and calm, not automatic file mutation.

## Goals

1. Make AI Agent the primary power workflow for project-aware work, while keeping AI Talk lightweight for chat and rewrite.
2. Replace implicit selection injection with explicit, removable context chips.
3. Make `@` context flexible: current file, selection, outline, open tabs, project files, folders, memory, skills, commands, and recent AI sessions should be discoverable from one menu.
4. Reduce visual weight in the right panel, especially action buttons.
5. Add Apply Changes as a review-first flow for AI-generated edits across one or more files.
6. Preserve CJK and long-file safety by using character offsets carefully, showing previews, and avoiding hidden bulk rewrites.

## Non-Goals

- No hidden model integration in the Rust core.
- No vector database or cloud codebase index in this iteration.
- No direct Cursor clone. Cursor is a design reference, not a requirement to match every affordance.
- No automatic agent self-apply as the default.
- No full git checkpoint system in the first Apply Changes implementation.

## Current Problems

### Selection Feels Too Sticky

AI Talk polls the active editor selection and, when `includeSelection` is on, injects it into the next chat request. This makes ordinary editing selection feel like an AI action. AI Agent does not show the same live selection chip, which makes behavior inconsistent across panels.

### Context Is Prompt Text Instead Of UI State

AI Agent builds a context pack and stores that full prompt as the visible user turn. This pollutes the transcript, session title, and saved session JSON with implementation details such as `## Context 1`.

### `@` Mentions Are Not Flexible Enough

The current mention menu lists fixed items:

- `@selection`
- `@current`
- `@outline`
- `@file:`
- `@folder:`

`@file:` and `@folder:` depend on typed path fragments and attach the first matching item. There is no file search result list, no path preview, no ambiguity resolution, no keyboard selection, and no drag/drop from the sidebar into the AI composer.

### Controls Are Too Large For A Narrow Panel

The default right panel width is 280px, but AI Talk and AI Agent expose many text buttons in one row. The result is cramped and visually noisy.

### Agent Output Has No Apply Layer

AI Agent currently renders assistant text, tool-use cards, and tool-result cards. If the agent edits files through the Claude CLI, Novelist does not present a first-class change review surface. If the agent only suggests changes in text, there is no safe way to apply them into the editor.

## Recommended Design

Use a shared AI composer and context system for both AI Talk and AI Agent, then add a review-first Apply Changes layer for Agent.

### Interaction Model

AI composer has three visible regions:

1. Context row: compact chips for attached items.
2. Input box: plain request text with slash and mention menus.
3. Action bar: mode selector, overflow menu, send/stop button.

Selection is never silently committed as context. When the editor has selected text and an AI panel is focused, show a suggested chip such as `Selection - 81 chars`. The chip has three states:

- Suggested: visible but not attached.
- Attached: included in the next request and shown in the context row.
- Dismissed: ignored until the selection changes.

Clicking the chip attaches it. Typing `@selection` also attaches it. This keeps selection useful without making normal editing feel dangerous.

### `@` Context Menu

The mention menu should become a searchable picker. Typing `@` opens categories; typing after `@` searches across available items.

Supported items for this iteration:

- Current file
- Current selection
- Current outline
- Open tabs
- Project files
- Project folders
- Project memory
- AI skills under `.novelist/ai/skills`
- AI commands under `.novelist/ai/commands`
- Recent AI Talk/Agent sessions

Each result shows:

- Label
- Kind icon
- Path or source
- Estimated size
- Whether it will be full content, outline only, or truncated

Picking a result inserts a chip, not raw prompt text. The input can keep a lightweight token for readability, but request construction uses the chip state.

Keyboard behavior:

- Arrow keys move through results.
- Enter attaches the highlighted result.
- Escape closes the menu.
- Backspace on an empty input can focus the last chip.

Mouse and sidebar behavior:

- Dragging a sidebar file or folder into the AI composer creates the matching context chip.
- Dropping multiple files creates multiple chips, bounded by the context budget.

### Context Data Model

Introduce a shared `AiContextAttachment` type:

```ts
type AiContextAttachment = {
  id: string;
  kind:
    | 'selection'
    | 'current-file'
    | 'outline'
    | 'open-tab'
    | 'project-file'
    | 'folder-summary'
    | 'memory'
    | 'skill'
    | 'command'
    | 'session';
  label: string;
  path?: string;
  source: 'editor' | 'project' | 'ai-assets' | 'session';
  mode: 'full' | 'excerpt' | 'outline' | 'summary';
  content: string;
  estimatedChars: number;
  truncated: boolean;
};
```

The visible user turn should store:

- `displayText`: what the user typed.
- `attachmentIds` or a snapshot of attachment metadata.

The outbound prompt should be derived at send time and should not replace the visible user text.

### Panel Layout

AI Agent should prioritize the transcript and composer:

- Top row: session tabs, compact title, status dot, settings icon.
- Mode: segmented control or compact dropdown with `Ask`, `Plan`, `Act`.
- Secondary actions: `Fork`, `Save`, `Compact`, `Clear` move into an overflow menu.
- Primary action: one prominent Send/Stop button.
- Tool cards: compact by default, expanded on click.

AI Talk can keep Chat/Rewrite tabs, but should use the same composer/context row in Chat mode. Rewrite should become a special command path rather than a completely separate mental model where possible.

### Apply Changes

Apply Changes is a new review-first subsystem. It accepts an AI-generated proposal and converts it into a structured change set.

V1 supports:

- Apply to the active editor selection.
- Apply to the active file.
- Apply to one or more project files by path.
- Accept all, reject all, accept file, reject file.

Hunk-level accept/reject is deferred to Phase 4. V1 may display hunks for review, but acceptance is file-level.

Change set shape:

```ts
type AiChangeSet = {
  id: string;
  sourceSessionId: string;
  createdAt: string;
  summary: string;
  files: AiFileChange[];
};

type AiFileChange = {
  path: string;
  status: 'modify' | 'create';
  originalText: string | null;
  proposedText: string;
  hunks: AiDiffHunk[];
  conflict?: string;
};

type AiDiffHunk = {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: Array<{ kind: 'context' | 'added' | 'removed'; text: string }>;
};
```

Apply flow:

1. Agent proposes a change set in a structured fenced block or tool-style envelope.
2. Novelist parses it and shows an Apply Changes card in the Agent transcript.
3. User opens the card to see a diff grouped by file.
4. User accepts or rejects files/hunks.
5. Accepted changes are applied through editor dispatch for open active files and through atomic write commands for project files.
6. The file watcher/self-write suppression path should treat accepted Apply changes as Novelist writes.

The first implementation should prefer deterministic parsing over a separate Apply model. If a proposal cannot be parsed safely, show it as text and offer Copy, not Apply.

### Proposal Format

The initial proposal format should be explicit and easy to validate:

````md
```novelist-change-set
{
  "summary": "Tighten Chapter 1 opening",
  "files": [
    {
      "path": "/tmp/project/Chapter 1.md",
      "status": "modify",
      "proposedText": "# Chapter 1\n\n..."
    }
  ]
}
```
````

For active-selection rewrites, `path` may be the active file and the change may include a selection range. Ranges must be validated against the latest document snapshot before applying.

### Agent Prompting

AI Agent should be instructed:

- For analysis or conversation, answer normally.
- For proposed edits, output prose plus a `novelist-change-set` block when the user asks to change files.
- Do not hide file modifications inside plain text.
- Prefer small, reviewable changes.

Permission mode remains separate. Even if Claude CLI can edit files, Novelist should steer the panel UI toward proposal-and-review unless the user explicitly chooses direct agent editing.

## Architecture

### Shared Context Modules

Refactor the current shared context code into three layers:

- `context-sources.ts`: reads editor/project/session/assets and produces candidate attachments.
- `context-picker.svelte`: UI for search, keyboard navigation, preview, and attach.
- `context-pack.ts`: converts selected attachments into an outbound prompt.

Existing `context.ts` can be split or kept as a facade during migration.

### Shared Composer

Create a reusable `AiComposer.svelte` used by AI Talk and AI Agent.

Responsibilities:

- Text input.
- Suggested selection chip.
- Attached context row.
- Mention and slash menus.
- Compact action bar.
- Drag/drop handling.

The panel owns send behavior; the composer owns interaction state.

### Apply Modules

Add shared Apply modules:

- `apply/change-set.ts`: schema, parser, validation.
- `apply/diff.ts`: line/word diff helpers with CJK-safe text handling.
- `apply/apply.ts`: apply accepted changes to editor or files.
- `ApplyChangesCard.svelte`: review UI.

AI Agent consumes Apply first. AI Talk Rewrite may later reuse the same review card.

### Rust Boundary

Avoid new AI logic in Rust. Rust may need only narrow file commands if the current frontend commands are insufficient:

- Read current file before apply.
- Atomic write accepted content.
- Create a new file with body for accepted `create` changes.

Do not add a Rust-side patch engine unless frontend diff/apply cannot safely handle the cases.

## Error Handling

Apply must refuse when:

- The file path is outside the current project unless explicitly allowed.
- The latest file content no longer matches the expected original snapshot.
- The change set JSON is invalid.
- A proposed file is too large to show safely in the panel.
- The target is a binary or unsupported file.

On conflict, show the file as `Needs review` and provide:

- Copy proposed text.
- Open file.
- Regenerate request.

No silent overwrite.

## Testing Plan

Unit tests:

- Mention parsing and search result ranking.
- Context attachment dedupe and prompt packing.
- Selection suggested/attached/dismissed state.
- Change set parser.
- Invalid change set rejection.
- CJK diff behavior.
- Apply validation against stale original text.

E2E tests:

- `@` menu searches project files and attaches the selected file as a chip.
- Sidebar drag into AI composer creates a context chip.
- Selection does not auto-send as context until attached.
- Agent user turn displays clean text, while outbound prompt contains attached context.
- Apply Changes card appears from a `novelist-change-set` response.
- Accepting an Apply change updates an open editor.
- Rejecting an Apply change leaves the file unchanged.

Manual/browser checks:

- Default 280px panel does not overflow action buttons.
- Wide panel remains readable.
- Dark/light themes render chips and diffs legibly.
- Long CJK selections stay readable and do not shift layout.

## Rollout Phases

### Phase 1: Context UX Cleanup

- Shared composer shell.
- Explicit selection chip.
- Clean visible transcript text.
- Overflow action menu.
- Remove Talk-only implicit selection injection.

### Phase 2: Real `@` Picker

- Search project files/folders/open tabs/assets.
- Add chips from picker.
- Add sidebar drag/drop to composer.
- Keep prompt packing behind the chip model.

### Phase 3: Apply Changes V1

- Parse `novelist-change-set`.
- Render Apply Changes card.
- Accept/reject at file level.
- Apply to active editor and project files.
- Add stale-content conflict checks.

### Phase 4: Polish

- Hunk-level accept/reject.
- Session references.
- Better folder condensation.
- Optional direct-agent-edit mode with explicit warning.

## Acceptance Criteria

- Selecting text in the editor no longer silently changes the next AI request.
- A user can type `@chap` and attach `Chapter 1.md` from a result list without knowing `@file:` syntax.
- AI Agent transcript shows the user's actual request, not the internal context pack.
- AI Agent action bar fits cleanly in the default right panel width.
- An agent-generated change set can be reviewed and accepted into an open markdown file.
- Rejected changes leave files untouched.
- Stale file conflicts are detected before overwrite.
- Existing AI panel E2E tests continue to pass, with new E2E coverage for context and Apply.
