import { describe, it, expect, vi, beforeEach } from 'vitest';

/**
 * [precision] chinese.ts — Simplified/Traditional conversion + pinyin.
 * Uses module mocks for opencc-js subpath presets and pinyin-pro to avoid
 * pulling in the full dictionaries (which are huge). Verifies lazy-loading
 * contract + per-direction memoization of the OpenCC converters.
 */

const s2tConverterFactory = vi.fn(({ from, to }: { from: string; to: string }) =>
  (text: string) => `${from}->${to}:${text}`,
);
const t2sConverterFactory = vi.fn(({ from, to }: { from: string; to: string }) =>
  (text: string) => `${from}->${to}:${text}`,
);

vi.mock('opencc-js/cn2t', () => ({
  Converter: s2tConverterFactory,
}));

vi.mock('opencc-js/t2cn', () => ({
  Converter: t2sConverterFactory,
}));

vi.mock('pinyin-pro', () => ({
  pinyin: (text: string, opts: { toneType: string; type: string }) =>
    `${text}|${opts.toneType}|${opts.type}`,
}));

beforeEach(() => {
  s2tConverterFactory.mockClear();
  t2sConverterFactory.mockClear();
  vi.resetModules();
});

describe('[precision] chinese — simplifiedToTraditional', () => {
  it('delegates to the opencc cn->tw converter', async () => {
    const { simplifiedToTraditional } = await import('$lib/utils/chinese');
    const result = await simplifiedToTraditional('简体');
    expect(result).toBe('cn->tw:简体');
  });

  it('constructs the simplified-to-traditional converter exactly once across repeated calls', async () => {
    const { simplifiedToTraditional } = await import('$lib/utils/chinese');
    await simplifiedToTraditional('a');
    await simplifiedToTraditional('b');
    expect(s2tConverterFactory).toHaveBeenCalledTimes(1);
    expect(s2tConverterFactory).toHaveBeenCalledWith({ from: 'cn', to: 'tw' });
    expect(t2sConverterFactory).not.toHaveBeenCalled();
  });

  it('constructs each direction independently when both are used', async () => {
    const { simplifiedToTraditional, traditionalToSimplified } = await import(
      '$lib/utils/chinese'
    );
    await simplifiedToTraditional('a');
    await simplifiedToTraditional('b');
    await traditionalToSimplified('c');
    expect(s2tConverterFactory).toHaveBeenCalledTimes(1);
    expect(t2sConverterFactory).toHaveBeenCalledTimes(1);
    expect(s2tConverterFactory).toHaveBeenCalledWith({ from: 'cn', to: 'tw' });
    expect(t2sConverterFactory).toHaveBeenCalledWith({ from: 'tw', to: 'cn' });
  });
});

describe('[precision] chinese — traditionalToSimplified', () => {
  it('delegates to the opencc tw->cn converter', async () => {
    const { traditionalToSimplified } = await import('$lib/utils/chinese');
    expect(await traditionalToSimplified('繁體')).toBe('tw->cn:繁體');
  });
});

describe('[precision] chinese — toPinyin', () => {
  it('uses symbol tone marks and string return type', async () => {
    const { toPinyin } = await import('$lib/utils/chinese');
    expect(await toPinyin('你好')).toBe('你好|symbol|string');
  });
});
