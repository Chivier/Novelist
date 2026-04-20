/**
 * Bridge SDK for the YOLO plugin. Wraps Novelist's host-iframe postMessage
 * protocol in a small async API so UI code can `await` requests and consume
 * stream events as async iterators.
 */

type BridgeRequest = {
  type: 'bridge:request';
  id: string;
  method: string;
  params?: unknown;
};

type BridgeResponse = {
  type: 'bridge:response';
  id: string;
  ok: boolean;
  data?: unknown;
  error?: string;
};

type BridgeEvent = {
  type: 'bridge:event';
  event: string;
  payload: unknown;
};

type HostReady = {
  type: 'bridge:host-ready';
  pluginId: string;
  permissions: string[];
};

let nextId = 0;
const pending = new Map<string, { resolve: (data: unknown) => void; reject: (err: Error) => void }>();
const eventSubscribers = new Map<string, Set<(payload: unknown) => void>>();

let hostReadyResolve: ((info: { pluginId: string; permissions: string[] }) => void) | null = null;
const hostReady = new Promise<{ pluginId: string; permissions: string[] }>((resolve) => {
  hostReadyResolve = resolve;
});

window.addEventListener('message', (event: MessageEvent) => {
  const data = event.data;
  if (!data || typeof data !== 'object') return;

  if (data.type === 'bridge:host-ready') {
    const ready = data as HostReady;
    hostReadyResolve?.({ pluginId: ready.pluginId, permissions: ready.permissions ?? [] });
    return;
  }

  if (data.type === 'bridge:response') {
    const res = data as BridgeResponse;
    const slot = pending.get(res.id);
    if (!slot) return;
    pending.delete(res.id);
    if (res.ok) slot.resolve(res.data);
    else slot.reject(new Error(res.error ?? 'bridge error'));
    return;
  }

  if (data.type === 'bridge:event') {
    const ev = data as BridgeEvent;
    const subs = eventSubscribers.get(ev.event);
    if (!subs) return;
    for (const fn of subs) fn(ev.payload);
    return;
  }

  if (data.type === 'theme-update') {
    document.dispatchEvent(new CustomEvent('yolo:theme-update', { detail: data.theme }));
  }
});

function postRequest(method: string, params?: unknown): Promise<unknown> {
  const id = `req-${++nextId}`;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    const msg: BridgeRequest = { type: 'bridge:request', id, method, params };
    window.parent.postMessage(msg, '*');
  });
}

export function whenHostReady(): Promise<{ pluginId: string; permissions: string[] }> {
  // Plugin may load before or after host posts host-ready, so kick a request
  // as a fallback to learn permissions even if we missed the proactive post.
  void postRequest('bridge.ready')
    .then((info) => {
      const i = info as { pluginId?: string; permissions?: string[] } | undefined;
      hostReadyResolve?.({
        pluginId: i?.pluginId ?? '',
        permissions: i?.permissions ?? [],
      });
    })
    .catch(() => {
      /* ignore; the proactive host-ready may still arrive */
    });
  return hostReady;
}

export const bridge = {
  request: postRequest,

  async getTheme(): Promise<Record<string, string>> {
    return (await postRequest('theme.get')) as Record<string, string>;
  },

  async getSelection(): Promise<{
    tabId: string;
    filePath: string | null;
    fullDoc: string;
    from: number;
    to: number;
    text: string;
  } | null> {
    return (await postRequest('editor.getSelection')) as never;
  },

  async replaceRange(args: { from: number; to: number; text: string }): Promise<void> {
    await postRequest('editor.replaceRange', args);
  },

  async insertAtCursor(text: string): Promise<void> {
    await postRequest('editor.insertAtCursor', { text });
  },

  async getActiveFilePath(): Promise<string | null> {
    return (await postRequest('project.getActiveFilePath')) as string | null;
  },

  async startAiStream(req: {
    url: string;
    headers: [string, string][];
    body: string;
    sse: boolean;
  }): Promise<string> {
    const result = (await postRequest('ai.fetchStream.start', req)) as { streamId: string };
    return result.streamId;
  },

  async cancelAiStream(streamId: string): Promise<void> {
    await postRequest('ai.fetchStream.cancel', { streamId });
  },
};

/**
 * Async iterator over a started AI stream. Yields each `{kind, ...}` event the
 * Rust bridge emits — the consumer is responsible for parsing the payload
 * (e.g. OpenAI delta JSON).
 */
export async function* aiStream(streamId: string): AsyncGenerator<
  { kind: 'chunk'; data: string } | { kind: 'done' } | { kind: 'error'; message: string; status?: number }
> {
  const eventName = `ai-stream://${streamId}`;
  const queue: unknown[] = [];
  let waiter: ((v: void) => void) | null = null;

  const handler = (payload: unknown) => {
    queue.push(payload);
    waiter?.();
    waiter = null;
  };

  let subs = eventSubscribers.get(eventName);
  if (!subs) {
    subs = new Set();
    eventSubscribers.set(eventName, subs);
  }
  subs.add(handler);

  try {
    while (true) {
      if (queue.length === 0) {
        await new Promise<void>((resolve) => {
          waiter = resolve;
        });
      }
      const ev = queue.shift() as
        | { kind: 'chunk'; data: string }
        | { kind: 'done' }
        | { kind: 'error'; message: string; status?: number };
      yield ev;
      if (ev.kind === 'done' || ev.kind === 'error') return;
    }
  } finally {
    subs.delete(handler);
    if (subs.size === 0) eventSubscribers.delete(eventName);
  }
}
