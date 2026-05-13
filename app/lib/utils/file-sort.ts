import { parseNumber } from './numbering';

export type SortMode =
  | 'name-asc'
  | 'name-desc'
  | 'numeric-asc'
  | 'numeric-desc'
  | 'mtime-desc'
  | 'mtime-asc'
  | 'ctime-desc'
  | 'ctime-asc';

export interface SortableEntry {
  name: string;
  is_dir: boolean;
  mtime?: number | null;
  ctime?: number | null;
}

/**
 * Extract the leftmost run of digits OR Chinese numerals from the name (sans .md).
 * Returns { prefix, value, suffix } or null if no number found.
 */
function extractLeftmostNumber(name: string): { prefix: string; value: number; suffix: string } | null {
  const stem = name.replace(/\.[^.]+$/, '');
  const numberRuns = /\d+|[零一二三四五六七八九十百]+/gu;
  for (const match of stem.matchAll(numberRuns)) {
    const numStr = match[0];
    const parsed = parseNumber(numStr);
    if (parsed === null) continue;
    const pos = match.index ?? 0;
    return {
      prefix: stem.slice(0, pos),
      value: parsed.value,
      suffix: stem.slice(pos + numStr.length),
    };
  }
  return null;
}

function compareName(a: string, b: string): number {
  return a.toLowerCase().localeCompare(b.toLowerCase()) || a.localeCompare(b);
}

function compareNumeric(a: SortableEntry, b: SortableEntry): number {
  const an = extractLeftmostNumber(a.name);
  const bn = extractLeftmostNumber(b.name);
  if (an && bn) {
    const pfxCmp = compareName(an.prefix, bn.prefix);
    if (pfxCmp !== 0) return pfxCmp;
    if (an.value !== bn.value) return an.value - bn.value;
    return compareName(an.suffix, bn.suffix) || compareName(a.name, b.name);
  }
  if (an && !bn) return -1;
  if (!an && bn) return 1;
  return compareName(a.name, b.name);
}

export function compareByMode(a: SortableEntry, b: SortableEntry, mode: SortMode): number {
  // Folders always first
  if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;

  switch (mode) {
    case 'name-asc': return compareName(a.name, b.name);
    case 'name-desc': return compareName(b.name, a.name);
    case 'numeric-asc': return compareNumeric(a, b);
    case 'numeric-desc': return -compareNumeric(a, b);
    case 'mtime-desc': return (b.mtime ?? 0) - (a.mtime ?? 0);
    case 'mtime-asc': return (a.mtime ?? 0) - (b.mtime ?? 0);
    case 'ctime-desc': return (b.ctime ?? b.mtime ?? 0) - (a.ctime ?? a.mtime ?? 0);
    case 'ctime-asc': return (a.ctime ?? a.mtime ?? 0) - (b.ctime ?? b.mtime ?? 0);
  }
}
