/**
 * Settings search index + matcher.
 *
 * Each entry is a curated pointer to a settable thing in `Settings.svelte`.
 * Entries reference i18n keys (so the displayed label / hint stays in sync
 * with the locale) and an optional `anchor` — a `data-testid` that the UI
 * can scrollIntoView after switching sections.
 *
 * The index is intentionally curated rather than auto-extracted: keeps
 * search results to "actual settings the user wants to find", not every
 * sub-label, header, or button text. Add entries when you ship a new
 * setting; the entries below cover everything visible in v0.2.4.
 */

export interface SearchableSetting {
  /** Stable id, also serves as React-style key for result rendering. */
  id: string;
  /** Section to switch to when the user picks this result. */
  sectionId: string;
  /** i18n key for the section label (used in result group headers). */
  sectionLabelKey: string;
  /** i18n key for the displayed label. */
  labelKey: string;
  /** Optional secondary text for matching/displaying (e.g. setting hint). */
  hintKey?: string;
  /** Optional scroll anchor — a `data-testid` to scroll into view. */
  anchor?: string;
  /** Extra search terms not already covered by label/hint translations. */
  keywords?: string[];
}

export const SETTINGS_SEARCH_INDEX: SearchableSetting[] = [
  // Editor
  { id: 'editor.language', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.language' },
  { id: 'editor.font', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.font' },
  { id: 'editor.size', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.size' },
  { id: 'editor.lineHeight', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.lineHeight' },
  { id: 'editor.width', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.width' },
  { id: 'editor.autoSave', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.autoSave' },
  { id: 'editor.tabIndent', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.tabIndent' },
  { id: 'editor.highlightMatches', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.highlightMatches' },
  { id: 'editor.renderImages', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.renderImages' },
  // Editor → New File
  { id: 'newfile.detect', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.editor.newFile.detectFromFolder', hintKey: 'settings.editor.newFile.detectFromFolderHint', anchor: 'settings-newfile-detect' },
  { id: 'newfile.template', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.editor.newFile.template', hintKey: 'settings.editor.newFile.templateHint', anchor: 'settings-newfile-template', keywords: ['macro', 'date', 'datetime', '宏', '日期'] },
  { id: 'newfile.autoRename', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.editor.newFile.autoRename', hintKey: 'settings.editor.newFile.autoRenameHint', anchor: 'settings-newfile-autorename', keywords: ['h1', 'rename', '重命名'] },
  { id: 'newfile.defaultDir', sectionId: 'editor', sectionLabelKey: 'settings.editor', labelKey: 'settings.editor.newFile.defaultDir', hintKey: 'settings.editor.newFile.defaultDirHint', anchor: 'settings-newfile-defaultdir' },

  // Theme
  { id: 'theme.mode', sectionId: 'theme', sectionLabelKey: 'settings.theme', labelKey: 'settings.theme', keywords: ['dark', 'light', '深色', '浅色'] },
  { id: 'theme.importTypora', sectionId: 'theme', sectionLabelKey: 'settings.theme', labelKey: 'settings.theme.importTypora' },

  // Shortcuts
  { id: 'shortcuts.list', sectionId: 'shortcuts', sectionLabelKey: 'settings.shortcuts', labelKey: 'settings.shortcuts', keywords: ['keybinding', 'hotkey', '快捷键'] },

  // Templates
  { id: 'templates.list', sectionId: 'templates', sectionLabelKey: 'settings.templates', labelKey: 'settings.templates' },

  // Plugins
  { id: 'plugins.list', sectionId: 'plugins', sectionLabelKey: 'settings.plugins', labelKey: 'settings.plugins' },

  // Image hosts
  { id: 'imageHosts.list', sectionId: 'image-hosts', sectionLabelKey: 'settings.imageHosts', labelKey: 'settings.imageHosts', keywords: ['s3', 'oss', 'qiniu', 'r2', 'imgur', 'sm.ms', '图床'] },

  // Publish
  { id: 'publish.list', sectionId: 'publish', sectionLabelKey: 'settings.publish', labelKey: 'settings.publish', keywords: ['ghost', 'wordpress', 'medium', '发布'] },

  // Sync
  { id: 'sync.list', sectionId: 'sync', sectionLabelKey: 'settings.sync', labelKey: 'settings.sync', keywords: ['webdav', 'snapshot', '同步', '快照'] },
];

type Translator = (key: string) => string;

/**
 * Filter the index for the given query. Empty / whitespace query → empty
 * result list (caller falls back to the normal nav).
 *
 * Match rules:
 *  - Lowercased substring across all of: translated label, translated hint,
 *    translated section label, raw keywords, raw id.
 *  - Multi-token query (whitespace-separated): every token must be found
 *    in *some* searchable field — token AND, field OR. CJK is matched as
 *    contiguous substrings (no word-segmentation).
 */
export function filterSettingsIndex(
  index: SearchableSetting[],
  query: string,
  t: Translator,
): SearchableSetting[] {
  const q = query.trim();
  if (!q) return [];
  const tokens = q.toLowerCase().split(/\s+/).filter(Boolean);
  if (tokens.length === 0) return [];

  return index.filter((entry) => {
    const haystack = [
      t(entry.labelKey),
      entry.hintKey ? t(entry.hintKey) : '',
      t(entry.sectionLabelKey),
      ...(entry.keywords ?? []),
      entry.id,
    ]
      .join(' \n ')
      .toLowerCase();
    return tokens.every((tok) => haystack.includes(tok));
  });
}

/**
 * Group search results by section, preserving the index's original order
 * within each section. Result is a stable array of `[sectionLabelKey, items]`
 * pairs so the renderer can `{#each}` over it.
 */
export function groupBySection(
  matches: SearchableSetting[],
): Array<{ sectionId: string; sectionLabelKey: string; items: SearchableSetting[] }> {
  const seen = new Map<string, { sectionId: string; sectionLabelKey: string; items: SearchableSetting[] }>();
  for (const m of matches) {
    const cur = seen.get(m.sectionId);
    if (cur) cur.items.push(m);
    else seen.set(m.sectionId, { sectionId: m.sectionId, sectionLabelKey: m.sectionLabelKey, items: [m] });
  }
  return Array.from(seen.values());
}
