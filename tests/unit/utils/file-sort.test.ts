import { describe, it, expect } from 'vitest';
import { compareByMode, type SortMode } from '$lib/utils/file-sort';

interface F { name: string; is_dir: boolean; mtime?: number; }

const sortNames = (files: F[], mode: SortMode): string[] =>
  [...files].sort((a, b) => compareByMode(a, b, mode)).map(f => f.name);

describe('compareByMode — name-asc', () => {
  it('case-insensitive lex order', () => {
    expect(sortNames(
      [{ name: 'Banana', is_dir: false }, { name: 'apple', is_dir: false }],
      'name-asc'
    )).toEqual(['apple', 'Banana']);
  });
  it('folders first within same mode', () => {
    expect(sortNames(
      [{ name: 'a.md', is_dir: false }, { name: 'z', is_dir: true }],
      'name-asc'
    )).toEqual(['z', 'a.md']);
  });
});

describe('compareByMode — name-desc', () => {
  it('reverses lex order', () => {
    expect(sortNames(
      [{ name: 'a', is_dir: false }, { name: 'b', is_dir: false }],
      'name-desc'
    )).toEqual(['b', 'a']);
  });
});

describe('compareByMode — numeric-asc', () => {
  it('orders chinese chapters numerically', () => {
    expect(sortNames(
      [
        { name: '第十章.md', is_dir: false },
        { name: '第二章.md', is_dir: false },
        { name: '第一章.md', is_dir: false },
      ],
      'numeric-asc'
    )).toEqual(['第一章.md', '第二章.md', '第十章.md']);
  });
  it('orders arabic prefixes numerically', () => {
    expect(sortNames(
      [
        { name: '10-finale.md', is_dir: false },
        { name: '2-rising.md', is_dir: false },
        { name: '1-intro.md', is_dir: false },
      ],
      'numeric-asc'
    )).toEqual(['1-intro.md', '2-rising.md', '10-finale.md']);
  });
  it('non-numeric files sort after numbered, alphabetically', () => {
    expect(sortNames(
      [
        { name: 'notes.md', is_dir: false },
        { name: '1-intro.md', is_dir: false },
        { name: 'appendix.md', is_dir: false },
      ],
      'numeric-asc'
    )).toEqual(['1-intro.md', 'appendix.md', 'notes.md']);
  });
  it('folders still first', () => {
    expect(sortNames(
      [{ name: '1.md', is_dir: false }, { name: 'aaa', is_dir: true }],
      'numeric-asc'
    )).toEqual(['aaa', '1.md']);
  });
});

describe('compareByMode — numeric-desc', () => {
  it('reverses numeric order', () => {
    expect(sortNames(
      [
        { name: '第一章.md', is_dir: false },
        { name: '第十章.md', is_dir: false },
        { name: '第二章.md', is_dir: false },
      ],
      'numeric-desc'
    )).toEqual(['第十章.md', '第二章.md', '第一章.md']);
  });
});

describe('compareByMode — mtime-desc', () => {
  it('newest first', () => {
    expect(sortNames(
      [
        { name: 'old', is_dir: false, mtime: 100 },
        { name: 'new', is_dir: false, mtime: 200 },
      ],
      'mtime-desc'
    )).toEqual(['new', 'old']);
  });
});
