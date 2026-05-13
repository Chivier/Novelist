interface Command {
  id: string;
  label: string;
  /**
   * Optional alternate-language label rendered as a small subscript in
   * the command palette and matched alongside `label` when the user
   * searches. Lets a Chinese-locale user find a command by typing its
   * English name (or vice versa) without switching locale.
   *
   * May be set directly OR resolved lazily via `secondaryLabelFn` —
   * the latter lets us look up the alternate-locale translation only
   * when the palette renders, after both locale chunks have loaded.
   */
  secondaryLabel?: string;
  secondaryLabelFn?: () => string | undefined;
  shortcut?: string;
  handler: () => void;
}

/** Resolve a command's secondary label, preferring the lazy fn when set. */
export function getSecondaryLabel(c: Command): string | undefined {
  if (c.secondaryLabelFn) {
    const v = c.secondaryLabelFn();
    if (v && v !== c.label) return v;
    return undefined;
  }
  return c.secondaryLabel;
}

/**
 * Window (in milliseconds) during which a second `execute(id)` call
 * for the same ID is ignored. Protects against OS menu accelerator
 * + JS keydown handler both dispatching the same command when a
 * shortcut has a menu equivalent.
 */
const DEDUPE_WINDOW_MS = 50;

class CommandRegistry {
  commands = $state<Command[]>([]);
  private lastExecuted = new Map<string, number>();

  register(cmd: Command) {
    // Avoid duplicates
    if (!this.commands.find(c => c.id === cmd.id)) {
      this.commands.push(cmd);
    }
  }

  execute(id: string) {
    const now = Date.now();
    const last = this.lastExecuted.get(id);
    if (last !== undefined && now - last < DEDUPE_WINDOW_MS) {
      return;
    }
    this.lastExecuted.set(id, now);

    const cmd = this.commands.find(c => c.id === id);
    if (cmd) cmd.handler();
  }

  search(query: string): Command[] {
    if (!query.trim()) return this.commands;
    const q = query.toLowerCase();
    return this.commands.filter(c => {
      if (c.label.toLowerCase().includes(q)) return true;
      const sec = getSecondaryLabel(c);
      if (sec && sec.toLowerCase().includes(q)) return true;
      return false;
    });
  }

  /**
   * Test helper — drops the dedupe window memory. Not called from
   * production code. Production tests that fire the same command in
   * rapid succession (e.g. two shortcut parser variants for the same
   * action) would otherwise trip the 50 ms dedupe window.
   */
  _resetDedupe() {
    this.lastExecuted.clear();
  }
}

export const commandRegistry = new CommandRegistry();
export type { Command };
