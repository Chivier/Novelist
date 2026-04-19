import { describe, it, expect } from 'vitest';
import { extractFirstH1 } from '$lib/utils/h1';

describe('extractFirstH1', () => {
  it('extracts ATX H1', () => {
    expect(extractFirstH1('# Hello\n\nbody')).toBe('Hello');
  });
  it('strips trailing #', () => {
    expect(extractFirstH1('# Title #')).toBe('Title');
    expect(extractFirstH1('# Title ###')).toBe('Title');
  });
  it('trims whitespace', () => {
    expect(extractFirstH1('#   Spaced   ')).toBe('Spaced');
  });
  it('returns null when no H1', () => {
    expect(extractFirstH1('## Only H2')).toBeNull();
    expect(extractFirstH1('plain body')).toBeNull();
    expect(extractFirstH1('')).toBeNull();
  });
  it('returns first when multiple H1s', () => {
    expect(extractFirstH1('# First\n\n# Second')).toBe('First');
  });
  it('skips frontmatter', () => {
    const md = '---\ntitle: meta\n---\n\n# Real Title\n';
    expect(extractFirstH1(md)).toBe('Real Title');
  });
  it('skips fenced code blocks', () => {
    const md = '```\n# fake\n```\n\n# real\n';
    expect(extractFirstH1(md)).toBe('real');
  });
  it('skips tilde-fenced code blocks', () => {
    const md = '~~~\n# fake\n~~~\n\n# real\n';
    expect(extractFirstH1(md)).toBe('real');
  });
  it('skips indented (4-space) code blocks', () => {
    const md = '    # fake\n\n# real\n';
    expect(extractFirstH1(md)).toBe('real');
  });
  it('recognizes Setext H1', () => {
    expect(extractFirstH1('Title\n=====\n')).toBe('Title');
    expect(extractFirstH1('Multi line\n===\n')).toBe('Multi line');
  });
  it('does not treat # without space as H1', () => {
    // ATX requires space or end-of-line after #
    expect(extractFirstH1('#NoSpace\n')).toBeNull();
    expect(extractFirstH1('#\n')).toBe('');
  });
});
