<script lang="ts">
  import { onMount } from 'svelte';
  import { bridge, aiStream, whenHostReady } from './bridge';
  import { buildChatRequest, parseChatDelta, type ChatMessage } from './openai';
  import { createSettingsStore } from './settings.svelte';

  const settings = createSettingsStore();

  type Tab = 'chat' | 'rewrite';
  let activeTab = $state<Tab>('chat');
  let settingsOpen = $state(false);

  // ------------------------------- Chat -------------------------------

  type DisplayMessage = { role: 'user' | 'assistant'; content: string };
  const HISTORY_KEY = 'yolo:chat-history:v1';

  let messages = $state<DisplayMessage[]>(loadHistory());
  let chatInput = $state('');
  let chatStreaming = $state(false);
  let chatStreamId: string | null = null;
  let chatScroller = $state<HTMLDivElement | undefined>(undefined);

  function loadHistory(): DisplayMessage[] {
    try {
      const raw = localStorage.getItem(HISTORY_KEY);
      if (!raw) return [];
      const data = JSON.parse(raw);
      return Array.isArray(data) ? data : [];
    } catch {
      return [];
    }
  }

  function saveHistory() {
    try {
      localStorage.setItem(HISTORY_KEY, JSON.stringify(messages));
    } catch {
      /* ignore */
    }
  }

  async function buildChatContext(userText: string): Promise<ChatMessage[]> {
    const ctx: ChatMessage[] = [];
    if (settings.value.systemPrompt.trim()) {
      ctx.push({ role: 'system', content: settings.value.systemPrompt });
    }

    const snap = await bridge.getSelection().catch(() => null);
    if (snap) {
      if (settings.value.includeCurrentFile && snap.fullDoc.trim()) {
        ctx.push({
          role: 'user',
          content: `The user is currently editing "${snap.filePath ?? 'untitled'}". Document contents:\n\n${snap.fullDoc}`,
        });
      } else if (settings.value.includeSelection && snap.text.trim()) {
        ctx.push({
          role: 'user',
          content: `Selected text the user is asking about:\n\n${snap.text}`,
        });
      }
    }

    for (const m of messages) {
      ctx.push({ role: m.role, content: m.content });
    }
    ctx.push({ role: 'user', content: userText });
    return ctx;
  }

  async function sendChat() {
    const text = chatInput.trim();
    if (!text || chatStreaming) return;
    if (!settings.value.apiKey) {
      messages = [...messages, { role: 'assistant', content: '⚠️ Set an API key in Settings first.' }];
      saveHistory();
      return;
    }
    messages = [...messages, { role: 'user', content: text }];
    chatInput = '';
    saveHistory();

    const assistantIdx = messages.length;
    messages = [...messages, { role: 'assistant', content: '' }];
    chatStreaming = true;

    let buffered = '';
    try {
      const req = buildChatRequest({
        baseUrl: settings.value.baseUrl,
        apiKey: settings.value.apiKey,
        model: settings.value.model,
        temperature: settings.value.temperature,
        messages: await buildChatContext(text),
      });
      chatStreamId = await bridge.startAiStream(req);
      for await (const ev of aiStream(chatStreamId)) {
        if (ev.kind === 'chunk') {
          const delta = parseChatDelta(ev.data);
          if (delta) {
            buffered += delta;
            messages[assistantIdx] = { role: 'assistant', content: buffered };
            scrollChat();
          }
        } else if (ev.kind === 'error') {
          messages[assistantIdx] = {
            role: 'assistant',
            content: `${buffered}\n\n⚠️ ${ev.message}${ev.status ? ` (HTTP ${ev.status})` : ''}`,
          };
        }
      }
    } catch (e) {
      messages[assistantIdx] = {
        role: 'assistant',
        content: `${buffered}\n\n⚠️ ${e instanceof Error ? e.message : String(e)}`,
      };
    } finally {
      chatStreaming = false;
      chatStreamId = null;
      saveHistory();
    }
  }

  async function cancelChat() {
    if (chatStreamId) {
      const id = chatStreamId;
      chatStreamId = null;
      await bridge.cancelAiStream(id).catch(() => {});
    }
    chatStreaming = false;
  }

  function clearChat() {
    messages = [];
    saveHistory();
  }

  function scrollChat() {
    queueMicrotask(() => {
      if (chatScroller) chatScroller.scrollTop = chatScroller.scrollHeight;
    });
  }

  function chatKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      void sendChat();
    }
  }

  // ------------------------------ Rewrite ------------------------------

  type RewriteSnap = { from: number; to: number; original: string };

  let rewriteInstr = $state('');
  let rewriteSnap = $state<RewriteSnap | null>(null);
  let rewriteOutput = $state('');
  let rewriteStreaming = $state(false);
  let rewriteStreamId: string | null = null;
  let rewriteError = $state('');

  async function captureSelection() {
    rewriteError = '';
    const snap = await bridge.getSelection().catch(() => null);
    if (!snap || !snap.text.trim()) {
      rewriteError = 'No text selected in the editor.';
      rewriteSnap = null;
      return;
    }
    rewriteSnap = { from: snap.from, to: snap.to, original: snap.text };
    rewriteOutput = '';
  }

  async function runRewrite() {
    if (!rewriteSnap || !rewriteInstr.trim() || rewriteStreaming) return;
    if (!settings.value.apiKey) {
      rewriteError = 'Set an API key in Settings first.';
      return;
    }
    rewriteError = '';
    rewriteOutput = '';
    rewriteStreaming = true;

    try {
      const req = buildChatRequest({
        baseUrl: settings.value.baseUrl,
        apiKey: settings.value.apiKey,
        model: settings.value.model,
        temperature: settings.value.temperature,
        messages: [
          {
            role: 'system',
            content:
              'You rewrite text per the user instruction. Return ONLY the rewritten text — no preamble, no markdown fences, no commentary.',
          },
          {
            role: 'user',
            content: `Instruction: ${rewriteInstr}\n\nText to rewrite:\n${rewriteSnap.original}`,
          },
        ],
      });
      rewriteStreamId = await bridge.startAiStream(req);
      for await (const ev of aiStream(rewriteStreamId)) {
        if (ev.kind === 'chunk') {
          const delta = parseChatDelta(ev.data);
          if (delta) rewriteOutput += delta;
        } else if (ev.kind === 'error') {
          rewriteError = `${ev.message}${ev.status ? ` (HTTP ${ev.status})` : ''}`;
        }
      }
    } catch (e) {
      rewriteError = e instanceof Error ? e.message : String(e);
    } finally {
      rewriteStreaming = false;
      rewriteStreamId = null;
    }
  }

  async function cancelRewrite() {
    if (rewriteStreamId) {
      const id = rewriteStreamId;
      rewriteStreamId = null;
      await bridge.cancelAiStream(id).catch(() => {});
    }
    rewriteStreaming = false;
  }

  async function acceptRewrite() {
    if (!rewriteSnap || !rewriteOutput) return;
    await bridge.replaceRange({
      from: rewriteSnap.from,
      to: rewriteSnap.to,
      text: rewriteOutput,
    });
    rewriteSnap = null;
    rewriteOutput = '';
    rewriteInstr = '';
  }

  function rejectRewrite() {
    rewriteOutput = '';
  }

  // ------------------------------ Lifecycle ----------------------------

  let permissionsBanner = $state('');
  onMount(() => {
    void whenHostReady().then((info) => {
      if (!info.permissions.includes('ai:http')) {
        permissionsBanner = "This plugin needs the 'ai:http' permission. Add it to manifest.toml.";
      }
    });
  });
</script>

<main>
  {#if permissionsBanner}
    <div class="banner">{permissionsBanner}</div>
  {/if}

  <header>
    <div class="tabs">
      <button class:active={activeTab === 'chat'} onclick={() => (activeTab = 'chat')}>Chat</button>
      <button class:active={activeTab === 'rewrite'} onclick={() => (activeTab = 'rewrite')}>Rewrite</button>
    </div>
    <button class="gear" title="Settings" onclick={() => (settingsOpen = !settingsOpen)}>⚙</button>
  </header>

  {#if settingsOpen}
    <section class="settings">
      <label>
        <span>Base URL</span>
        <input
          type="text"
          value={settings.value.baseUrl}
          oninput={(e) => settings.update({ baseUrl: e.currentTarget.value })}
        />
      </label>
      <label>
        <span>API Key</span>
        <input
          type="password"
          value={settings.value.apiKey}
          oninput={(e) => settings.update({ apiKey: e.currentTarget.value })}
          placeholder="sk-…"
        />
      </label>
      <label>
        <span>Model</span>
        <input
          type="text"
          value={settings.value.model}
          oninput={(e) => settings.update({ model: e.currentTarget.value })}
        />
      </label>
      <label>
        <span>Temperature</span>
        <input
          type="number"
          step="0.1"
          min="0"
          max="2"
          value={settings.value.temperature}
          oninput={(e) => settings.update({ temperature: Number(e.currentTarget.value) })}
        />
      </label>
      <label class="full">
        <span>System prompt</span>
        <textarea
          rows="3"
          value={settings.value.systemPrompt}
          oninput={(e) => settings.update({ systemPrompt: e.currentTarget.value })}
        ></textarea>
      </label>
      <label class="check">
        <input
          type="checkbox"
          checked={settings.value.includeCurrentFile}
          onchange={(e) => settings.update({ includeCurrentFile: e.currentTarget.checked })}
        />
        <span>Include current file in chat context</span>
      </label>
      <label class="check">
        <input
          type="checkbox"
          checked={settings.value.includeSelection}
          onchange={(e) => settings.update({ includeSelection: e.currentTarget.checked })}
        />
        <span>Include current selection in chat context</span>
      </label>
      <p class="hint">
        API key is stored in this iframe's localStorage (origin: <code>asset://…/yolo/</code>) and never sent to Novelist's
        backend or other plugins.
      </p>
    </section>
  {/if}

  {#if activeTab === 'chat'}
    <div class="chat" bind:this={chatScroller}>
      {#each messages as m (m)}
        <div class="msg {m.role}">
          <div class="role">{m.role === 'user' ? 'You' : 'Assistant'}</div>
          <div class="content">{m.content}</div>
        </div>
      {/each}
      {#if messages.length === 0}
        <div class="empty">
          <p>Start a conversation. <kbd>Cmd</kbd>+<kbd>Enter</kbd> to send.</p>
        </div>
      {/if}
    </div>
    <div class="composer">
      <textarea
        rows="3"
        placeholder="Ask anything…"
        value={chatInput}
        oninput={(e) => (chatInput = e.currentTarget.value)}
        onkeydown={chatKeydown}
      ></textarea>
      <div class="composer-actions">
        <button class="ghost" onclick={clearChat} disabled={chatStreaming}>Clear</button>
        {#if chatStreaming}
          <button class="primary" onclick={cancelChat}>Stop</button>
        {:else}
          <button class="primary" onclick={sendChat} disabled={!chatInput.trim()}>Send</button>
        {/if}
      </div>
    </div>
  {:else}
    <div class="rewrite">
      <div class="row">
        <button class="primary" onclick={captureSelection}>Use current selection</button>
        {#if rewriteSnap}
          <span class="meta">{rewriteSnap.original.length} chars captured</span>
        {/if}
      </div>
      {#if rewriteSnap}
        <details open>
          <summary>Original</summary>
          <pre>{rewriteSnap.original}</pre>
        </details>
        <textarea
          rows="2"
          placeholder="Instruction (e.g. 'tighten this paragraph', 'translate to Chinese')"
          value={rewriteInstr}
          oninput={(e) => (rewriteInstr = e.currentTarget.value)}
        ></textarea>
        <div class="row">
          {#if rewriteStreaming}
            <button class="primary" onclick={cancelRewrite}>Stop</button>
          {:else}
            <button class="primary" onclick={runRewrite} disabled={!rewriteInstr.trim()}>Rewrite</button>
          {/if}
        </div>
      {/if}
      {#if rewriteOutput}
        <details open>
          <summary>Rewritten</summary>
          <pre>{rewriteOutput}</pre>
        </details>
        <div class="row">
          <button class="primary" onclick={acceptRewrite} disabled={rewriteStreaming}>Accept &amp; replace</button>
          <button class="ghost" onclick={rejectRewrite} disabled={rewriteStreaming}>Discard</button>
        </div>
      {/if}
      {#if rewriteError}
        <div class="banner">{rewriteError}</div>
      {/if}
    </div>
  {/if}
</main>

<style>
  :global(html), :global(body) {
    margin: 0;
    padding: 0;
    height: 100%;
  }
  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 13px;
    color: var(--novelist-text, #1f2937);
    background: var(--novelist-bg, #fff);
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }
  .banner {
    background: #fef3c7;
    color: #78350f;
    padding: 6px 10px;
    font-size: 12px;
    border-bottom: 1px solid var(--novelist-border, #e5e7eb);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-bottom: 1px solid var(--novelist-border, #e5e7eb);
    background: var(--novelist-bg-secondary, #f9fafb);
  }
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tabs button {
    background: none;
    border: 1px solid transparent;
    padding: 4px 10px;
    border-radius: 4px;
    color: var(--novelist-text-secondary, #6b7280);
    cursor: pointer;
    font-size: 12px;
  }
  .tabs button.active {
    color: var(--novelist-text, #1f2937);
    border-color: var(--novelist-border, #e5e7eb);
    background: var(--novelist-bg, #fff);
  }
  .gear {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    color: var(--novelist-text-secondary, #6b7280);
  }
  .settings {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 10px;
    background: var(--novelist-bg-secondary, #f9fafb);
    border-bottom: 1px solid var(--novelist-border, #e5e7eb);
  }
  .settings label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
    color: var(--novelist-text-secondary, #6b7280);
  }
  .settings label.full {
    grid-column: 1 / -1;
  }
  .settings label.check {
    grid-column: 1 / -1;
    flex-direction: row;
    align-items: center;
    gap: 6px;
    color: var(--novelist-text, #1f2937);
    font-size: 12px;
  }
  .settings input,
  .settings textarea {
    background: var(--novelist-bg, #fff);
    border: 1px solid var(--novelist-border, #e5e7eb);
    color: var(--novelist-text, #1f2937);
    padding: 4px 6px;
    border-radius: 3px;
    font: inherit;
    font-size: 12px;
  }
  .settings .hint {
    grid-column: 1 / -1;
    margin: 0;
    font-size: 11px;
    color: var(--novelist-text-secondary, #6b7280);
  }
  .settings code {
    background: var(--novelist-bg, #fff);
    padding: 1px 4px;
    border-radius: 2px;
  }
  .chat {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .msg {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .msg .role {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--novelist-text-secondary, #6b7280);
  }
  .msg .content {
    white-space: pre-wrap;
    word-wrap: break-word;
    padding: 6px 8px;
    border-radius: 6px;
    background: var(--novelist-bg-secondary, #f3f4f6);
  }
  .msg.user .content {
    background: var(--novelist-accent, #2563eb);
    color: #fff;
    align-self: flex-end;
    max-width: 85%;
  }
  .empty {
    color: var(--novelist-text-secondary, #6b7280);
    text-align: center;
    margin-top: 30%;
    font-size: 12px;
  }
  kbd {
    background: var(--novelist-bg-secondary, #f3f4f6);
    border: 1px solid var(--novelist-border, #d1d5db);
    border-radius: 3px;
    padding: 1px 4px;
    font-size: 11px;
  }
  .composer {
    border-top: 1px solid var(--novelist-border, #e5e7eb);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--novelist-bg-secondary, #f9fafb);
  }
  .composer textarea {
    width: 100%;
    box-sizing: border-box;
    background: var(--novelist-bg, #fff);
    border: 1px solid var(--novelist-border, #e5e7eb);
    color: var(--novelist-text, #1f2937);
    border-radius: 4px;
    padding: 6px 8px;
    font: inherit;
    resize: vertical;
  }
  .composer-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
  button.primary {
    background: var(--novelist-accent, #2563eb);
    color: #fff;
    border: none;
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }
  button.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button.ghost {
    background: none;
    color: var(--novelist-text-secondary, #6b7280);
    border: 1px solid var(--novelist-border, #e5e7eb);
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }
  .rewrite {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .rewrite .row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .rewrite .meta {
    font-size: 11px;
    color: var(--novelist-text-secondary, #6b7280);
  }
  .rewrite textarea {
    width: 100%;
    box-sizing: border-box;
    background: var(--novelist-bg, #fff);
    border: 1px solid var(--novelist-border, #e5e7eb);
    color: var(--novelist-text, #1f2937);
    border-radius: 4px;
    padding: 6px 8px;
    font: inherit;
    resize: vertical;
  }
  .rewrite details {
    border: 1px solid var(--novelist-border, #e5e7eb);
    border-radius: 4px;
  }
  .rewrite summary {
    cursor: pointer;
    padding: 4px 8px;
    font-size: 11px;
    color: var(--novelist-text-secondary, #6b7280);
    background: var(--novelist-bg-secondary, #f9fafb);
    user-select: none;
  }
  .rewrite pre {
    margin: 0;
    padding: 8px;
    white-space: pre-wrap;
    word-wrap: break-word;
    font-family: inherit;
    font-size: 12px;
  }
</style>
