import { describe, it, expect } from 'vitest';
import { parseTemplate, renderTemplate, isPlaceholder, inferNextName, renameFromH1 } from '$lib/utils/placeholder';

describe('parseTemplate', () => {
  it('parses Untitled {N}', () => {
    const t = parseTemplate('Untitled {N}');
    expect(t).toEqual({
      raw: 'Untitled {N}',
      prefix: 'Untitled ',
      suffix: '',
      hasTitleSlot: false,
      titleSlotPosition: null,
      forceStyle: null,
    });
  });
  it('parses 第{N}章', () => {
    const t = parseTemplate('第{N}章');
    expect(t?.prefix).toBe('第');
    expect(t?.suffix).toBe('章');
    expect(t?.hasTitleSlot).toBe(false);
  });
  it('parses Chapter {N}', () => {
    const t = parseTemplate('Chapter {N}');
    expect(t?.prefix).toBe('Chapter ');
    expect(t?.suffix).toBe('');
  });
  it('parses {N}-{title}', () => {
    const t = parseTemplate('{N}-{title}');
    expect(t?.prefix).toBe('');
    expect(t?.suffix).toBe('-{title}');
    expect(t?.hasTitleSlot).toBe(true);
    expect(t?.titleSlotPosition).toBe('after');
  });
  it('rejects template missing {N}', () => {
    expect(parseTemplate('no number')).toBeNull();
  });
  it('rejects template with multiple {N}', () => {
    expect(parseTemplate('{N}-{N}')).toBeNull();
  });
  it('rejects empty', () => {
    expect(parseTemplate('')).toBeNull();
  });

  // Extended placeholder tokens for forced style / padding ({3N}, {cN}, {CN}, {rN}).
  it('parses {2N} as Arabic width 2', () => {
    const t = parseTemplate('{2N}-{title}');
    expect(t?.forceStyle).toEqual({ kind: 'arabic', width: 2 });
  });
  it('parses {3N} as Arabic width 3', () => {
    const t = parseTemplate('Chapter {3N}');
    expect(t?.forceStyle).toEqual({ kind: 'arabic', width: 3 });
    expect(t?.prefix).toBe('Chapter ');
    expect(t?.suffix).toBe('');
  });
  it('parses {cN} as Chinese-lower forced', () => {
    const t = parseTemplate('{cN}. {title}');
    expect(t?.forceStyle).toEqual({ kind: 'chinese-lower' });
  });
  it('parses {CN} as Chinese-upper forced', () => {
    expect(parseTemplate('{CN} 篇')?.forceStyle).toEqual({ kind: 'chinese-upper' });
  });
  it('parses {rN} as Roman forced', () => {
    expect(parseTemplate('Part {rN}')?.forceStyle).toEqual({ kind: 'roman-upper' });
  });
  it('bare {N} has forceStyle null (natural)', () => {
    expect(parseTemplate('第{N}章')?.forceStyle).toBeNull();
  });
});

describe('renderTemplate', () => {
  it('renders Untitled 1.md', () => {
    const t = parseTemplate('Untitled {N}')!;
    expect(renderTemplate(t, 1, { kind: 'arabic', width: 1 }, null)).toBe('Untitled 1.md');
  });
  it('renders 第一章.md (chinese-lower)', () => {
    const t = parseTemplate('第{N}章')!;
    expect(renderTemplate(t, 1, { kind: 'chinese-lower' }, null)).toBe('第一章.md');
  });
  it('renders 03-Untitled.md when title slot present', () => {
    const t = parseTemplate('{N}-{title}')!;
    expect(renderTemplate(t, 3, { kind: 'arabic', width: 2 }, null)).toBe('03-Untitled.md');
  });
  it('substitutes title slot when title given', () => {
    const t = parseTemplate('{N}-{title}')!;
    expect(renderTemplate(t, 3, { kind: 'arabic', width: 2 }, '开篇')).toBe('03-开篇.md');
  });
  it('appends title with space when no slot (chapter prefix)', () => {
    const t = parseTemplate('第{N}章')!;
    expect(renderTemplate(t, 3, { kind: 'chinese-lower' }, '开篇')).toBe('第三章 开篇.md');
  });
  it('appends title without space when stem ends in closing bracket', () => {
    const t = parseTemplate('【第{N}章】')!;
    expect(renderTemplate(t, 3, { kind: 'chinese-lower' }, '开篇')).toBe('【第三章】开篇.md');
  });
  it('Untitled {N} replaces full name when title given', () => {
    const t = parseTemplate('Untitled {N}')!;
    expect(renderTemplate(t, 1, { kind: 'arabic', width: 1 }, '开篇')).toBe('开篇.md');
  });
});

describe('isPlaceholder', () => {
  it('detects Untitled N', () => {
    expect(isPlaceholder('Untitled 1.md')).toBe(true);
    expect(isPlaceholder('Untitled 42.md')).toBe(true);
  });
  it('detects 第N章 (chinese-lower or arabic)', () => {
    expect(isPlaceholder('第三章.md')).toBe(true);
    expect(isPlaceholder('第3章.md')).toBe(true);
  });
  it('detects Chapter N', () => {
    expect(isPlaceholder('Chapter 3.md')).toBe(true);
  });
  it('detects {N}-Untitled', () => {
    expect(isPlaceholder('03-Untitled.md')).toBe(true);
    expect(isPlaceholder('03_Untitled.md')).toBe(true);
    expect(isPlaceholder('3.Untitled.md')).toBe(true);
  });
  it('detects legacy novelist_scratch_<ts>', () => {
    expect(isPlaceholder('novelist_scratch_1234567890.md')).toBe(true);
  });
  it('rejects user-named files', () => {
    expect(isPlaceholder('开篇.md')).toBe(false);
    expect(isPlaceholder('Hello World.md')).toBe(false);
    expect(isPlaceholder('第三章 开篇.md')).toBe(false);
    expect(isPlaceholder('03-开篇.md')).toBe(false);
  });
});

describe('inferNextName', () => {
  const defaultTemplate = parseTemplate('Untitled {N}')!;

  it('empty folder uses default template', () => {
    expect(inferNextName([], defaultTemplate)).toBe('Untitled 1.md');
  });

  it('user template applied to empty folder', () => {
    const t = parseTemplate('第{N}章')!;
    expect(inferNextName([], t)).toBe('第一章.md');
  });

  it('infers next chapter from chinese-lower series (≥2 matches)', () => {
    expect(inferNextName(['第一章.md', '第二章.md'], defaultTemplate)).toBe('第三章.md');
  });

  it('infers next chapter from arabic series', () => {
    expect(inferNextName(['Chapter 1.md', 'Chapter 2.md', 'Chapter 5.md'], defaultTemplate))
      .toBe('Chapter 6.md');
  });

  it('preserves zero-padding width', () => {
    expect(inferNextName(['01-intro.md', '02-rising.md'], defaultTemplate))
      .toBe('03-Untitled.md');
  });

  it('skips serial members (序章, 楔子)', () => {
    expect(inferNextName(['序章.md', '第一章.md', '第二章.md'], defaultTemplate))
      .toBe('第三章.md');
  });

  it('falls back to default template when only 1 match (auto threshold = 2)', () => {
    expect(inferNextName(['第一章.md', 'notes.md'], defaultTemplate)).toBe('Untitled 1.md');
  });

  it('user template lowers threshold to 1', () => {
    const t = parseTemplate('第{N}章')!;
    expect(inferNextName(['第一章.md', 'notes.md'], t)).toBe('第二章.md');
  });

  it('{3N} template renders zero-padded width 3 on empty folder', () => {
    const t = parseTemplate('{3N}-{title}')!;
    expect(inferNextName([], t)).toBe('001-Untitled.md');
  });

  it('{3N} template continues series from existing files', () => {
    const t = parseTemplate('{3N}-{title}')!;
    expect(inferNextName(['001-intro.md', '002-rising.md'], t)).toBe('003-Untitled.md');
  });

  it('{2N} template: 09 → 10 keeps width', () => {
    const t = parseTemplate('{2N}-{title}')!;
    expect(inferNextName(['08-foo.md', '09-bar.md'], t)).toBe('10-Untitled.md');
  });

  it('{cN} template renders Chinese-lower regardless of folder style', () => {
    const t = parseTemplate('{cN}. {title}')!;
    expect(inferNextName([], t)).toBe('一. Untitled.md');
    // Even with an Arabic-style sibling the forceStyle wins for new names
    expect(inferNextName(['1. foo.md'], t)).toBe('二. Untitled.md');
  });

  it('{rN} template renders Roman', () => {
    const t = parseTemplate('Chapter {rN}')!;
    expect(inferNextName([], t)).toBe('Chapter I.md');
    expect(inferNextName(['Chapter I.md', 'Chapter II.md'], t)).toBe('Chapter III.md');
  });

  it('avoids collision by bumping number', () => {
    expect(inferNextName(['第一章.md', '第二章.md', '第三章.md'], defaultTemplate))
      .toBe('第四章.md');
  });

  it('Untitled fallback bumps number on collision', () => {
    expect(inferNextName(['Untitled 1.md', 'Untitled 2.md'], defaultTemplate))
      .toBe('Untitled 3.md');
  });
});

describe('renameFromH1', () => {
  it('Untitled 1.md → 开篇.md', () => {
    expect(renameFromH1('Untitled 1.md', '开篇', [])).toBe('开篇.md');
  });
  it('returns null when filename is not a placeholder', () => {
    expect(renameFromH1('开篇.md', 'NewTitle', [])).toBeNull();
  });
  it('returns null when H1 is empty after sanitization', () => {
    expect(renameFromH1('Untitled 1.md', '   ', [])).toBeNull();
    expect(renameFromH1('Untitled 1.md', '', [])).toBeNull();
  });
  it('第三章.md + 开篇 → 第三章 开篇.md', () => {
    expect(renameFromH1('第三章.md', '开篇', [])).toBe('第三章 开篇.md');
  });
  it('Chapter 3.md + Opening → Chapter 3 Opening.md', () => {
    expect(renameFromH1('Chapter 3.md', 'Opening', [])).toBe('Chapter 3 Opening.md');
  });
  it('03-Untitled.md + 开篇 → 03-开篇.md', () => {
    expect(renameFromH1('03-Untitled.md', '开篇', [])).toBe('03-开篇.md');
  });
  it('03_Untitled.md + Opening → 03_Opening.md', () => {
    expect(renameFromH1('03_Untitled.md', 'Opening', [])).toBe('03_Opening.md');
  });
  it('legacy novelist_scratch → uses H1 as full name', () => {
    expect(renameFromH1('novelist_scratch_1234.md', '开篇', [])).toBe('开篇.md');
  });
  it('collision bumps with " 2"', () => {
    expect(renameFromH1('Untitled 1.md', '开篇', ['开篇.md'])).toBe('开篇 2.md');
    expect(renameFromH1('Untitled 1.md', '开篇', ['开篇.md', '开篇 2.md'])).toBe('开篇 3.md');
  });
  it('non-placeholder returns null even with a fresh H1 (body H1 must not rename)', () => {
    // Once the file has a real name, typing more H1s in the body must not
    // re-trigger auto-rename. isPlaceholder is the only gate.
    expect(renameFromH1('开篇.md', 'Second heading', [])).toBeNull();
    expect(renameFromH1('Chapter 3 开篇.md', 'Another', [])).toBeNull();
    expect(renameFromH1('03-开篇.md', 'Another', [])).toBeNull();
  });
});
