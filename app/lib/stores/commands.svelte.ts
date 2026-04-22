interface Command {
  id: string;
  label: string;
  shortcut?: string;
  handler: () => void;
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
    return this.commands.filter(c => c.label.toLowerCase().includes(q));
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
