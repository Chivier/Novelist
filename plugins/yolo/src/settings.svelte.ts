/**
 * Persistent settings for the YOLO plugin. localStorage keys live under the
 * iframe's own asset:// origin so they don't collide with the host app or
 * other plugins.
 */

const KEY = 'yolo:settings:v1';

export type YoloSettings = {
  baseUrl: string;
  apiKey: string;
  model: string;
  temperature: number;
  systemPrompt: string;
  includeCurrentFile: boolean;
  includeSelection: boolean;
};

const DEFAULTS: YoloSettings = {
  baseUrl: 'https://api.openai.com/v1',
  apiKey: '',
  model: 'gpt-4o-mini',
  temperature: 0.7,
  systemPrompt: 'You are a helpful writing assistant for novelists.',
  includeCurrentFile: false,
  includeSelection: true,
};

function load(): YoloSettings {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<YoloSettings>;
    return { ...DEFAULTS, ...parsed };
  } catch {
    return { ...DEFAULTS };
  }
}

export function createSettingsStore() {
  let value = $state<YoloSettings>(load());

  function persist() {
    try {
      localStorage.setItem(KEY, JSON.stringify(value));
    } catch {
      /* quota exceeded or storage disabled — silently ignore */
    }
  }

  return {
    get value() {
      return value;
    },
    update(patch: Partial<YoloSettings>) {
      value = { ...value, ...patch };
      persist();
    },
    reset() {
      value = { ...DEFAULTS };
      persist();
    },
  };
}
