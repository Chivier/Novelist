import { describe, it, expect } from 'vitest';
import {
  filterSettingsIndex,
  groupBySection,
  type SearchableSetting,
} from '$lib/utils/settings-search';

const idx: SearchableSetting[] = [
  { id: 'editor.font', sectionId: 'editor', sectionLabelKey: 'sec.editor', labelKey: 'lbl.font' },
  { id: 'editor.size', sectionId: 'editor', sectionLabelKey: 'sec.editor', labelKey: 'lbl.size' },
  { id: 'editor.autosave', sectionId: 'editor', sectionLabelKey: 'sec.editor', labelKey: 'lbl.autosave', hintKey: 'hint.autosave' },
  { id: 'theme.mode', sectionId: 'theme', sectionLabelKey: 'sec.theme', labelKey: 'lbl.theme', keywords: ['dark', '深色'] },
  { id: 'newfile.template', sectionId: 'editor', sectionLabelKey: 'sec.editor', labelKey: 'lbl.newfile.tpl', keywords: ['macro', '宏'] },
];

const t = (key: string): string => {
  const dict: Record<string, string> = {
    'sec.editor': 'Editor',
    'sec.theme': 'Theme',
    'lbl.font': 'Font family',
    'lbl.size': 'Font size',
    'lbl.autosave': 'Auto-save',
    'hint.autosave': 'Save the file every N minutes',
    'lbl.theme': 'Theme mode',
    'lbl.newfile.tpl': 'Default filename template',
  };
  return dict[key] ?? key;
};

describe('filterSettingsIndex', () => {
  it('returns empty for empty / whitespace query', () => {
    expect(filterSettingsIndex(idx, '', t)).toEqual([]);
    expect(filterSettingsIndex(idx, '   ', t)).toEqual([]);
  });

  it('matches by translated label substring (case-insensitive)', () => {
    const r = filterSettingsIndex(idx, 'font', t);
    expect(r.map(e => e.id)).toEqual(['editor.font', 'editor.size']);
  });

  it('matches by translated hint', () => {
    const r = filterSettingsIndex(idx, 'every', t);
    expect(r.map(e => e.id)).toEqual(['editor.autosave']);
  });

  it('matches by translated section label', () => {
    const r = filterSettingsIndex(idx, 'theme', t);
    expect(r.map(e => e.id)).toContain('theme.mode');
  });

  it('matches by raw English keyword when label is in another language', () => {
    const r = filterSettingsIndex(idx, 'macro', t);
    expect(r.map(e => e.id)).toEqual(['newfile.template']);
  });

  it('matches CJK keyword', () => {
    const r = filterSettingsIndex(idx, '宏', t);
    expect(r.map(e => e.id)).toEqual(['newfile.template']);
  });

  it('multi-token query is AND across tokens, OR across fields', () => {
    // "font size" → must match both tokens; only "Font size" entry has both.
    const r = filterSettingsIndex(idx, 'font size', t);
    expect(r.map(e => e.id)).toEqual(['editor.size']);
  });

  it('no-match query returns empty', () => {
    expect(filterSettingsIndex(idx, 'unknownXYZ', t)).toEqual([]);
  });
});

describe('groupBySection', () => {
  it('preserves first-seen order across sections and within sections', () => {
    const matches = filterSettingsIndex(idx, '', t);
    expect(groupBySection(matches)).toEqual([]);

    const all = filterSettingsIndex(idx, 'e', t); // matches plenty
    const groups = groupBySection(all);
    // First section seen is 'editor' (index order); 'theme' comes second.
    expect(groups[0].sectionId).toBe('editor');
  });

  it('puts items into their respective sections', () => {
    const all = filterSettingsIndex(idx, 'theme', t);
    const groups = groupBySection(all);
    expect(groups).toHaveLength(1);
    expect(groups[0].sectionId).toBe('theme');
    expect(groups[0].items.map(i => i.id)).toEqual(['theme.mode']);
  });
});
