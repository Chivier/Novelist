import { parseNumber } from './numbering';

export type SortMode =
  | 'name-asc'
  | 'name-desc'
  | 'numeric-asc'
  | 'numeric-desc'
  | 'mtime-desc'
  | 'mtime-asc';

export interface SortableEntry {
  name: string;
  is_dir: boolean;
  mtime?: number | null;
}

/**
 * Extract the leftmost run of digits OR Chinese numerals from the name (sans .md).
 * Returns { prefix, value, suffix } or null if no number found.
 */
function extractLeftmostNumber(name: string): { prefix: string; value: number; suffix: string } | null {
  const stem = name.replace(/\.[^.]+$/, '');
  // Try arabic
  const arabicMatch = /^(.*?)(\d+)(.*)$/.exec(stem);
  // Try CJK numerals
  const cjkMatch = /^(.*?)([\u4e00-\u9fff]+)(.*)$/.exec(stem);
  const candidates: Array<{ prefix: string; numStr: string; suffix: string; pos: number }> = [];
  if (arabicMatch) candidates.push({ prefix: arabicMatch[1], numStr: arabicMatch[2], suffix: arabicMatch[3], pos: arabicMatch[1].length });
  if (cjkMatch) {
    const parsed = parseNumber(cjkMatch[2]);
    if (parsed !== null) candidates.push({ prefix: cjkMatch[1], numStr: cjkMatch[2], suffix: cjkMatch[3], pos: cjkMatch[1].length });
  }
  if (candidates.length === 0) return null;
  // Earliest position wins; on tie, prefer arabic (listed first naturally)
  candidates.sort((a, b) => a.pos - b.pos);
  const c = candidates[0];
  const parsed = parseNumber(c.numStr);
  if (parsed === null) return null;
  return { prefix: c.prefix, value: parsed.value, suffix: c.suffix };
}

function compareNumeric(a: SortableEntry, b: SortableEntry): number {
  const an = extractLeftmostNumber(a.name);
  const bn = extractLeftmostNumber(b.name);
  if (an && bn) {
    const pfxCmp = an.prefix.toLowerCase().localeCompare(bn.prefix.toLowerCase());
    if (pfxCmp !== 0) return pfxCmp;
    if (an.value !== bn.value) return an.value - bn.value;
    return an.suffix.toLowerCase().localeCompare(bn.suffix.toLowerCase());
  }
  if (an && !bn) return -1;
  if (!an && bn) return 1;
  return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
}

export function compareByMode(a: SortableEntry, b: SortableEntry, mode: SortMode): number {
  // Folders always first
  if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;

  switch (mode) {
    case 'name-asc': return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    case 'name-desc': return b.name.toLowerCase().localeCompare(a.name.toLowerCase());
    case 'numeric-asc': return compareNumeric(a, b);
    case 'numeric-desc': return -compareNumeric(a, b);
    case 'mtime-desc': return (b.mtime ?? 0) - (a.mtime ?? 0);
    case 'mtime-asc': return (a.mtime ?? 0) - (b.mtime ?? 0);
  }
}
