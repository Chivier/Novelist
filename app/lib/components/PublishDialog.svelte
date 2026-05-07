<script lang="ts">
  import type { ChannelConfig, PlatformConfig } from '$lib/ipc/commands';
  import { dispatchPublish, type DialogPayload } from '$lib/services/publish';
  import { t } from '$lib/i18n';

  interface Props {
    channel: ChannelConfig;
    doc: { dir: string; text: string };
    onClose: () => void;
  }
  let { channel, doc, onClose }: Props = $props();

  // Pre-fill title from H1 of the doc, or use empty.
  function extractH1(text: string): string {
    const m = text.match(/^#\s+(.+)$/m);
    return m ? m[1].trim() : '';
  }

  // ASCII-only kebab-case from a title; falls back to "post" for empty/CJK.
  function slugify(s: string): string {
    const cleaned = s
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')
      .trim()
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-');
    return cleaned || 'post';
  }

  function statusOptionsFor(platform: PlatformConfig['platform']): { value: string; label: string }[] {
    switch (platform) {
      case 'ghost':
        return [{ value: 'draft', label: 'Draft' }, { value: 'published', label: 'Published' }];
      case 'wordpress_self_hosted':
      case 'wordpress_com':
        return [
          { value: 'draft', label: 'Draft' },
          { value: 'pending', label: 'Pending review' },
          { value: 'private', label: 'Private' },
          { value: 'publish', label: 'Publish' },
        ];
      case 'medium':
        return [
          { value: 'draft', label: 'Draft' },
          { value: 'unlisted', label: 'Unlisted' },
          { value: 'public', label: 'Public' },
        ];
    }
  }

  function defaultStatusFor(platform: PlatformConfig['platform']): string {
    return platform === 'medium' ? 'public' : 'draft';
  }

  function baseUrlForChannel(c: ChannelConfig): string {
    switch (c.platform) {
      case 'ghost':                 return c.admin_url;
      case 'wordpress_self_hosted': return c.site_url;
      case 'wordpress_com':         return `https://${c.site_id_or_domain}`;
      case 'medium':                return 'https://medium.com';
    }
  }

  // svelte-ignore state_referenced_locally
  let title = $state(extractH1(doc.text));
  let tagInput = $state('');
  let tags = $state<string[]>([]);
  let excerpt = $state('');
  // svelte-ignore state_referenced_locally
  let slug = $state(slugify(extractH1(doc.text)));
  // svelte-ignore state_referenced_locally
  let status = $state(defaultStatusFor(channel.platform));
  let coverFile = $state<File | null>(null);
  let coverPreviewUrl = $state<string | null>(null);

  let publishing = $state(false);
  let errorMessage = $state<string | null>(null);
  let successUrl = $state<string | null>(null);

  function addTagFromInput() {
    const v = tagInput.trim();
    if (v && !tags.includes(v)) tags = [...tags, v];
    tagInput = '';
  }
  function removeTag(t: string) { tags = tags.filter(x => x !== t); }

  function onTagKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTagFromInput();
    } else if (e.key === 'Backspace' && tagInput === '' && tags.length > 0) {
      tags = tags.slice(0, -1);
    }
  }

  async function pickCover() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*';
    input.addEventListener('change', () => {
      const f = input.files?.[0];
      if (f) setCover(f);
    });
    input.click();
  }

  function setCover(f: File) {
    coverFile = f;
    if (coverPreviewUrl) URL.revokeObjectURL(coverPreviewUrl);
    coverPreviewUrl = URL.createObjectURL(f);
  }

  function clearCover() {
    coverFile = null;
    if (coverPreviewUrl) URL.revokeObjectURL(coverPreviewUrl);
    coverPreviewUrl = null;
  }

  function onCoverDrop(e: DragEvent) {
    e.preventDefault();
    const f = e.dataTransfer?.files?.[0];
    if (f && f.type.startsWith('image/')) setCover(f);
  }

  async function doPublish() {
    if (!title.trim()) {
      errorMessage = t('publish.titleRequired');
      return;
    }
    publishing = true;
    errorMessage = null;
    successUrl = null;

    try {
      const payload: DialogPayload = {
        title: title.trim(),
        tags,
        slug: slug.trim() || undefined,
        excerpt: excerpt.trim() || undefined,
        status,
      };
      if (coverFile) {
        const buf = await coverFile.arrayBuffer();
        payload.coverImage = {
          bytes: new Uint8Array(buf),
          filename: coverFile.name,
          mime: coverFile.type || 'image/png',
        };
      }
      const result = await dispatchPublish(channel, payload, doc);
      successUrl = result.url;
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      publishing = false;
    }
  }

  function openInBrowser() {
    if (successUrl) window.open(successUrl, '_blank');
  }
</script>

<div
  class="modal-backdrop"
  onclick={(e) => { if (e.target === e.currentTarget && !publishing) onClose(); }}
  onkeydown={(e) => { if (e.key === 'Escape' && !publishing) onClose(); }}
  role="button"
  tabindex="0"
>
  <div class="modal" role="dialog" tabindex="-1">
    <div class="header">
      <div class="header-name">{channel.name}</div>
      <div class="header-url">{baseUrlForChannel(channel)}</div>
    </div>

    {#if successUrl}
      <div class="success-banner">
        <span>{t('publish.success')}</span>
        <button class="link-btn" onclick={openInBrowser}>{t('publish.openInBrowser')}</button>
        <button class="close-btn" onclick={onClose}>{t('publish.close')}</button>
      </div>
    {:else}
      <div class="form">
        <label for="pub-title" class="lbl">{t('publish.title')} <span class="req">*</span></label>
        <input id="pub-title" type="text" class="inp" bind:value={title} />

        <div class="row">
          <div class="col">
            <span class="lbl">{t('publish.coverImage')}</span>
            <div
              class="cover-drop"
              role="button"
              tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onCoverDrop}
              onclick={pickCover}
              onkeydown={(e) => { if (e.key === 'Enter') pickCover(); }}
            >
              {#if coverPreviewUrl}
                <img src={coverPreviewUrl} alt="cover preview" />
              {:else}
                <div class="cover-placeholder">{t('publish.coverPlaceholder')}</div>
              {/if}
            </div>
            <div class="cover-actions">
              <button class="small-btn" onclick={pickCover}>{t('publish.choose')}</button>
              {#if coverFile}<button class="small-btn" onclick={clearCover}>{t('publish.remove')}</button>{/if}
            </div>
          </div>

          <div class="col">
            <label for="pub-tags" class="lbl">{t('publish.tags')}</label>
            <div class="tag-row">
              {#each tags as tag}
                <span class="tag-chip">{tag} <button class="chip-x" onclick={() => removeTag(tag)}>×</button></span>
              {/each}
              <input
                id="pub-tags"
                type="text"
                class="tag-inp"
                bind:value={tagInput}
                onkeydown={onTagKeydown}
                onblur={addTagFromInput}
                placeholder={tags.length === 0 ? t('publish.tagsPlaceholder') : ''}
              />
            </div>

            <label for="pub-excerpt" class="lbl">{t('publish.excerpt')}</label>
            <textarea id="pub-excerpt" class="inp" rows="3" bind:value={excerpt}></textarea>

            <label for="pub-slug" class="lbl">{t('publish.slug')}</label>
            <input id="pub-slug" type="text" class="inp" bind:value={slug} />
          </div>
        </div>

        <div class="row">
          <label for="pub-status" class="lbl status-lbl">{t('publish.status')}</label>
          <select id="pub-status" class="inp inp-select" bind:value={status}>
            {#each statusOptionsFor(channel.platform) as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        {#if errorMessage}
          <div class="error-banner">{errorMessage}</div>
        {/if}

        <div class="footer">
          <button class="ghost-btn" onclick={onClose} disabled={publishing}>{t('publish.cancel')}</button>
          <button class="primary-btn" onclick={doPublish} disabled={publishing}>
            {publishing ? t('publish.publishing') : t('publish.publish')}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    z-index: 2000;
  }
  .modal {
    background: var(--novelist-bg);
    border: 1px solid var(--novelist-border);
    border-radius: 8px;
    padding: 16px;
    width: 720px;
    max-width: 90vw;
    max-height: 90vh;
    overflow-y: auto;
  }
  .header {
    border-bottom: 1px solid var(--novelist-border);
    padding-bottom: 8px; margin-bottom: 12px;
  }
  .header-name { font-weight: 600; font-size: 14px; }
  .header-url { font-size: 12px; color: var(--novelist-text-secondary); }

  .form { display: flex; flex-direction: column; gap: 8px; }
  .lbl { font-size: 12px; color: var(--novelist-text-secondary); margin-top: 6px; display: block; }
  .req { color: #d24a4a; }
  .inp {
    width: 100%; padding: 6px 8px; border-radius: 4px;
    border: 1px solid var(--novelist-border); background: var(--novelist-bg);
    color: var(--novelist-text); font-size: 14px;
  }
  .inp-select { width: auto; padding-right: 24px; }
  .row { display: flex; gap: 16px; align-items: flex-start; }
  .col { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .status-lbl { margin-top: 0; align-self: center; }

  .cover-drop {
    border: 1px dashed var(--novelist-border);
    border-radius: 4px;
    height: 140px;
    display: flex; align-items: center; justify-content: center;
    overflow: hidden;
    cursor: pointer;
  }
  .cover-drop img { max-width: 100%; max-height: 100%; }
  .cover-placeholder { font-size: 12px; color: var(--novelist-text-secondary); padding: 0 8px; text-align: center; }
  .cover-actions { display: flex; gap: 6px; margin-top: 6px; }
  .small-btn {
    font-size: 11px; padding: 3px 8px;
    border: 1px solid var(--novelist-border); background: transparent; border-radius: 3px;
    cursor: pointer;
  }

  .tag-row {
    display: flex; flex-wrap: wrap; gap: 4px;
    border: 1px solid var(--novelist-border); border-radius: 4px;
    padding: 4px; min-height: 28px; align-items: center;
  }
  .tag-chip {
    background: color-mix(in srgb, var(--novelist-accent) 16%, transparent);
    border-radius: 3px; padding: 2px 6px;
    font-size: 12px; display: inline-flex; gap: 4px; align-items: center;
  }
  .chip-x { background: none; border: none; cursor: pointer; font-size: 12px; padding: 0; line-height: 1; }
  .tag-inp { flex: 1; min-width: 80px; border: none; outline: none; background: transparent; font-size: 12px; }

  .error-banner {
    background: rgba(210, 74, 74, 0.12);
    border-left: 3px solid #d24a4a;
    padding: 6px 10px;
    font-size: 12px;
    color: #d24a4a;
    margin-top: 8px;
    border-radius: 0 3px 3px 0;
  }

  .success-banner {
    display: flex; gap: 12px; align-items: center;
    background: rgba(45, 168, 74, 0.12);
    border-left: 3px solid #2da84a;
    padding: 8px 12px;
    border-radius: 0 3px 3px 0;
    font-size: 14px;
  }

  .footer {
    display: flex; justify-content: flex-end; gap: 8px;
    margin-top: 12px; padding-top: 8px;
    border-top: 1px solid var(--novelist-border);
  }
  .ghost-btn {
    padding: 6px 14px; font-size: 14px;
    border: 1px solid var(--novelist-border); background: transparent;
    border-radius: 4px; cursor: pointer;
  }
  .primary-btn {
    padding: 6px 14px; font-size: 14px;
    border: none; background: var(--novelist-accent); color: white;
    border-radius: 4px; cursor: pointer;
  }
  .primary-btn:disabled, .ghost-btn:disabled { opacity: 0.5; cursor: default; }
  .link-btn { background: none; border: none; color: var(--novelist-accent); cursor: pointer; padding: 0; font-size: 14px; text-decoration: underline; }
  .close-btn { background: none; border: 1px solid var(--novelist-border); border-radius: 3px; padding: 2px 8px; font-size: 12px; cursor: pointer; margin-left: auto; }
</style>
