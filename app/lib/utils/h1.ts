/**
 * Extract the first H1 heading text from a markdown document.
 * Returns null if no H1 found.
 *
 * Recognizes:
 * - ATX: `# Title` (with required space or end-of-line after #)
 * - Setext: `Title\n====`
 *
 * Skips:
 * - YAML frontmatter (--- block at top)
 * - Fenced code blocks (``` and ~~~)
 * - Indented code blocks (4 spaces)
 */
export function extractFirstH1(markdown: string): string | null {
  const lines = markdown.split('\n');
  let i = 0;

  // Skip frontmatter
  if (lines[0]?.trim() === '---') {
    i = 1;
    while (i < lines.length && lines[i].trim() !== '---') i++;
    if (i < lines.length) i++; // skip closing ---
  }

  let inFence: string | null = null; // '```' or '~~~' or null

  for (; i < lines.length; i++) {
    const line = lines[i];

    // Toggle fence
    if (inFence) {
      if (line.trimStart().startsWith(inFence)) inFence = null;
      continue;
    }
    const trimStart = line.trimStart();
    if (trimStart.startsWith('```')) {
      inFence = '```';
      continue;
    }
    if (trimStart.startsWith('~~~')) {
      inFence = '~~~';
      continue;
    }

    // Indented code block (4+ spaces)
    if (line.startsWith('    ')) continue;

    // ATX H1: `#` followed by space/tab + text, or `#` alone (end of line).
    // Rejects `#Text` (no space), and `##...` (higher-level heading).
    const atx = /^#(?:[ \t](.*)|)$/.exec(line);
    if (atx && !line.startsWith('##')) {
      let text = atx[1] ?? '';
      // Strip closing # sequence (per CommonMark)
      text = text.replace(/\s+#+\s*$/, '');
      return text.trim();
    }

    // Setext H1: next line is === run
    if (i + 1 < lines.length && /^=+\s*$/.test(lines[i + 1]) && line.trim().length > 0) {
      return line.trim();
    }
  }

  return null;
}
