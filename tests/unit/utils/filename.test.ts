import { describe, it, expect } from 'vitest';
import { sanitizeFilenameStem } from '$lib/utils/filename';

describe('sanitizeFilenameStem', () => {
  it('passes ordinary text through', () => {
    expect(sanitizeFilenameStem('开篇')).toBe('开篇');
    expect(sanitizeFilenameStem('Hello World')).toBe('Hello World');
  });
  it('strips leading hash and whitespace from H1 carry-over', () => {
    expect(sanitizeFilenameStem('#  开篇')).toBe('开篇');
    expect(sanitizeFilenameStem('# 开篇 #')).toBe('开篇');
  });
  it('strips .md extension if user typed it', () => {
    expect(sanitizeFilenameStem('foo.md')).toBe('foo');
    expect(sanitizeFilenameStem('foo.markdown')).toBe('foo');
  });
  it('replaces forbidden filesystem chars with -', () => {
    expect(sanitizeFilenameStem('a/b\\c:d*e?f"g<h>i|j')).toBe('a-b-c-d-e-f-g-h-i-j');
  });
  it('collapses repeated dashes', () => {
    expect(sanitizeFilenameStem('a///b')).toBe('a-b');
  });
  it('trims length to 80 chars (UTF-8 aware)', () => {
    const long = '字'.repeat(100);
    expect(sanitizeFilenameStem(long).length).toBe(80);
  });
  it('returns empty for whitespace-only', () => {
    expect(sanitizeFilenameStem('   ')).toBe('');
    expect(sanitizeFilenameStem('')).toBe('');
  });
  it('prefixes underscore to leading dot', () => {
    expect(sanitizeFilenameStem('.hidden')).toBe('_.hidden');
  });
});
