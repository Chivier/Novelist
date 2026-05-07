import { describe, it, expect } from 'vitest';
import { formatDate } from '$lib/utils/date-format';

// 2026-05-07 14:08:32 in the local TZ. Constructing via Date(year, month-1, ...)
// keeps tests stable across the CI machine's timezone.
const SAMPLE = new Date(2026, 4, 7, 14, 8, 32);

describe('formatDate', () => {
  it('handles all individual tokens', () => {
    expect(formatDate(SAMPLE, 'YYYY')).toBe('2026');
    expect(formatDate(SAMPLE, 'YY')).toBe('26');
    expect(formatDate(SAMPLE, 'MM')).toBe('05');
    expect(formatDate(SAMPLE, 'M')).toBe('5');
    expect(formatDate(SAMPLE, 'DD')).toBe('07');
    expect(formatDate(SAMPLE, 'D')).toBe('7');
    expect(formatDate(SAMPLE, 'HH')).toBe('14');
    expect(formatDate(SAMPLE, 'mm')).toBe('08');
    expect(formatDate(SAMPLE, 'ss')).toBe('32');
  });

  it('combines tokens into common patterns', () => {
    expect(formatDate(SAMPLE, 'YYYYMMDD')).toBe('20260507');
    expect(formatDate(SAMPLE, 'YYMMDD')).toBe('260507');
    expect(formatDate(SAMPLE, 'YYYY-MM-DD')).toBe('2026-05-07');
    expect(formatDate(SAMPLE, 'HHmm')).toBe('1408');
    expect(formatDate(SAMPLE, 'HHmmss')).toBe('140832');
    expect(formatDate(SAMPLE, 'YYYYMMDD-HHmmss')).toBe('20260507-140832');
  });

  it('passes unknown characters through verbatim (CJK literals)', () => {
    expect(formatDate(SAMPLE, 'YYYY年MM月DD日')).toBe('2026年05月07日');
    expect(formatDate(SAMPLE, 'WW')).toBe('WW'); // unknown token kept verbatim
  });

  it('reserved fmt "timestamp" returns Unix seconds', () => {
    const out = formatDate(SAMPLE, 'timestamp');
    expect(out).toBe(String(Math.floor(SAMPLE.getTime() / 1000)));
    expect(/^\d+$/.test(out)).toBe(true);
  });

  it('reserved fmt "iso" returns ISO 8601 with local offset', () => {
    const out = formatDate(SAMPLE, 'iso');
    // Format: YYYY-MM-DDTHH:mm:ss±HH:mm
    expect(out).toMatch(/^2026-05-07T14:08:32[+-]\d{2}:\d{2}$/);
  });

  it('handles single-digit months/days/hours without padding for M/D', () => {
    const d = new Date(2026, 0, 3, 9, 5, 7); // Jan 3 09:05:07
    expect(formatDate(d, 'M-D')).toBe('1-3');
    expect(formatDate(d, 'MM-DD')).toBe('01-03');
    expect(formatDate(d, 'HH:mm:ss')).toBe('09:05:07');
  });

  it('handles year boundaries (leap-year Feb 29, end-of-year midnight)', () => {
    const leap = new Date(2024, 1, 29, 0, 0, 0);
    expect(formatDate(leap, 'YYYY-MM-DD')).toBe('2024-02-29');

    const eoy = new Date(2026, 11, 31, 23, 59, 59);
    expect(formatDate(eoy, 'YYYYMMDD-HHmmss')).toBe('20261231-235959');
  });
});
