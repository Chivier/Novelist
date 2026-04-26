import { describe, expect, it } from 'vitest';
import {
  buildContextPack,
  commandInstruction,
  contextPackToPrompt,
  parseMentions,
  parseSkillTokens,
  parseSlashCommand,
  skillAssetsForTokens,
  stripMentionTokens,
  stripSkillTokens,
  type AiContextItem,
} from '$lib/components/ai-shared/context';

describe('[contract] AI shared context parsing', () => {
  it('parses supported mention tokens', () => {
    expect(parseMentions('Use @selection with @current and @file:chapter-1')).toEqual([
      { kind: 'selection', raw: '@selection' },
      { kind: 'current-file', raw: '@current' },
      { kind: 'project-file', raw: '@file:chapter-1', query: 'chapter-1' },
    ]);
  });

  it('strips mention and skill tokens from user text', () => {
    const text = 'Please /keep @current $plot-doctor tighten this';
    expect(stripMentionTokens(text)).toBe('Please /keep $plot-doctor tighten this');
    expect(stripSkillTokens(text)).toBe('Please /keep @current tighten this');
  });

  it('parses slash commands and leaves unknown commands alone', () => {
    expect(parseSlashCommand('/rewrite make it sharper')).toEqual({
      id: 'rewrite',
      raw: '/rewrite',
      rest: 'make it sharper',
    });
    expect(parseSlashCommand('/nope test')).toBeNull();
  });

  it('maps slash commands to built-in writing instructions', () => {
    const cmd = parseSlashCommand('/summarize @current');
    expect(commandInstruction(cmd)).toContain('Summarize');
    expect(commandInstruction(parseSlashCommand('/plan next'))).toBeNull();
  });

  it('resolves skill tokens against available prompt assets', () => {
    const assets = [
      { id: 'skills/plot-doctor/SKILL.md', kind: 'skill', path: '/p/SKILL.md', name: 'plot-doctor', content: 'plot' },
    ];
    expect(parseSkillTokens('$plot-doctor and $missing')).toEqual([
      { raw: '$plot-doctor', name: 'plot-doctor' },
      { raw: '$missing', name: 'missing' },
    ]);
    expect(skillAssetsForTokens(parseSkillTokens('$plot-doctor $missing'), assets)).toEqual([assets[0]]);
  });
});

describe('[contract] AI context packs', () => {
  it('dedupes and truncates context items within the character budget', () => {
    const items: AiContextItem[] = [
      { id: 'a', kind: 'current-file', label: 'A', path: '/a.md', content: 'a'.repeat(30) },
      { id: 'a-copy', kind: 'current-file', label: 'A copy', path: '/a.md', content: 'duplicate' },
      { id: 'b', kind: 'manual-note', label: 'B', content: 'b'.repeat(50) },
    ];
    const pack = buildContextPack('ask', items, 40);
    expect(pack.items).toHaveLength(2);
    expect(pack.items[0].content).toBe('a'.repeat(30));
    expect(pack.items[1].truncated).toBe(true);
    expect(pack.estimatedChars).toBeGreaterThanOrEqual(40);
  });

  it('renders context packs as a prompt with labels and paths', () => {
    const prompt = contextPackToPrompt({
      userText: 'help',
      estimatedChars: 4,
      items: [{ id: 'x', kind: 'selection', label: 'Selection', path: '/x.md', content: 'text' }],
    });
    expect(prompt).toContain('## Context 1: Selection');
    expect(prompt).toContain('Path: /x.md');
    expect(prompt).toContain('## User request\nhelp');
  });
});
