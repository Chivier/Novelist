import { describe, expect, it } from 'vitest';
import {
  CURSOR_ANCHOR,
  extractCursorAnchor,
  makeTemplateContext,
  resolveBody,
  resolveFilename,
} from '../../../app/lib/utils/template-tokens';

describe('makeTemplateContext', () => {
  it('formats date and time with leading zeros', () => {
    const ctx = makeTemplateContext({ now: new Date(2026, 0, 4, 5, 7) });
    expect(ctx.date).toBe('2026-01-04');
    expect(ctx.time).toBe('05:07');
  });

  it('derives filename stem from active path', () => {
    const ctx = makeTemplateContext({ now: new Date(2026, 0, 1), activeFilePath: '/proj/chapters/第一章.md' });
    expect(ctx.filename).toBe('第一章');
  });

  it('derives filename as empty when no active file', () => {
    const ctx = makeTemplateContext({ now: new Date(2026, 0, 1) });
    expect(ctx.filename).toBe('');
  });

  it('derives project name from project dir', () => {
    const ctx = makeTemplateContext({ now: new Date(2026, 0, 1), projectDir: '/Users/me/Documents/MyNovel/' });
    expect(ctx.project).toBe('MyNovel');
  });

  it('handles dotfile-only filenames without stem confusion', () => {
    const ctx = makeTemplateContext({ now: new Date(2026, 0, 1), activeFilePath: '/x/.hidden' });
    // basename is ".hidden"; stem logic treats a leading-dot-only name as having no extension.
    expect(ctx.filename).toBe('.hidden');
  });
});

describe('resolveBody', () => {
  const ctx = makeTemplateContext({
    now: new Date(2026, 3, 20, 14, 5),
    activeFilePath: '/p/draft.md',
    projectDir: '/p',
  });

  it('substitutes all four known tokens', () => {
    const out = resolveBody('d={date} t={time} f={filename} p={project}', ctx);
    expect(out).toBe('d=2026-04-20 t=14:05 f=draft p=p');
  });

  it('leaves unknown tokens verbatim', () => {
    const out = resolveBody('hi {notATemplate} {name} {date}', ctx);
    expect(out).toBe('hi {notATemplate} {name} 2026-04-20');
  });

  it('does not resolve partial / malformed tokens', () => {
    const out = resolveBody('{date {time} }date}', ctx);
    // "{date " has a space → not matched; "{time}" matches; "}date}" has trailing without opening → not matched.
    expect(out).toBe('{date 14:05 }date}');
  });

  it('resolves multiple occurrences of the same token', () => {
    expect(resolveBody('{date}/{date}/{date}', ctx)).toBe('2026-04-20/2026-04-20/2026-04-20');
  });
});

describe('extractCursorAnchor', () => {
  it('returns -1 when anchor absent', () => {
    const { body, anchor } = extractCursorAnchor('# Title\n\nbody\n');
    expect(anchor).toBe(-1);
    expect(body).toBe('# Title\n\nbody\n');
  });

  it('returns offset of the first anchor and strips it', () => {
    const { body, anchor } = extractCursorAnchor(`# T\n\n${CURSOR_ANCHOR}\n`);
    expect(anchor).toBe(5);
    expect(body).toBe('# T\n\n\n');
  });

  it('when multiple anchors present, first offset wins and rest are silently stripped', () => {
    const raw = `A${CURSOR_ANCHOR}B${CURSOR_ANCHOR}C`;
    const { body, anchor } = extractCursorAnchor(raw);
    expect(anchor).toBe(1);
    expect(body).toBe('ABC');
  });
});

describe('resolveFilename', () => {
  const ctx = makeTemplateContext({
    now: new Date(2026, 5, 15, 9, 0),
    activeFilePath: '/p/current.md',
    projectDir: '/p/My Project',
  });

  it('substitutes {date} / {filename} / {project}', () => {
    expect(resolveFilename('journal-{date}.md', ctx)).toBe('journal-2026-06-15.md');
    expect(resolveFilename('sibling-of-{filename}.md', ctx)).toBe('sibling-of-current.md');
    expect(resolveFilename('{project}-notes.md', ctx)).toBe('My Project-notes.md');
  });

  it('passes {N}-style numbering placeholders through unchanged (caller runs inferNextName)', () => {
    expect(resolveFilename('第{N}章 {filename}.md', ctx)).toBe('第{N}章 current.md');
    expect(resolveFilename('ch-{3N}.md', ctx)).toBe('ch-{3N}.md');
  });
});
