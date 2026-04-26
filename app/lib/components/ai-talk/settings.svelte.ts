/**
 * Persistent settings for the AI Talk panel. Stored in the host's localStorage
 * under a namespaced key so it doesn't collide with editor state.
 */

const KEY = 'novelist:ai-talk:settings:v1';

export type AiTalkSettings = {
  activeProfileId: string;
  profiles: AiTalkProviderProfile[];
  baseUrl: string;
  apiKey: string;
  model: string;
  temperature: number;
  systemPrompt: string;
  includeCurrentFile: boolean;
  includeSelection: boolean;
};

export type AiTalkProviderProfile = {
  id: string;
  label: string;
  baseUrl: string;
  apiKey: string;
  model: string;
  temperature: number;
  custom?: boolean;
};

const DEFAULT_PROFILES: AiTalkProviderProfile[] = [
  { id: 'openai', label: 'OpenAI', baseUrl: 'https://api.openai.com/v1', apiKey: '', model: 'gpt-4o-mini', temperature: 0.7 },
  { id: 'anthropic', label: 'Anthropic-compatible', baseUrl: 'https://api.anthropic.com/v1', apiKey: '', model: 'claude-sonnet-4-5', temperature: 0.7 },
  { id: 'deepseek', label: 'DeepSeek', baseUrl: 'https://api.deepseek.com/v1', apiKey: '', model: 'deepseek-chat', temperature: 0.7 },
  { id: 'openrouter', label: 'OpenRouter', baseUrl: 'https://openrouter.ai/api/v1', apiKey: '', model: 'openai/gpt-4o-mini', temperature: 0.7 },
  { id: 'groq', label: 'Groq', baseUrl: 'https://api.groq.com/openai/v1', apiKey: '', model: 'llama-3.3-70b-versatile', temperature: 0.7 },
  { id: 'ollama', label: 'Ollama (local)', baseUrl: 'http://localhost:11434/v1', apiKey: '', model: 'llama3.2', temperature: 0.7 },
  { id: 'custom', label: 'Custom', baseUrl: 'https://api.openai.com/v1', apiKey: '', model: 'gpt-4o-mini', temperature: 0.7, custom: true },
];

const DEFAULTS: AiTalkSettings = {
  activeProfileId: 'openai',
  profiles: DEFAULT_PROFILES,
  baseUrl: 'https://api.openai.com/v1',
  apiKey: '',
  model: 'gpt-4o-mini',
  temperature: 0.7,
  systemPrompt: 'You are a helpful writing assistant for novelists.',
  includeCurrentFile: false,
  includeSelection: true,
};

function mergeProfiles(parsed: Partial<AiTalkSettings>): AiTalkProviderProfile[] {
  const saved = Array.isArray(parsed.profiles) ? parsed.profiles : [];
  const byId = new Map(saved.map((p) => [p.id, p]));
  const activeProfileId = parsed.activeProfileId ?? DEFAULTS.activeProfileId;
  return DEFAULT_PROFILES.map((profile) => {
    const merged = { ...profile, ...(byId.get(profile.id) ?? {}) };
    if (!Array.isArray(parsed.profiles) && profile.id === activeProfileId) {
      return {
        ...merged,
        baseUrl: parsed.baseUrl ?? merged.baseUrl,
        apiKey: parsed.apiKey ?? merged.apiKey,
        model: parsed.model ?? merged.model,
        temperature: parsed.temperature ?? merged.temperature,
      };
    }
    return merged;
  });
}

function hydrateActiveProfile(value: AiTalkSettings): AiTalkSettings {
  const active = value.profiles.find((p) => p.id === value.activeProfileId) ?? value.profiles[0];
  if (!active) return value;
  return {
    ...value,
    activeProfileId: active.id,
    baseUrl: active.baseUrl,
    apiKey: active.apiKey,
    model: active.model,
    temperature: active.temperature,
  };
}

function load(): AiTalkSettings {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<AiTalkSettings>;
    return hydrateActiveProfile({ ...DEFAULTS, ...parsed, profiles: mergeProfiles(parsed) });
  } catch {
    return { ...DEFAULTS };
  }
}

function persist(value: AiTalkSettings) {
  try {
    localStorage.setItem(KEY, JSON.stringify(value));
  } catch {
    /* quota exceeded or storage disabled — silently ignore */
  }
}

class AiTalkSettingsStore {
  value = $state<AiTalkSettings>(load());

  update(patch: Partial<AiTalkSettings>) {
    const next = { ...this.value, ...patch };
    const activeProfileId = patch.activeProfileId ?? next.activeProfileId;
    let profiles = next.profiles;
    const active = profiles.find((p) => p.id === activeProfileId);
    if (
      active &&
      (patch.baseUrl !== undefined ||
        patch.apiKey !== undefined ||
        patch.model !== undefined ||
        patch.temperature !== undefined)
    ) {
      profiles = profiles.map((p) =>
        p.id === active.id
          ? {
              ...p,
              baseUrl: patch.baseUrl ?? p.baseUrl,
              apiKey: patch.apiKey ?? p.apiKey,
              model: patch.model ?? p.model,
              temperature: patch.temperature ?? p.temperature,
            }
          : p,
      );
    }
    this.value = hydrateActiveProfile({ ...next, activeProfileId, profiles });
    persist(this.value);
  }

  reset() {
    this.value = { ...DEFAULTS };
    persist(this.value);
  }
}

export const aiTalkSettings = new AiTalkSettingsStore();
