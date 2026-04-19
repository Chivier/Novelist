/**
 * Convert arbitrary text (e.g., an H1 heading) into a safe filename stem.
 * - Strips leading `#` markers, surrounding whitespace, and trailing `.md`/`.markdown`
 * - Replaces forbidden filesystem chars with `-`
 * - Collapses repeated dashes
 * - Truncates to 80 chars (counted by code points, not bytes)
 * - Returns empty string when nothing usable remains
 */
export function sanitizeFilenameStem(input: string): string {
  let s = input.trim();
  // Strip leading ATX hashes ("# Title", "## also", "#" alone)
  s = s.replace(/^#+\s*/, '');
  // Strip trailing closing-hash sequence ("Title #" / "Title ###")
  s = s.replace(/\s+#+\s*$/, '').trim();
  // Strip trailing .md / .markdown
  s = s.replace(/\.(md|markdown)$/i, '').trim();
  // Replace forbidden chars
  s = s.replace(/[\/\\:*?"<>|]/g, '-');
  // Collapse repeated dashes
  s = s.replace(/-{2,}/g, '-');
  // Hidden-file guard
  if (s.startsWith('.')) s = '_' + s;
  // Length cap (code points)
  const codepoints = Array.from(s);
  if (codepoints.length > 80) s = codepoints.slice(0, 80).join('');
  return s;
}
