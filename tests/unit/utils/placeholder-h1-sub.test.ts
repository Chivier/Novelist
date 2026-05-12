import { describe, it, expect } from 'vitest';
import { applyH1Substitution } from '$lib/utils/placeholder';

describe('applyH1Substitution', () => {
  it('replaces rightmost occurrence of old H1 in stem (prefix preserved)', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '序幕', [])).toBe('第1章-序幕.md');
  });

  it('uses lastIndexOf when old H1 string appears in the prefix too', () => {
    // contrived: prefix literal happens to contain the same characters as old H1
    expect(applyH1Substitution('第开篇章-开篇.md', '开篇', '终结', [])).toBe('第开篇章-终结.md');
  });

  it('sanitizes the new H1 before substitution (filesystem-forbidden chars)', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', 'a/b:c', [])).toBe('第1章-a-b-c.md');
  });

  it('returns null when old H1 (sanitized) is not in the current filename — manual rename detach', () => {
    expect(applyH1Substitution('chapter1.md', '开篇', '序幕', [])).toBeNull();
  });

  it('returns null when sanitized new H1 is empty (nothing to substitute with)', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '   ', [])).toBeNull();
  });

  it('returns null when sanitized old H1 is empty', () => {
    expect(applyH1Substitution('第1章-开篇.md', '   ', '序幕', [])).toBeNull();
  });

  it('returns null when old and new H1 sanitize to the same stem', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '开篇 ', [])).toBeNull();
  });

  it('bumps with " 2" suffix on sibling collision', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '序幕', ['第1章-序幕.md'])).toBe(
      '第1章-序幕 2.md',
    );
  });

  it('does not collide with itself', () => {
    expect(applyH1Substitution('第1章-开篇.md', '开篇', '序幕', ['第1章-开篇.md'])).toBe(
      '第1章-序幕.md',
    );
  });

  it('replaces the whole stem when old H1 is the entire stem (no prefix)', () => {
    expect(applyH1Substitution('开篇.md', '开篇', '序幕', [])).toBe('序幕.md');
  });
});
