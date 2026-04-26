import { describe, expect, it } from 'vitest';
import { buildWordDiff } from '$lib/components/ai-shared/diff';

describe('[precision] AI inline edit diff', () => {
  it('keeps unchanged text and marks additions/removals', () => {
    expect(buildWordDiff('The old line', 'The new line')).toEqual([
      { kind: 'same', text: 'The ' },
      { kind: 'removed', text: 'old' },
      { kind: 'added', text: 'new' },
      { kind: 'same', text: ' line' },
    ]);
  });

  it('handles CJK text as non-whitespace tokens', () => {
    expect(buildWordDiff('第一章 风起', '第一章 雨落')).toEqual([
      { kind: 'same', text: '第一章 ' },
      { kind: 'removed', text: '风起' },
      { kind: 'added', text: '雨落' },
    ]);
  });
});
