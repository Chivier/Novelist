export type NumberStyle =
  | { kind: 'arabic'; width: number }
  | { kind: 'chinese-lower' }
  | { kind: 'chinese-upper' }
  | { kind: 'roman-upper' };

export interface ParsedNumber {
  value: number;
  style: NumberStyle;
}

export function parseNumber(s: string): ParsedNumber | null {
  if (/^\d+$/.test(s)) {
    return { value: parseInt(s, 10), style: { kind: 'arabic', width: s.length } };
  }
  return null;
}

export function formatNumber(value: number, style: NumberStyle): string {
  if (style.kind === 'arabic') return String(value).padStart(style.width, '0');
  throw new Error(`formatNumber: unsupported style ${style.kind}`);
}
