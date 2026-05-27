# AI Agent UX Apply Changes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build explicit AI context attachment, a searchable `@` picker, compact AI panel controls, and review-first Apply Changes V1 for Novelist's native AI panels.

**Architecture:** Keep Rust out of AI logic and implement the UX in focused TypeScript/Svelte modules under `app/lib/components/ai-shared/`. Introduce a shared composer/context attachment model, integrate it into AI Talk and AI Agent, then add a deterministic `novelist-change-set` parser and file-level Apply Changes review card.

**Tech Stack:** Svelte 5 runes, TypeScript, CodeMirror editor dispatch, existing Tauri IPC commands, Vitest unit tests, Playwright browser E2E with mocked IPC.

---

## File Structure

- Create `app/lib/components/ai-shared/attachments.ts`
  - Owns `AiContextAttachment`, attachment candidate search, prompt packing, and visible-turn helpers.
- Create `app/lib/components/ai-shared/selection-state.ts`
  - Pure suggested/attached/dismissed selection state reducer.
- Create `app/lib/components/ai-shared/AiComposer.svelte`
  - Shared composer with context chips, suggested selection chip, searchable `@` picker, slash menu slot, drag/drop, compact action bar.
- Create `app/lib/components/ai-shared/ApplyChangesCard.svelte`
  - File-level Apply Changes review UI.
- Create `app/lib/components/ai-shared/apply-change-set.ts`
  - `novelist-change-set` parser, validation, line diff generation, and conflict checks.
- Modify `app/lib/components/ai-shared/context.ts`
  - Keep legacy exports as a facade and route new code through `attachments.ts`.
- Modify `app/lib/components/ai-agent/sessions.svelte.ts`
  - Extend user turns to store `displayText` and attachment metadata without breaking old sessions.
- Modify `app/lib/components/ai-agent/AiAgentImpl.svelte`
  - Use `AiComposer`, keep visible transcript clean, parse change sets from assistant output, and render Apply cards.
- Modify `app/lib/components/ai-talk/AiTalkImpl.svelte`
  - Use explicit composer attachments instead of implicit selection injection.
- Modify `tests/unit/components/ai-shared-context.test.ts`
  - Add attachment search, prompt packing, and clean-display tests.
- Create `tests/unit/components/ai-shared-selection-state.test.ts`
  - Cover suggested/attached/dismissed selection state.
- Create `tests/unit/components/ai-shared-apply-change-set.test.ts`
  - Cover parser, invalid proposals, CJK-safe line diffs, and stale conflict detection.
- Modify `tests/unit/components/ai-agent-sessions.test.ts`
  - Cover migration-safe clean user turns.
- Modify `tests/e2e/specs/ai-panels.spec.ts`
  - Add E2E coverage for `@` file attach, explicit selection attach, clean Agent transcript, and Apply Changes accept/reject.

## Task 1: Attachment Model and Prompt Packing

**Files:**
- Create: `app/lib/components/ai-shared/attachments.ts`
- Modify: `app/lib/components/ai-shared/context.ts`
- Modify: `tests/unit/components/ai-shared-context.test.ts`

- [ ] **Step 1: Write failing attachment tests**

Add these imports to `tests/unit/components/ai-shared-context.test.ts`:

```ts
import {
  attachmentToContextItem,
  buildPromptFromAttachments,
  createAttachmentFromContext,
  displayTextFromInput,
  searchAttachmentCandidates,
  type AiContextAttachment,
} from '$lib/components/ai-shared/attachments';
```

Append this describe block:

```ts
describe('[contract] AI context attachments', () => {
  const base: AiContextAttachment[] = [
    {
      id: 'file:/project/Chapter 1.md',
      kind: 'project-file',
      label: 'Chapter 1.md',
      path: '/project/Chapter 1.md',
      source: 'project',
      mode: 'full',
      content: '# Chapter 1\n\nOpening text',
      estimatedChars: 24,
      truncated: false,
    },
    {
      id: 'memory',
      kind: 'memory',
      label: 'Project memory',
      path: '/project/.novelist/ai/memory.md',
      source: 'ai-assets',
      mode: 'summary',
      content: 'Tone notes',
      estimatedChars: 10,
      truncated: false,
    },
  ];

  it('searches attachment candidates by label and path', () => {
    expect(searchAttachmentCandidates(base, 'chap').map((x) => x.id)).toEqual(['file:/project/Chapter 1.md']);
    expect(searchAttachmentCandidates(base, 'memory').map((x) => x.id)).toEqual(['memory']);
  });

  it('builds outbound prompt without replacing visible text', () => {
    const packed = buildPromptFromAttachments('summarize this', base, 2000);
    expect(packed.visibleText).toBe('summarize this');
    expect(packed.outboundText).toContain('## Context 1: Chapter 1.md');
    expect(packed.outboundText).toContain('## User request\nsummarize this');
  });

  it('strips mention and skill tokens only for display text when requested', () => {
    expect(displayTextFromInput('@current $plot-doctor summarize this')).toBe('summarize this');
  });

  it('round-trips legacy context items through attachments', () => {
    const attachment = createAttachmentFromContext({
      id: 'selection',
      kind: 'selection',
      label: 'Selection',
      path: '/project/a.md',
      content: 'selected text',
    });
    expect(attachment.source).toBe('editor');
    expect(attachmentToContextItem(attachment)).toMatchObject({
      id: 'selection',
      kind: 'selection',
      label: 'Selection',
      content: 'selected text',
    });
  });
});
```

- [ ] **Step 2: Run the focused unit test and verify failure**

Run: `pnpm vitest run --project unit tests/unit/components/ai-shared-context.test.ts`

Expected: FAIL with module-not-found errors for `$lib/components/ai-shared/attachments`.

- [ ] **Step 3: Implement `attachments.ts`**

Create `app/lib/components/ai-shared/attachments.ts`:

```ts
import {
  buildContextPack,
  contextPackToPrompt,
  stripMentionTokens,
  stripSkillTokens,
  type AiContextItem,
  type AiContextKind,
} from './context';

export type AiContextAttachmentKind =
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

export type AiContextAttachment = {
  id: string;
  kind: AiContextAttachmentKind;
  label: string;
  path?: string;
  source: 'editor' | 'project' | 'ai-assets' | 'session';
  mode: 'full' | 'excerpt' | 'outline' | 'summary';
  content: string;
  estimatedChars: number;
  truncated: boolean;
};

export type PackedAttachmentPrompt = {
  visibleText: string;
  outboundText: string;
  attachments: AiContextAttachment[];
  estimatedChars: number;
};

const EDITOR_KINDS = new Set<AiContextAttachmentKind>(['selection', 'current-file', 'outline', 'open-tab']);
const ASSET_KINDS = new Set<AiContextAttachmentKind>(['memory', 'skill', 'command']);

function sourceForKind(kind: AiContextAttachmentKind): AiContextAttachment['source'] {
  if (EDITOR_KINDS.has(kind)) return 'editor';
  if (ASSET_KINDS.has(kind)) return 'ai-assets';
  if (kind === 'session') return 'session';
  return 'project';
}

function modeForKind(kind: AiContextAttachmentKind): AiContextAttachment['mode'] {
  if (kind === 'outline' || kind === 'folder-summary') return 'outline';
  if (kind === 'memory' || kind === 'session') return 'summary';
  if (kind === 'selection' || kind === 'open-tab') return 'excerpt';
  return 'full';
}

export function createAttachmentFromContext(item: AiContextItem): AiContextAttachment {
  const kind = item.kind === 'manual-note'
    ? item.id === 'memory' ? 'memory' : 'skill'
    : item.kind as AiContextAttachmentKind;
  return {
    id: item.id,
    kind,
    label: item.label,
    path: item.path,
    source: sourceForKind(kind),
    mode: modeForKind(kind),
    content: item.content,
    estimatedChars: item.content.length,
    truncated: item.truncated ?? false,
  };
}

export function attachmentToContextItem(attachment: AiContextAttachment): AiContextItem {
  const kind: AiContextKind = attachment.kind === 'memory' || attachment.kind === 'skill' || attachment.kind === 'command' || attachment.kind === 'session'
    ? 'manual-note'
    : attachment.kind;
  return {
    id: attachment.id,
    kind,
    label: attachment.label,
    path: attachment.path,
    content: attachment.content,
    truncated: attachment.truncated,
  };
}

export function dedupeAttachments(items: readonly AiContextAttachment[]): AiContextAttachment[] {
  const seen = new Set<string>();
  const out: AiContextAttachment[] = [];
  for (const item of items) {
    const key = `${item.kind}:${item.path ?? item.id}`;
    if (seen.has(key)) continue;
    seen.add(key);
    out.push(item);
  }
  return out;
}

export function searchAttachmentCandidates(
  candidates: readonly AiContextAttachment[],
  query: string,
): AiContextAttachment[] {
  const q = query.trim().toLowerCase().replace(/^@/, '');
  const unique = dedupeAttachments(candidates);
  if (!q) return unique;
  return unique.filter((item) => {
    const haystack = `${item.label} ${item.path ?? ''} ${item.kind}`.toLowerCase();
    return haystack.includes(q);
  });
}

export function displayTextFromInput(input: string): string {
  return stripSkillTokens(stripMentionTokens(input)).replace(/\s+/g, ' ').trim();
}

export function buildPromptFromAttachments(
  userText: string,
  attachments: readonly AiContextAttachment[],
  limit?: number,
): PackedAttachmentPrompt {
  const visibleText = userText.trim();
  const contextItems = dedupeAttachments(attachments).map(attachmentToContextItem);
  const pack = buildContextPack(visibleText, contextItems, limit);
  return {
    visibleText,
    outboundText: pack.items.length > 0 ? contextPackToPrompt(pack) : visibleText,
    attachments: pack.items.map(createAttachmentFromContext),
    estimatedChars: pack.estimatedChars,
  };
}
```

- [ ] **Step 4: Run the focused unit test and verify pass**

Run: `pnpm vitest run --project unit tests/unit/components/ai-shared-context.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit Task 1**

Run:

```bash
git add app/lib/components/ai-shared/attachments.ts app/lib/components/ai-shared/context.ts tests/unit/components/ai-shared-context.test.ts
git commit -m "feat(ai): add explicit context attachments"
```

Expected: commit includes only Task 1 files.

## Task 2: Selection State and Shared Composer

**Files:**
- Create: `app/lib/components/ai-shared/selection-state.ts`
- Create: `app/lib/components/ai-shared/AiComposer.svelte`
- Create: `tests/unit/components/ai-shared-selection-state.test.ts`

- [ ] **Step 1: Write failing selection-state tests**

Create `tests/unit/components/ai-shared-selection-state.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import {
  reduceSelectionSuggestion,
  type SelectionSuggestionState,
} from '$lib/components/ai-shared/selection-state';

const initial: SelectionSuggestionState = {
  snapshotKey: null,
  status: 'none',
};

describe('[contract] AI selection suggestion state', () => {
  it('shows a new selection as suggested rather than attached', () => {
    expect(reduceSelectionSuggestion(initial, { type: 'selection-changed', key: 'a:0:5' })).toEqual({
      snapshotKey: 'a:0:5',
      status: 'suggested',
    });
  });

  it('attaches the current suggestion explicitly', () => {
    const suggested = reduceSelectionSuggestion(initial, { type: 'selection-changed', key: 'a:0:5' });
    expect(reduceSelectionSuggestion(suggested, { type: 'attach' })).toEqual({
      snapshotKey: 'a:0:5',
      status: 'attached',
    });
  });

  it('dismisses a suggestion until the selection changes', () => {
    const dismissed = reduceSelectionSuggestion({ snapshotKey: 'a:0:5', status: 'suggested' }, { type: 'dismiss' });
    expect(dismissed).toEqual({ snapshotKey: 'a:0:5', status: 'dismissed' });
    expect(reduceSelectionSuggestion(dismissed, { type: 'selection-changed', key: 'a:0:5' })).toBe(dismissed);
    expect(reduceSelectionSuggestion(dismissed, { type: 'selection-changed', key: 'a:2:7' })).toEqual({
      snapshotKey: 'a:2:7',
      status: 'suggested',
    });
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run: `pnpm vitest run --project unit tests/unit/components/ai-shared-selection-state.test.ts`

Expected: FAIL with module-not-found for `selection-state`.

- [ ] **Step 3: Implement `selection-state.ts`**

Create `app/lib/components/ai-shared/selection-state.ts`:

```ts
export type SelectionSuggestionStatus = 'none' | 'suggested' | 'attached' | 'dismissed';

export type SelectionSuggestionState = {
  snapshotKey: string | null;
  status: SelectionSuggestionStatus;
};

export type SelectionSuggestionEvent =
  | { type: 'selection-changed'; key: string | null }
  | { type: 'attach' }
  | { type: 'dismiss' }
  | { type: 'clear' };

export function reduceSelectionSuggestion(
  state: SelectionSuggestionState,
  event: SelectionSuggestionEvent,
): SelectionSuggestionState {
  if (event.type === 'clear') return { snapshotKey: null, status: 'none' };
  if (event.type === 'attach') {
    return state.snapshotKey ? { ...state, status: 'attached' } : state;
  }
  if (event.type === 'dismiss') {
    return state.snapshotKey ? { ...state, status: 'dismissed' } : state;
  }
  if (!event.key) return { snapshotKey: null, status: 'none' };
  if (event.key === state.snapshotKey) return state;
  return { snapshotKey: event.key, status: 'suggested' };
}
```

- [ ] **Step 4: Create `AiComposer.svelte`**

Create `app/lib/components/ai-shared/AiComposer.svelte`:

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';
  import AiContextBar from './AiContextBar.svelte';
  import AiMentionMenu from './AiMentionMenu.svelte';
  import AiCommandMenu from './AiCommandMenu.svelte';
  import type { SlashCommandId } from './context';
  import type { AiContextAttachment } from './attachments';
  import { attachmentToContextItem } from './attachments';
  import { IconClose, IconDocument } from '../icons';

  type SuggestedSelection = {
    attachment: AiContextAttachment;
    status: 'suggested' | 'attached' | 'dismissed';
  } | null;

  type Props = {
    value: string;
    placeholder: string;
    attachments: readonly AiContextAttachment[];
    mentionVisible: boolean;
    mentionQuery: string;
    commandVisible: boolean;
    commandQuery: string;
    suggestedSelection?: SuggestedSelection;
    busy?: boolean;
    canSend: boolean;
    sendLabel?: string;
    onInput: (value: string) => void;
    onSend: () => void;
    onStop?: () => void;
    onPickMention: (token: string) => void;
    onPickCommand: (id: SlashCommandId) => void;
    onRemoveAttachment: (id: string) => void;
    onClearAttachments: () => void;
    onAttachSelection?: () => void;
    onDismissSelection?: () => void;
    onDropPaths?: (paths: string[]) => void;
    actions?: Snippet;
  };

  let {
    value,
    placeholder,
    attachments,
    mentionVisible,
    mentionQuery,
    commandVisible,
    commandQuery,
    suggestedSelection = null,
    busy = false,
    canSend,
    sendLabel = 'Send',
    onInput,
    onSend,
    onStop,
    onPickMention,
    onPickCommand,
    onRemoveAttachment,
    onClearAttachments,
    onAttachSelection,
    onDismissSelection,
    onDropPaths,
    actions,
  }: Props = $props();

  let contextItems = $derived(attachments.map(attachmentToContextItem));

  function keydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      if (busy) onStop?.();
      else if (canSend) onSend();
    }
  }

  function drop(e: DragEvent) {
    const raw = e.dataTransfer?.getData('text/plain') || '';
    const paths = raw.split('\n').map((p) => p.trim()).filter(Boolean);
    if (paths.length > 0) {
      e.preventDefault();
      onDropPaths?.(paths);
    }
  }
</script>

<div class="ai-composer" data-testid="ai-composer" ondrop={drop} ondragover={(e) => e.preventDefault()}>
  {#if suggestedSelection && suggestedSelection.status === 'suggested'}
    <div class="selection-suggestion" data-testid="ai-selection-suggestion">
      <button type="button" class="suggestion-main" onclick={onAttachSelection}>
        <IconDocument size={12} />
        <span>{suggestedSelection.attachment.label}</span>
      </button>
      <button type="button" class="suggestion-close" aria-label="Dismiss selection" onclick={onDismissSelection}>
        <IconClose size={12} />
      </button>
    </div>
  {/if}
  <AiContextBar
    items={contextItems}
    onRemove={onRemoveAttachment}
    onClear={onClearAttachments}
  />
  <AiCommandMenu visible={commandVisible} query={commandQuery} onPick={onPickCommand} />
  <AiMentionMenu visible={mentionVisible} query={mentionQuery} onPick={onPickMention} />
  <textarea
    rows="3"
    {placeholder}
    value={value}
    oninput={(e) => onInput(e.currentTarget.value)}
    onkeydown={keydown}
  ></textarea>
  <div class="composer-actions">
    {#if actions}
      {@render actions()}
    {/if}
    {#if busy}
      <button class="novelist-btn novelist-btn-primary" type="button" onclick={onStop}>Stop</button>
    {:else}
      <button class="novelist-btn novelist-btn-primary" type="button" onclick={onSend} disabled={!canSend}>{sendLabel}</button>
    {/if}
  </div>
</div>

<style>
  .ai-composer {
    border-top: 1px solid var(--novelist-border);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--novelist-bg-secondary);
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    background: var(--novelist-bg);
    border: 1px solid var(--novelist-border);
    color: var(--novelist-text);
    border-radius: 4px;
    padding: 6px 8px;
    font: inherit;
    resize: vertical;
  }
  .selection-suggestion {
    display: flex;
    align-items: center;
    border: 1px dashed color-mix(in srgb, var(--novelist-accent) 45%, var(--novelist-border));
    border-radius: 4px;
    background: color-mix(in srgb, var(--novelist-accent) 8%, var(--novelist-bg));
    overflow: hidden;
  }
  .suggestion-main {
    flex: 1;
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    border: 0;
    background: transparent;
    color: var(--novelist-text);
    padding: 4px 6px;
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }
  .suggestion-main span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .suggestion-close {
    display: inline-flex;
    align-items: center;
    border: 0;
    background: transparent;
    color: var(--novelist-text-secondary);
    padding: 4px 6px;
    cursor: pointer;
  }
  .composer-actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
</style>
```

- [ ] **Step 5: Run selection test and Svelte check**

Run:

```bash
pnpm vitest run --project unit tests/unit/components/ai-shared-selection-state.test.ts
pnpm check
```

Expected: unit test PASS and `svelte-check` exits 0.

- [ ] **Step 6: Commit Task 2**

Run:

```bash
git add app/lib/components/ai-shared/selection-state.ts app/lib/components/ai-shared/AiComposer.svelte tests/unit/components/ai-shared-selection-state.test.ts
git commit -m "feat(ai): add shared composer shell"
```

Expected: commit includes only Task 2 files.

## Task 3: Integrate Composer Into AI Agent and Clean Transcript

**Files:**
- Modify: `app/lib/components/ai-agent/sessions.svelte.ts`
- Modify: `app/lib/components/ai-agent/AiAgentImpl.svelte`
- Modify: `tests/unit/components/ai-agent-sessions.test.ts`
- Modify: `tests/e2e/specs/ai-panels.spec.ts`

- [ ] **Step 1: Write failing session-store test for clean user turns**

Append to `tests/unit/components/ai-agent-sessions.test.ts`:

```ts
it('keeps display text separate from outbound prompt metadata', () => {
  const id = aiAgentSessions.create();
  aiAgentSessions.updateTurns(id, [{
    role: 'user',
    text: '## Context 1: hidden\n\n## User request\nsummarize',
    displayText: 'summarize',
    attachments: [{ id: 'current:/p/a.md', label: 'Current file: a.md', kind: 'current-file' }],
  }]);
  expect(aiAgentSessions.active?.turns[0]).toMatchObject({
    role: 'user',
    text: '## Context 1: hidden\n\n## User request\nsummarize',
    displayText: 'summarize',
  });
  expect(aiAgentSessions.active?.title).toBe('summarize');
});
```

- [ ] **Step 2: Run test to verify failure**

Run: `pnpm vitest run --project unit tests/unit/components/ai-agent-sessions.test.ts`

Expected: FAIL because the `Turn` user type does not support `displayText` and title derivation uses `text`.

- [ ] **Step 3: Extend AI Agent user turn type**

Modify `app/lib/components/ai-agent/sessions.svelte.ts`:

```ts
export type TurnAttachmentMeta = {
  id: string;
  label: string;
  kind: string;
  path?: string;
};

export type UserTurn = {
  role: 'user';
  text: string;
  displayText?: string;
  attachments?: TurnAttachmentMeta[];
};

export type AssistantTurn = {
  role: 'assistant';
  text: string;
  cards: Card[];
  cost?: number;
};

export type Turn = UserTurn | AssistantTurn;
```

Update `deriveAgentTitle`:

```ts
export function deriveAgentTitle(turns: Turn[]): string {
  const firstUser = turns.find((t) => t.role === 'user') as UserTurn | undefined;
  if (!firstUser) return 'New agent';
  const first = (firstUser.displayText ?? firstUser.text).trim().split(/\n/)[0];
  return first.length > MAX_TITLE_LENGTH
    ? first.slice(0, MAX_TITLE_LENGTH - 1) + '…'
    : first || 'New agent';
}
```

- [ ] **Step 4: Run session-store test**

Run: `pnpm vitest run --project unit tests/unit/components/ai-agent-sessions.test.ts`

Expected: PASS.

- [ ] **Step 5: Integrate `AiComposer` in Agent**

In `app/lib/components/ai-agent/AiAgentImpl.svelte`:

1. Import:

```ts
import AiComposer from '$lib/components/ai-shared/AiComposer.svelte';
import {
  buildPromptFromAttachments,
  createAttachmentFromContext,
  displayTextFromInput,
  type AiContextAttachment,
} from '$lib/components/ai-shared/attachments';
import {
  reduceSelectionSuggestion,
  type SelectionSuggestionState,
} from '$lib/components/ai-shared/selection-state';
import { getSelectionContext } from '$lib/components/ai-shared/context';
```

2. Replace `contextItems` state with:

```ts
let attachments = $state<AiContextAttachment[]>([]);
let selectionState = $state<SelectionSuggestionState>({ snapshotKey: null, status: 'none' });
```

3. Add helpers:

```ts
function addAttachments(items: AiContextAttachment[]) {
  const seen = new Set(attachments.map((item) => item.id));
  attachments = [...attachments, ...items.filter((item) => !seen.has(item.id))];
}

function removeAttachment(id: string) {
  attachments = attachments.filter((item) => item.id !== id);
}

function clearAttachments() {
  attachments = [];
}

function attachSelectionSuggestion() {
  const item = getSelectionContext();
  if (!item) return;
  addAttachments([createAttachmentFromContext(item)]);
  selectionState = reduceSelectionSuggestion(selectionState, { type: 'attach' });
}
```

4. In `send()`, replace context packing:

```ts
const mentionContexts = await resolveMentionContexts(text);
const skillTokens = parseSkillTokens(text);
const skillContext = skillAssetsForTokens(skillTokens, promptAssets).map((skill) =>
  createAttachmentFromContext({
    id: `skill:${skill.id}`,
    kind: 'manual-note',
    label: `Skill: ${skill.name}`,
    path: skill.path,
    content: skill.content,
  }),
);
const mentionAttachments = mentionContexts.map(createAttachmentFromContext);
if (mentionAttachments.length > 0 || skillContext.length > 0) {
  addAttachments([...mentionAttachments, ...skillContext]);
}
const instruction = commandInstruction(slash);
const displayText = displayTextFromInput(slash ? slash.rest || text : text);
const userText = instruction
  ? `${instruction}\n\nUser request: ${displayText || slash?.rest || text}`
  : displayText || text;
const packed = buildPromptFromAttachments(userText, [...attachments, ...mentionAttachments, ...skillContext]);
const outbound = packed.outboundText;
aiAgentSessions.updateTurns(sessionId, [
  ...s.turns,
  {
    role: 'user',
    text: outbound,
    displayText: displayText || text,
    attachments: packed.attachments.map((item) => ({
      id: item.id,
      label: item.label,
      kind: item.kind,
      path: item.path,
    })),
  },
]);
```

5. In transcript rendering, replace user turn body with:

```svelte
<div class="text">{turn.displayText ?? turn.text}</div>
{#if turn.attachments?.length}
  <div class="turn-attachments">
    {#each turn.attachments as item (item.id)}
      <span>{item.label}</span>
    {/each}
  </div>
{/if}
```

6. Replace the composer block with:

```svelte
<AiComposer
  value={input}
  placeholder="Ask the agent... @current /plan $plot-doctor"
  attachments={attachments}
  mentionVisible={mentionMenuVisible}
  mentionQuery={mentionQuery}
  commandVisible={commandMenuVisible}
  commandQuery={commandQuery}
  canSend={Boolean(input.trim())}
  onInput={(value) => (input = value)}
  onSend={send}
  onStop={stopActiveSession}
  onPickMention={pickMention}
  onPickCommand={pickCommand}
  onRemoveAttachment={removeAttachment}
  onClearAttachments={clearAttachments}
  onAttachSelection={attachSelectionSuggestion}
  onDismissSelection={() => (selectionState = reduceSelectionSuggestion(selectionState, { type: 'dismiss' }))}
>
  {#snippet actions()}
    <button class="novelist-btn novelist-btn-ghost novelist-btn-sm" data-testid="ai-agent-mode-toggle" onclick={() => setActiveMode(agentMode === 'plan' ? 'act' : 'plan')}>
      {agentMode === 'plan' ? 'Act' : 'Plan'}
    </button>
    <button class="novelist-btn novelist-btn-ghost novelist-btn-sm" data-testid="ai-agent-more" title="More actions">More</button>
  {/snippet}
</AiComposer>
```

Then add a small CSS block for `.turn-attachments`.

- [ ] **Step 6: Add E2E assertion for clean Agent transcript**

Append to `tests/e2e/specs/ai-panels.spec.ts` under `AI Agent panel`:

```ts
test('Agent displays clean user text while sending attached context', async ({ app }) => {
  await app.evaluate(() => {
    (window as any).__TAURI_MOCK_STATE__.setClaudeCliDetectResult({
      path: '/opt/homebrew/bin/claude',
      version: '1.0.0',
    });
  });
  await enterProject(app);
  await app.getByText('Chapter 1', { exact: true }).click();
  await app.getByTestId('panel-toggle-ai-agent').click();

  await app.getByRole('textbox', { name: /Ask the agent/i }).fill('@current summarize this');
  await app.getByRole('button', { name: 'Send' }).click();

  await expect(app.getByTestId('ai-agent-panel')).toContainText('summarize this');
  await expect(app.getByTestId('ai-agent-panel')).not.toContainText('## Context 1');
});
```

- [ ] **Step 7: Run focused checks**

Run:

```bash
pnpm vitest run --project unit tests/unit/components/ai-agent-sessions.test.ts tests/unit/components/ai-shared-context.test.ts
pnpm test:e2e:browser -- ai-panels.spec.ts
pnpm check
```

Expected: all pass.

- [ ] **Step 8: Commit Task 3**

Run:

```bash
git add app/lib/components/ai-agent/sessions.svelte.ts app/lib/components/ai-agent/AiAgentImpl.svelte tests/unit/components/ai-agent-sessions.test.ts tests/e2e/specs/ai-panels.spec.ts
git commit -m "feat(ai): clean agent context transcript"
```

Expected: commit includes only Task 3 files.

## Task 4: Real Searchable `@` Picker

**Files:**
- Modify: `app/lib/components/ai-shared/attachments.ts`
- Modify: `app/lib/components/ai-shared/AiMentionMenu.svelte`
- Modify: `app/lib/components/ai-agent/AiAgentImpl.svelte`
- Modify: `app/lib/components/ai-talk/AiTalkImpl.svelte`
- Modify: `tests/unit/components/ai-shared-context.test.ts`
- Modify: `tests/e2e/specs/ai-panels.spec.ts`

- [ ] **Step 1: Write failing search-ranking unit test**

Append to `tests/unit/components/ai-shared-context.test.ts`:

```ts
it('ranks prefix matches before fuzzy path matches', () => {
  const candidates: AiContextAttachment[] = [
    {
      id: 'file:/project/Notes/outline.md',
      kind: 'project-file',
      label: 'outline.md',
      path: '/project/Notes/outline.md',
      source: 'project',
      mode: 'full',
      content: 'outline',
      estimatedChars: 7,
      truncated: false,
    },
    {
      id: 'file:/project/Chapter 1.md',
      kind: 'project-file',
      label: 'Chapter 1.md',
      path: '/project/Chapter 1.md',
      source: 'project',
      mode: 'full',
      content: 'chapter',
      estimatedChars: 7,
      truncated: false,
    },
  ];
  expect(searchAttachmentCandidates(candidates, 'chap').map((x) => x.label)).toEqual(['Chapter 1.md']);
});
```

- [ ] **Step 2: Run test to verify current behavior**

Run: `pnpm vitest run --project unit tests/unit/components/ai-shared-context.test.ts`

Expected: FAIL if search ordering or filtering is not strong enough for the new test.

- [ ] **Step 3: Extend `AiMentionMenu.svelte` to accept dynamic candidates**

Change props in `app/lib/components/ai-shared/AiMentionMenu.svelte`:

```ts
import type { AiContextAttachment } from './attachments';

type Mention = {
  token: string;
  label: string;
  hint: string;
  attachment?: AiContextAttachment;
};

type Props = {
  visible: boolean;
  query: string;
  candidates?: readonly AiContextAttachment[];
  onPick: (token: string, attachment?: AiContextAttachment) => void;
};
```

Build dynamic mentions:

```ts
const STATIC_MENTIONS: Mention[] = [
  { token: '@selection', label: '@selection', hint: 'selected editor text' },
  { token: '@current', label: '@current', hint: 'current file contents' },
  { token: '@outline', label: '@outline', hint: 'current file heading outline' },
];

let dynamicMentions = $derived((candidates ?? []).map((item) => ({
  token: `@${item.id}`,
  label: item.label,
  hint: item.path ?? item.kind,
  attachment: item,
})));
let filtered = $derived([...STATIC_MENTIONS, ...dynamicMentions]
  .filter((m) => `${m.label} ${m.hint}`.toLowerCase().includes(query.toLowerCase()))
  .slice(0, 8));
```

Update click handler:

```svelte
<button type="button" onclick={() => onPick(mention.token, mention.attachment)}>
```

- [ ] **Step 4: Build candidate lists in Agent and Talk**

In both `AiAgentImpl.svelte` and `AiTalkImpl.svelte`, derive candidates from:

```ts
let mentionCandidates = $derived([
  ...promptAssets.map((asset) => createAttachmentFromContext({
    id: `${asset.kind}:${asset.id}`,
    kind: 'manual-note',
    label: `${asset.kind === 'skill' ? 'Skill' : 'Command'}: ${asset.name}`,
    path: asset.path,
    content: asset.content,
  })),
  ...aiAgentSessions.sessions.map((session) => ({
    id: `session:${session.id}`,
    kind: 'session' as const,
    label: `Session: ${session.title}`,
    source: 'session' as const,
    mode: 'summary' as const,
    content: session.turns.map((turn) => turn.role === 'user' ? `USER: ${turn.displayText ?? turn.text}` : `ASSISTANT: ${turn.text}`).join('\n\n'),
    estimatedChars: session.title.length,
    truncated: false,
  })),
]);
```

For AI Talk, use `aiTalkSessions.sessions` and `messages`.

Update `pickMention` signatures:

```ts
function pickMention(token: string, attachment?: AiContextAttachment) {
  if (attachment) {
    addAttachments([attachment]);
    input = input.replace(/(^|\s)@[^\s]*$/, '$1');
    return;
  }
  input = input.replace(/(^|\s)@[^\s]*$/, `$1${token}`);
}
```

- [ ] **Step 5: Pass candidates through `AiComposer`**

Add `mentionCandidates?: readonly AiContextAttachment[]` to `AiComposer.svelte` props and pass it to `AiMentionMenu`:

```svelte
<AiMentionMenu
  visible={mentionVisible}
  query={mentionQuery}
  candidates={mentionCandidates}
  onPick={onPickMention}
/>
```

- [ ] **Step 6: Add E2E `@chap` attach test**

Append to `tests/e2e/specs/ai-panels.spec.ts`:

```ts
test('Agent @ picker attaches a project file by search', async ({ app }) => {
  await app.evaluate(() => {
    (window as any).__TAURI_MOCK_STATE__.setClaudeCliDetectResult({
      path: '/opt/homebrew/bin/claude',
      version: '1.0.0',
    });
  });
  await enterProject(app);
  await app.getByTestId('panel-toggle-ai-agent').click();

  await app.getByRole('textbox', { name: /Ask the agent/i }).fill('@chap');
  await app.getByRole('button', { name: /Chapter 1.md/ }).click();

  await expect(app.getByTestId('ai-context-bar')).toContainText('Chapter 1.md');
});
```

- [ ] **Step 7: Run checks**

Run:

```bash
pnpm vitest run --project unit tests/unit/components/ai-shared-context.test.ts
pnpm test:e2e:browser -- ai-panels.spec.ts
pnpm check
```

Expected: all pass.

- [ ] **Step 8: Commit Task 4**

Run:

```bash
git add app/lib/components/ai-shared/attachments.ts app/lib/components/ai-shared/AiMentionMenu.svelte app/lib/components/ai-shared/AiComposer.svelte app/lib/components/ai-agent/AiAgentImpl.svelte app/lib/components/ai-talk/AiTalkImpl.svelte tests/unit/components/ai-shared-context.test.ts tests/e2e/specs/ai-panels.spec.ts
git commit -m "feat(ai): add searchable context picker"
```

Expected: commit includes only Task 4 files.

## Task 5: Apply Changes Parser and Diff

**Files:**
- Create: `app/lib/components/ai-shared/apply-change-set.ts`
- Create: `tests/unit/components/ai-shared-apply-change-set.test.ts`

- [ ] **Step 1: Write failing Apply parser tests**

Create `tests/unit/components/ai-shared-apply-change-set.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import {
  buildLineDiff,
  parseChangeSetsFromText,
  validateFileChange,
} from '$lib/components/ai-shared/apply-change-set';

describe('[contract] AI Apply Changes parser', () => {
  it('parses a novelist-change-set fenced block', () => {
    const text = [
      'Here is the edit:',
      '```novelist-change-set',
      JSON.stringify({
        summary: 'Tighten opening',
        files: [{ path: '/project/Chapter 1.md', status: 'modify', proposedText: '# Chapter 1\n\nNew text' }],
      }),
      '```',
    ].join('\n');
    const sets = parseChangeSetsFromText(text, { sourceSessionId: 's1' });
    expect(sets).toHaveLength(1);
    expect(sets[0].summary).toBe('Tighten opening');
    expect(sets[0].files[0].path).toBe('/project/Chapter 1.md');
  });

  it('rejects invalid JSON blocks without throwing', () => {
    expect(parseChangeSetsFromText('```novelist-change-set\n{bad\n```', { sourceSessionId: 's1' })).toEqual([]);
  });

  it('builds CJK-safe line diffs', () => {
    const hunks = buildLineDiff('第一章\n风起\n', '第一章\n雨落\n');
    expect(hunks[0].lines).toEqual([
      { kind: 'context', text: '第一章' },
      { kind: 'removed', text: '风起' },
      { kind: 'added', text: '雨落' },
    ]);
  });

  it('detects stale original text before apply', () => {
    expect(validateFileChange({
      path: '/project/a.md',
      status: 'modify',
      originalText: 'old',
      proposedText: 'new',
      hunks: [],
    }, 'changed')).toEqual({ ok: false, reason: 'File changed since proposal was generated.' });
  });
});
```

- [ ] **Step 2: Run parser tests to verify failure**

Run: `pnpm vitest run --project unit tests/unit/components/ai-shared-apply-change-set.test.ts`

Expected: FAIL with module-not-found.

- [ ] **Step 3: Implement `apply-change-set.ts`**

Create `app/lib/components/ai-shared/apply-change-set.ts`:

```ts
export type AiDiffLine = { kind: 'context' | 'added' | 'removed'; text: string };

export type AiDiffHunk = {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: AiDiffLine[];
};

export type AiFileChange = {
  path: string;
  status: 'modify' | 'create';
  originalText: string | null;
  proposedText: string;
  hunks: AiDiffHunk[];
  conflict?: string;
};

export type AiChangeSet = {
  id: string;
  sourceSessionId: string;
  createdAt: string;
  summary: string;
  files: AiFileChange[];
};

type ParseOptions = {
  sourceSessionId: string;
};

type ValidationResult = { ok: true } | { ok: false; reason: string };

function uuid(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) return crypto.randomUUID();
  return `changes-${Date.now().toString(36)}-${Math.floor(Math.random() * 1e6).toString(36)}`;
}

function normalizeFile(raw: unknown): AiFileChange | null {
  if (!raw || typeof raw !== 'object') return null;
  const file = raw as Record<string, unknown>;
  if (typeof file.path !== 'string' || !file.path.trim()) return null;
  if (file.status !== 'modify' && file.status !== 'create') return null;
  if (typeof file.proposedText !== 'string') return null;
  const originalText = typeof file.originalText === 'string' ? file.originalText : null;
  return {
    path: file.path,
    status: file.status,
    originalText,
    proposedText: file.proposedText,
    hunks: Array.isArray(file.hunks) ? file.hunks as AiDiffHunk[] : buildLineDiff(originalText ?? '', file.proposedText),
  };
}

export function parseChangeSetsFromText(text: string, options: ParseOptions): AiChangeSet[] {
  const out: AiChangeSet[] = [];
  const re = /```novelist-change-set\s*([\s\S]*?)```/g;
  let match: RegExpExecArray | null;
  while ((match = re.exec(text))) {
    try {
      const raw = JSON.parse(match[1]) as Record<string, unknown>;
      const files = Array.isArray(raw.files) ? raw.files.map(normalizeFile).filter(Boolean) as AiFileChange[] : [];
      if (files.length === 0) continue;
      out.push({
        id: uuid(),
        sourceSessionId: options.sourceSessionId,
        createdAt: new Date().toISOString(),
        summary: typeof raw.summary === 'string' && raw.summary.trim() ? raw.summary : 'Proposed changes',
        files,
      });
    } catch {
      continue;
    }
  }
  return out;
}

export function buildLineDiff(original: string, proposed: string): AiDiffHunk[] {
  const a = original.replace(/\n$/, '').split('\n');
  const b = proposed.replace(/\n$/, '').split('\n');
  const lines: AiDiffLine[] = [];
  const max = Math.max(a.length, b.length);
  for (let i = 0; i < max; i++) {
    if (a[i] === b[i]) {
      if (a[i] !== undefined) lines.push({ kind: 'context', text: a[i] });
    } else {
      if (a[i] !== undefined) lines.push({ kind: 'removed', text: a[i] });
      if (b[i] !== undefined) lines.push({ kind: 'added', text: b[i] });
    }
  }
  return [{
    oldStart: 1,
    oldLines: a.length,
    newStart: 1,
    newLines: b.length,
    lines,
  }];
}

export function validateFileChange(change: AiFileChange, latestText: string | null): ValidationResult {
  if (change.status === 'modify' && latestText == null) {
    return { ok: false, reason: 'Target file does not exist.' };
  }
  if (change.status === 'create' && latestText != null) {
    return { ok: false, reason: 'Target file already exists.' };
  }
  if (change.originalText != null && latestText != null && change.originalText !== latestText) {
    return { ok: false, reason: 'File changed since proposal was generated.' };
  }
  return { ok: true };
}
```

- [ ] **Step 4: Run parser tests**

Run: `pnpm vitest run --project unit tests/unit/components/ai-shared-apply-change-set.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit Task 5**

Run:

```bash
git add app/lib/components/ai-shared/apply-change-set.ts tests/unit/components/ai-shared-apply-change-set.test.ts
git commit -m "feat(ai): parse apply change sets"
```

Expected: commit includes only Task 5 files.

## Task 6: Apply Changes Card and Agent Integration

**Files:**
- Create: `app/lib/components/ai-shared/ApplyChangesCard.svelte`
- Modify: `app/lib/components/ai-agent/sessions.svelte.ts`
- Modify: `app/lib/components/ai-agent/AiAgentImpl.svelte`
- Modify: `tests/e2e/specs/ai-panels.spec.ts`

- [ ] **Step 1: Extend Agent card types**

In `app/lib/components/ai-agent/sessions.svelte.ts`, import type:

```ts
import type { AiChangeSet } from '$lib/components/ai-shared/apply-change-set';
```

Change card types:

```ts
export type ApplyChangesCard = { kind: 'apply-changes'; changeSet: AiChangeSet; status?: 'pending' | 'accepted' | 'rejected' | 'conflict' };
export type Card = ToolCard | ToolResultCard | ApplyChangesCard;
```

- [ ] **Step 2: Create `ApplyChangesCard.svelte`**

Create `app/lib/components/ai-shared/ApplyChangesCard.svelte`:

```svelte
<script lang="ts">
  import type { AiChangeSet, AiFileChange } from './apply-change-set';

  type Props = {
    changeSet: AiChangeSet;
    status?: 'pending' | 'accepted' | 'rejected' | 'conflict';
    onAcceptFile: (file: AiFileChange) => void;
    onRejectFile: (file: AiFileChange) => void;
    onAcceptAll: () => void;
    onRejectAll: () => void;
  };

  let { changeSet, status = 'pending', onAcceptFile, onRejectFile, onAcceptAll, onRejectAll }: Props = $props();
</script>

<section class="apply-card" data-testid="ai-apply-changes-card">
  <header>
    <div>
      <strong>Apply Changes</strong>
      <span>{changeSet.summary}</span>
    </div>
    <em>{status}</em>
  </header>
  <div class="files">
    {#each changeSet.files as file (file.path)}
      <details open>
        <summary>
          <span>{file.status}</span>
          <code>{file.path}</code>
        </summary>
        {#each file.hunks as hunk}
          <pre>{#each hunk.lines as line}{line.kind === 'added' ? '+' : line.kind === 'removed' ? '-' : ' '}{line.text}
{/each}</pre>
        {/each}
        {#if file.conflict}
          <p class="conflict">{file.conflict}</p>
        {/if}
        <div class="file-actions">
          <button class="novelist-btn novelist-btn-primary novelist-btn-sm" type="button" onclick={() => onAcceptFile(file)}>Accept file</button>
          <button class="novelist-btn novelist-btn-ghost novelist-btn-sm" type="button" onclick={() => onRejectFile(file)}>Reject file</button>
        </div>
      </details>
    {/each}
  </div>
  <footer>
    <button class="novelist-btn novelist-btn-primary novelist-btn-sm" type="button" onclick={onAcceptAll}>Accept all</button>
    <button class="novelist-btn novelist-btn-ghost novelist-btn-sm" type="button" onclick={onRejectAll}>Reject all</button>
  </footer>
</section>

<style>
  .apply-card {
    border: 1px solid var(--novelist-border);
    border-radius: 4px;
    background: var(--novelist-bg-secondary);
    overflow: hidden;
  }
  header,
  footer,
  .file-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
  }
  header {
    justify-content: space-between;
    border-bottom: 1px solid var(--novelist-border);
  }
  header div {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  header span,
  header em {
    color: var(--novelist-text-secondary);
    font-size: 11px;
    font-style: normal;
  }
  details {
    border-bottom: 1px solid var(--novelist-border);
  }
  summary {
    cursor: pointer;
    padding: 5px 8px;
    font-size: 11px;
  }
  code {
    color: var(--novelist-text);
  }
  pre {
    margin: 0;
    padding: 6px 8px;
    max-height: 220px;
    overflow: auto;
    white-space: pre-wrap;
    background: var(--novelist-bg);
    font-size: 11px;
  }
  .conflict {
    margin: 6px 8px;
    color: #b45309;
    font-size: 11px;
  }
  footer {
    justify-content: flex-end;
  }
</style>
```

- [ ] **Step 3: Parse change sets in Agent result handling**

In `app/lib/components/ai-agent/AiAgentImpl.svelte`, import:

```ts
import ApplyChangesCard from '$lib/components/ai-shared/ApplyChangesCard.svelte';
import {
  parseChangeSetsFromText,
  validateFileChange,
  type AiFileChange,
} from '$lib/components/ai-shared/apply-change-set';
```

In `applyResult`, after computing `finalText`, parse:

```ts
const changeSets = parseChangeSetsFromText(finalText, { sourceSessionId: sessionId });
const cards = changeSets.length > 0
  ? [...cur.cards, ...changeSets.map((changeSet) => ({ kind: 'apply-changes' as const, changeSet }))]
  : cur.cards;
base[idx] = { ...cur, text: finalText, cards, cost };
```

When rendering cards, add a branch:

```svelte
{:else if c.kind === 'apply-changes'}
  <ApplyChangesCard
    changeSet={c.changeSet}
    status={c.status}
    onAcceptFile={(file) => acceptApplyFile(c.changeSet.id, file)}
    onRejectFile={(file) => rejectApplyFile(c.changeSet.id, file)}
    onAcceptAll={() => acceptApplyAll(c.changeSet.id)}
    onRejectAll={() => rejectApplyAll(c.changeSet.id)}
  />
```

- [ ] **Step 4: Implement apply handlers**

In `AiAgentImpl.svelte`, add:

```ts
async function latestFileText(path: string): Promise<string | null> {
  const result = await commands.readFile(path);
  return result.status === 'ok' ? result.data : null;
}

async function writeFileText(path: string, text: string): Promise<void> {
  const result = await commands.writeFile(path, text);
  if (result.status === 'error') throw new Error(result.error);
}

async function acceptApplyFile(_changeSetId: string, file: AiFileChange) {
  const latest = await latestFileText(file.path);
  const valid = validateFileChange(file, latest);
  if (!valid.ok) {
    file.conflict = valid.reason;
    return;
  }
  await writeFileText(file.path, file.proposedText);
}

async function rejectApplyFile(_changeSetId: string, _file: AiFileChange) {
  // File-level reject is represented by leaving file untouched in V1.
}

async function acceptApplyAll(changeSetId: string) {
  const session = activeSession;
  const applyCards = session?.turns.flatMap((turn) => turn.role === 'assistant' ? turn.cards : [])
    .filter((card) => card.kind === 'apply-changes' && card.changeSet.id === changeSetId) ?? [];
  for (const card of applyCards) {
    for (const file of card.changeSet.files) await acceptApplyFile(changeSetId, file);
  }
}

function rejectApplyAll(_changeSetId: string) {
  // Reject all leaves all target files untouched in V1.
}
```

If TypeScript requires immutable card updates, replace direct `file.conflict` mutation with a store update helper before running `pnpm check`.

- [ ] **Step 5: Add E2E Apply card test**

Append to `tests/e2e/specs/ai-panels.spec.ts`:

```ts
test('Agent renders Apply Changes card from a structured result', async ({ app }) => {
  await app.evaluate(() => {
    (window as any).__TAURI_MOCK_STATE__.setClaudeCliDetectResult({
      path: '/opt/homebrew/bin/claude',
      version: '1.0.0',
    });
  });
  await enterProject(app);
  await app.getByTestId('panel-toggle-ai-agent').click();
  await app.getByRole('textbox', { name: /Ask the agent/i }).fill('tighten chapter');
  await app.getByRole('button', { name: 'Send' }).click();

  const uuid = await app.evaluate(() => {
    const stored = JSON.parse(localStorage.getItem('novelist:ai-agent:sessions:v1') || '[]');
    return stored[0]?.sessionUuid;
  });
  await app.evaluate((sessionId) => {
    const payload = {
      type: 'result',
      subtype: 'success',
      result: [
        'Done.',
        '```novelist-change-set',
        JSON.stringify({
          summary: 'Tighten Chapter 1',
          files: [{
            path: '/tmp/novelist-test-project/Chapter 1.md',
            status: 'modify',
            originalText: '# Chapter 1\\n\\nIt was a dark and stormy night.\\n\\nThe wind howled through the trees.\\n',
            proposedText: '# Chapter 1\\n\\nNight pressed against the windows.\\n\\nThe wind worried the trees.\\n',
          }],
        }),
        '```',
      ].join('\\n'),
    };
    (window as any).__TAURI_MOCK_STATE__.emitClaudeStdout(sessionId, JSON.stringify(payload));
  }, uuid);

  await expect(app.getByTestId('ai-apply-changes-card')).toBeVisible();
  await expect(app.getByTestId('ai-apply-changes-card')).toContainText('Tighten Chapter 1');
});
```

- [ ] **Step 6: Run checks**

Run:

```bash
pnpm vitest run --project unit tests/unit/components/ai-shared-apply-change-set.test.ts tests/unit/components/ai-agent-sessions.test.ts
pnpm test:e2e:browser -- ai-panels.spec.ts
pnpm check
```

Expected: all pass.

- [ ] **Step 7: Commit Task 6**

Run:

```bash
git add app/lib/components/ai-shared/ApplyChangesCard.svelte app/lib/components/ai-shared/apply-change-set.ts app/lib/components/ai-agent/sessions.svelte.ts app/lib/components/ai-agent/AiAgentImpl.svelte tests/e2e/specs/ai-panels.spec.ts
git commit -m "feat(ai): review apply change sets"
```

Expected: commit includes only Task 6 files.

## Task 7: Final Verification and Documentation Check

**Files:**
- Modify only if verification reveals documentation drift.

- [ ] **Step 1: Run full relevant test suite**

Run:

```bash
pnpm vitest run --project unit tests/unit/components/ai-shared-context.test.ts tests/unit/components/ai-shared-selection-state.test.ts tests/unit/components/ai-shared-apply-change-set.test.ts tests/unit/components/ai-agent-sessions.test.ts tests/unit/components/ai-talk-sessions.test.ts
pnpm test:e2e:browser -- ai-panels.spec.ts
pnpm check
```

Expected: all commands exit 0.

- [ ] **Step 2: Run broad quick verification**

Run: `pnpm verify:quick`

Expected: exits 0. If existing unrelated dirty files cause failures, record exact failures and run the narrow passing commands above before final response.

- [ ] **Step 3: Inspect diff scope**

Run:

```bash
git status --short
git diff --stat
```

Expected: only AI-related source/tests and the plan are changed by this work; pre-existing unrelated files stay unstaged unless the user explicitly asks to include them.

- [ ] **Step 4: Commit final cleanup if needed**

If Task 7 changed docs or minor fixes:

```bash
git add <task-scoped-files>
git commit -m "chore(ai): verify agent ux changes"
```

Expected: final cleanup commit is scoped and does not include unrelated pre-existing changes.
