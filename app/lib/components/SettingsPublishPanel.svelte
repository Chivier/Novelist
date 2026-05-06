<script lang="ts">
  import { onMount } from 'svelte';
  import { commands, type ChannelConfig, type PlatformConfig, type PublishSettings } from '$lib/ipc/commands';

  type PlatformId = PlatformConfig['platform'];

  let settings: PublishSettings = $state({ channels: [] });
  let loading = $state(true);
  let saveError = $state<string | null>(null);
  let testStatus = $state<Record<string, { kind: 'idle' | 'pending' | 'ok' | 'err'; message?: string }>>({});

  let editing = $state<{ channel: ChannelConfig; isNew: boolean } | null>(null);

  onMount(async () => {
    await reload();
    loading = false;
  });

  async function reload() {
    const r = await commands.getPublishSettings();
    if (r.status === 'ok') settings = r.data;
  }

  async function persist() {
    saveError = null;
    const r = await commands.setPublishSettings(settings);
    if (r.status !== 'ok') saveError = r.error;
  }

  function platformLabel(p: PlatformId): string {
    switch (p) {
      case 'ghost':                 return 'Ghost';
      case 'wordpress_self_hosted': return 'WordPress (self-hosted)';
      case 'wordpress_com':         return 'WordPress.com';
      case 'medium':                return 'Medium';
    }
  }

  function newChannel(platform: PlatformId): ChannelConfig {
    const id = crypto.randomUUID();
    const name = platformLabel(platform);
    let body: PlatformConfig;
    switch (platform) {
      case 'ghost':                 body = { platform: 'ghost', admin_url: '', api_key: '' }; break;
      case 'wordpress_self_hosted': body = { platform: 'wordpress_self_hosted', site_url: '', username: '', app_password: '' }; break;
      case 'wordpress_com':         body = { platform: 'wordpress_com', site_id_or_domain: '', access_token: '' }; break;
      case 'medium':                body = { platform: 'medium', token: '' }; break;
    }
    return { id, name, ...body } as ChannelConfig;
  }

  function startAdd(platform: PlatformId) { editing = { channel: newChannel(platform), isNew: true }; }
  function startEdit(c: ChannelConfig)   { editing = { channel: structuredClone(c), isNew: false }; }
  function cancelEdit()                  { editing = null; }
  async function saveEdit() {
    if (!editing) return;
    const idx = settings.channels.findIndex(c => c.id === editing!.channel.id);
    if (idx >= 0) settings.channels[idx] = editing.channel;
    else settings.channels.push(editing.channel);
    editing = null;
    await persist();
  }
  async function removeChannel(id: string) {
    settings.channels = settings.channels.filter(c => c.id !== id);
    await persist();
  }

  /** Verify creds without creating a post. Per-platform "GET me"-style call.
   *  Implemented as a smoke-publish-with-bad-status that we don't actually
   *  send — instead just check connectivity by issuing a tiny no-op query.
   *  For simplicity, we reuse `convert_markdown_to_html` round-trip as a
   *  sanity check on local Pandoc, AND a real GET-me-style probe per
   *  platform if useful. v0.2.4 keeps this minimal: try a tag-search /
   *  user-info request that all platforms expose without side effects.
   *
   *  Rather than thread a separate "verify" Tauri command per platform,
   *  v0.2.4 ships a best-effort approach: we attempt a publish to a
   *  detect-only sentinel and rely on the auth-failure path to surface
   *  bad credentials. This is intentionally lightweight; a future spec
   *  can add per-platform "verify-only" commands.
   */
  async function testChannel(c: ChannelConfig) {
    testStatus[c.id] = { kind: 'pending' };
    try {
      // We don't actually have a verify-only IPC. Use an upload of a 1×1
      // PNG to the platform's media endpoint as a connectivity smoke
      // test — if it succeeds we know auth works; failures map to the
      // same PublishError variants we surface in the dialog.
      const onePixelPng = Uint8Array.from(
        atob('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/x8AAusB9Zy5HJ4AAAAASUVORK5CYII='),
        ch => ch.charCodeAt(0),
      );
      const args = { bytes: Array.from(onePixelPng), filename: 'novelist-test.png', mime: 'image/png', config: toPlatformConfig(c) };
      let r;
      switch (c.platform) {
        case 'ghost':                 r = await commands.uploadPostImageGhost(args.bytes, args.filename, args.mime, args.config); break;
        case 'wordpress_self_hosted': r = await commands.uploadPostImageWordpressSelfHosted(args.bytes, args.filename, args.mime, args.config); break;
        case 'wordpress_com':         r = await commands.uploadPostImageWordpressCom(args.bytes, args.filename, args.mime, args.config); break;
        case 'medium':                r = await commands.uploadPostImageMedium(args.bytes, args.filename, args.mime, args.config); break;
      }
      if (r.status !== 'ok') {
        testStatus[c.id] = { kind: 'err', message: r.error };
      } else {
        testStatus[c.id] = { kind: 'ok', message: r.data.url };
      }
    } catch (e) {
      testStatus[c.id] = { kind: 'err', message: e instanceof Error ? e.message : String(e) };
    }
  }

  function toPlatformConfig(c: ChannelConfig): PlatformConfig {
    const { id: _id, name: _name, ...rest } = c;
    return rest as PlatformConfig;
  }

  function fieldsFor(c: ChannelConfig): Array<{ key: string; label: string; placeholder?: string; secret?: boolean }> {
    switch (c.platform) {
      case 'ghost': return [
        { key: 'admin_url', label: 'Admin URL', placeholder: 'https://blog.example.com' },
        { key: 'api_key',   label: 'Admin API key (id:secret)', secret: true },
      ];
      case 'wordpress_self_hosted': return [
        { key: 'site_url',     label: 'Site URL', placeholder: 'https://blog.example.com' },
        { key: 'username',     label: 'WP username' },
        { key: 'app_password', label: 'Application password', secret: true },
      ];
      case 'wordpress_com': return [
        { key: 'site_id_or_domain', label: 'Site domain or id', placeholder: 'myblog.wordpress.com' },
        { key: 'access_token',      label: 'OAuth2 access token', secret: true },
      ];
      case 'medium': return [
        { key: 'token', label: 'Integration token (legacy)', secret: true },
      ];
    }
  }
</script>

<div class="publish-panel">
  <h3 class="text-xs font-semibold uppercase tracking-wide mb-4" style="color: var(--novelist-text-secondary);">Publish</h3>

  {#if loading}
    <div class="text-sm" style="color: var(--novelist-text-secondary);">Loading…</div>
  {:else}
    <div class="text-xs uppercase tracking-wide mb-2" style="color: var(--novelist-text-secondary);">Configured channels</div>

    {#if settings.channels.length === 0}
      <div class="text-sm mb-3" style="color: var(--novelist-text-secondary);">No publish channel configured yet. Add one below.</div>
    {:else}
      <ul class="space-y-2 mb-4">
        {#each settings.channels as ch (ch.id)}
          <li class="flex items-center justify-between gap-3 p-2 rounded" style="border: 1px solid var(--novelist-border);">
            <div class="flex flex-col flex-1">
              <span class="text-sm font-medium">{ch.name}</span>
              <span class="text-xs" style="color: var(--novelist-text-secondary);">{platformLabel(ch.platform)}</span>
            </div>
            {#if testStatus[ch.id]?.kind === 'pending'}
              <span class="text-xs" style="color: var(--novelist-text-secondary);">Testing…</span>
            {:else if testStatus[ch.id]?.kind === 'ok'}
              <span class="text-xs" style="color: #2da84a;" title={testStatus[ch.id].message}>✓ test ok</span>
            {:else if testStatus[ch.id]?.kind === 'err'}
              <span class="text-xs" style="color: #d24a4a;" title={testStatus[ch.id].message}>✗ test failed</span>
            {/if}
            <button class="text-xs px-2 py-1 cursor-pointer" onclick={() => testChannel(ch)} style="border: 1px solid var(--novelist-border); background: transparent;">Test</button>
            <button class="text-xs px-2 py-1 cursor-pointer" onclick={() => startEdit(ch)} style="border: 1px solid var(--novelist-border); background: transparent;">Edit</button>
            <button class="text-xs px-2 py-1 cursor-pointer" onclick={() => removeChannel(ch.id)} style="border: 1px solid var(--novelist-border); background: transparent; color: #d24a4a;">Delete</button>
          </li>
        {/each}
      </ul>
    {/if}

    <div class="text-xs uppercase tracking-wide mb-2" style="color: var(--novelist-text-secondary);">Add a channel</div>
    <div class="flex flex-wrap gap-2 mb-4">
      {#each ['ghost','wordpress_self_hosted','wordpress_com','medium'] as PlatformId[] as p}
        <button class="text-xs px-3 py-1.5 cursor-pointer rounded"
                style="border: 1px solid var(--novelist-border); background: transparent;"
                onclick={() => startAdd(p)}>{platformLabel(p)}</button>
      {/each}
    </div>

    {#if saveError}
      <div class="text-xs mb-3" style="color: #d24a4a;">Failed to save: {saveError}</div>
    {/if}
  {/if}

  {#if editing}
    <div class="modal-backdrop"
         onclick={(e) => { if (e.target === e.currentTarget) cancelEdit(); }}
         onkeydown={(e) => e.key === 'Escape' && cancelEdit()}
         role="button" tabindex="0">
      <div class="modal" role="dialog" tabindex="-1">
        <h4 class="text-sm font-semibold mb-3">{editing.isNew ? 'Add' : 'Edit'} {platformLabel(editing.channel.platform)} channel</h4>

        <div class="flex items-center justify-between mb-3 gap-2">
          <label class="text-xs w-32" for="ch-name">Name</label>
          <input id="ch-name" type="text" class="text-sm flex-1 px-2 py-1 rounded"
                 style="border: 1px solid var(--novelist-border); background: var(--novelist-bg);"
                 bind:value={editing.channel.name} />
        </div>

        {#each fieldsFor(editing.channel) as f}
          <div class="flex items-center justify-between mb-3 gap-2">
            <label class="text-xs w-32" for={`ch-${f.key}`}>{f.label}</label>
            <input id={`ch-${f.key}`}
                   type={f.secret ? 'password' : 'text'}
                   placeholder={f.placeholder ?? ''}
                   class="text-sm flex-1 px-2 py-1 rounded"
                   style="border: 1px solid var(--novelist-border); background: var(--novelist-bg);"
                   value={(editing.channel as unknown as Record<string, string>)[f.key] ?? ''}
                   oninput={(e) => {
                     const v = (e.currentTarget as HTMLInputElement).value;
                     (editing!.channel as unknown as Record<string, string>)[f.key] = v;
                   }} />
          </div>
        {/each}

        <div class="flex justify-end gap-2 mt-4">
          <button class="text-xs px-3 py-1.5 cursor-pointer rounded" onclick={cancelEdit}
                  style="border: 1px solid var(--novelist-border); background: transparent;">Cancel</button>
          <button class="text-xs px-3 py-1.5 cursor-pointer rounded" onclick={saveEdit}
                  style="border: none; background: var(--novelist-accent); color: white;">{editing.isNew ? 'Add' : 'Save'}</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .modal-backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    z-index: 1000;
  }
  .modal {
    background: var(--novelist-bg);
    border: 1px solid var(--novelist-border);
    border-radius: 8px;
    padding: 16px;
    width: 480px;
    max-width: 90vw;
    max-height: 80vh;
    overflow-y: auto;
  }
</style>
