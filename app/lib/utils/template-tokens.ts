/**
 * Snippet-template token utilities.
 *
 * Kept framework-free so vitest can import them directly. There are two
 * independent concerns here:
 *
 * 1. Body token substitution (`resolveBody`) — supports a fixed, read-only
 *    set of variables (`{date}`, `{time}`, `{datetime}`, `{filename}`,
 *    `{project}`). Date/time variables additionally accept a `:fmt`
 *    argument routed through `formatDate` (e.g. `{date:YYMMDD}`,
 *    `{datetime:timestamp}`). Unknown `{...}` tokens are left verbatim so
 *    user content isn't mangled.
 *
 * 2. Caret anchor (`extractCursorAnchor`) — the literal marker `$|$` inside a
 *    body. The first occurrence wins; it is stripped from the returned body
 *    and its byte offset is returned so the editor can place the caret there
 *    after inserting. Further markers are stripped silently.
 */

import { formatDate } from './date-format';

export interface TemplateContext {
  /** Localized YYYY-MM-DD */
  date: string;
  /** Localized HH:mm (24h) */
  time: string;
  /** Combined date + time, "YYYY-MM-DD HH:mm" */
  datetime: string;
  /** Active editor's file stem (without .md), or '' if no file is open. */
  filename: string;
  /** Project folder name, or '' if no project is open. */
  project: string;
  /** Raw Date used to derive `date` / `time` / `datetime`. Used by `:fmt`. */
  _now: Date;
}

/** Produce a default context from `now` + active filename / project dir. */
export function makeTemplateContext(opts: {
  now?: Date;
  activeFilePath?: string | null;
  projectDir?: string | null;
} = {}): TemplateContext {
  const now = opts.now ?? new Date();
  const date = formatDate(now, 'YYYY-MM-DD');
  const time = formatDate(now, 'HH:mm');
  return {
    date,
    time,
    datetime: `${date} ${time}`,
    filename: basenameStem(opts.activeFilePath ?? ''),
    project: basename(opts.projectDir ?? ''),
    _now: now,
  };
}

function basename(p: string): string {
  if (!p) return '';
  const parts = p.replace(/[\\/]+$/, '').split(/[\\/]/);
  return parts[parts.length - 1] ?? '';
}

function basenameStem(p: string): string {
  const b = basename(p);
  if (!b) return '';
  const dot = b.lastIndexOf('.');
  return dot > 0 ? b.slice(0, dot) : b;
}

/**
 * Expand recognized `{token}` and `{token:fmt}` variables in `raw` using
 * `ctx`. Unknown tokens are left alone so they can survive as literal text
 * in the user's document (e.g. `{myCustomThing}` written in a pasted code
 * block).
 *
 * The `:fmt` argument is honored only for `date` / `time` / `datetime`. For
 * other variables, the fmt suffix is ignored and the bare value is emitted
 * — this keeps the door open for future per-variable formatters without
 * breaking older templates.
 */
export function resolveBody(raw: string, ctx: TemplateContext): string {
  const dateLikes = new Set(['date', 'time', 'datetime']);
  const simple: Record<string, string> = {
    date: ctx.date,
    time: ctx.time,
    datetime: ctx.datetime,
    filename: ctx.filename,
    project: ctx.project,
  };
  return raw.replace(
    /\{([a-zA-Z][a-zA-Z0-9_-]*)(?::([^}]*))?\}/g,
    (match, name: string, fmt: string | undefined) => {
      if (!Object.prototype.hasOwnProperty.call(simple, name)) {
        // Unknown variable: emit verbatim so user prose / code samples survive.
        return match;
      }
      if (fmt && dateLikes.has(name)) {
        return formatDate(ctx._now, fmt);
      }
      return simple[name];
    },
  );
}

export const CURSOR_ANCHOR = '$|$';

/**
 * Strip all occurrences of the cursor anchor from `raw` and return the byte
 * offset at which the first one lived.
 *
 * - No anchor present → `{ body: raw, anchor: -1 }`.
 * - First anchor wins; additional anchors are stripped silently.
 */
export function extractCursorAnchor(raw: string): { body: string; anchor: number } {
  const idx = raw.indexOf(CURSOR_ANCHOR);
  if (idx < 0) return { body: raw, anchor: -1 };
  // Replace ALL occurrences so stray extra markers don't end up as literal "$|$"
  // in the document. Only the first one keeps its position semantics.
  const body = raw.split(CURSOR_ANCHOR).join('');
  return { body, anchor: idx };
}

/**
 * Resolve a `defaultFilename` template for new-file mode.
 *
 * - Substitutes the variable tokens (`{date}` etc.) via `resolveBody`.
 * - `{N}/{2N}/{CN}/...` placeholders are NOT resolved here — callers that
 *   want numbering inference should run the result through `inferNextName`.
 *   v1 keeps this pure; the frontend store does the optional numbering pass.
 */
export function resolveFilename(raw: string, ctx: TemplateContext): string {
  return resolveBody(raw, ctx);
}
