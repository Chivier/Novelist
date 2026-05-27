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
