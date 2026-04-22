import { describe, it, expect, vi, beforeEach } from 'vitest';

/**
 * [contract] syncMenu — pushes the current i18n labels + recent-projects
 * list into the Rust `refresh_menu` IPC so the native menu reflects the
 * UI state. Paired with the `useMenuSync` effect wired from App.svelte,
 * but tested here as a plain function (rune `$effect` tests are covered
 * via the integration path in App.svelte).
 */

const { hoisted } = vi.hoisted(() => {
  const refreshMenu = vi.fn(async (_labels: unknown, _recent: unknown) => ({
    status: 'ok' as const,
    data: null,
  }));

  // Minimal i18n stub — enough for the buildLabels() call. Each key
  // round-trips to a namespaced string so we can assert the shape
  // without depending on real translations.
  const t = (key: string) => `t:${key}`;

  return {
    hoisted: { refreshMenu, t },
  };
});

vi.mock('$lib/ipc/commands', () => ({
  commands: { refreshMenu: hoisted.refreshMenu },
}));

vi.mock('$lib/i18n/index.svelte', () => ({
  t: hoisted.t,
  i18n: { locale: 'en' },
}));

describe('[contract] syncMenu', () => {
  beforeEach(() => {
    hoisted.refreshMenu.mockClear();
  });

  it('calls commands.refreshMenu with the current labels and recents', async () => {
    const { syncMenu } = await import('$lib/composables/menu-sync.svelte');
    await syncMenu([
      { path: '/a', name: 'A', last_opened: '1', pinned: false, sort_order: null },
      { path: '/b', name: 'B', last_opened: '2', pinned: false, sort_order: null },
    ]);

    expect(hoisted.refreshMenu).toHaveBeenCalledOnce();
    const [labels, recent] = hoisted.refreshMenu.mock.calls[0];
    expect(recent).toEqual([
      { name: 'A', path: '/a' },
      { name: 'B', path: '/b' },
    ]);
    // Labels: at least one field present and resolved via the i18n stub.
    expect((labels as Record<string, string>).file_menu).toBe('t:menu.file');
    expect((labels as Record<string, string>).new_file).toBe('t:command.newFile');
    expect((labels as Record<string, string>).open_recent).toBe('t:menu.openRecent');
  });

  it('truncates the Open Recent list to 10 entries (matches menu UX)', async () => {
    const { syncMenu } = await import('$lib/composables/menu-sync.svelte');
    const recents = Array.from({ length: 15 }, (_, i) => ({
      path: `/p${i}`,
      name: `P${i}`,
      last_opened: String(i),
      pinned: false,
      sort_order: null,
    }));
    await syncMenu(recents);

    const [, passed] = hoisted.refreshMenu.mock.calls[0];
    expect((passed as unknown[]).length).toBe(10);
    expect((passed as { path: string }[])[0].path).toBe('/p0');
    expect((passed as { path: string }[])[9].path).toBe('/p9');
  });

  it('swallows refreshMenu errors so a broken menu build cannot crash the app', async () => {
    hoisted.refreshMenu.mockRejectedValueOnce(new Error('boom'));
    const { syncMenu } = await import('$lib/composables/menu-sync.svelte');
    await expect(syncMenu([])).resolves.toBeUndefined();
  });

  it('passes an empty recents array through untouched when the list is empty', async () => {
    const { syncMenu } = await import('$lib/composables/menu-sync.svelte');
    await syncMenu([]);
    const [, passed] = hoisted.refreshMenu.mock.calls[0];
    expect(passed).toEqual([]);
  });
});
