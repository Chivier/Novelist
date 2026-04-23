import type { Locale, TranslationMap } from './types';
import { en } from './locales/en';

const LOCALE_KEY = 'novelist-locale';

// English is always eagerly loaded — it's the fallback when a key is missing
// in the active locale, and the default for fresh installs. Other locales are
// loaded on demand via dynamic import so their JSON doesn't sit in the
// first-paint JS chunk for users who will never need them.
const translations: Partial<Record<Locale, TranslationMap>> = { en };

async function loadLocale(locale: Locale): Promise<TranslationMap> {
  const cached = translations[locale];
  if (cached) return cached;
  if (locale === 'zh-CN') {
    const mod = await import('./locales/zh-CN');
    translations['zh-CN'] = mod.zhCN;
    return mod.zhCN;
  }
  return en;
}

class I18nStore {
  locale = $state<Locale>('en');

  private get messages(): TranslationMap {
    return translations[this.locale] ?? en;
  }

  t(key: string, params?: Record<string, string | number>): string {
    let value = this.messages[key] ?? en[key] ?? key;

    if (typeof value === 'object') {
      const count = typeof params?.count === 'number' ? params.count : 0;
      if (count === 0 && value.zero) {
        value = value.zero;
      } else if (count === 1 && value.one) {
        value = value.one;
      } else {
        value = value.other;
      }
    }

    if (params) {
      for (const [k, v] of Object.entries(params)) {
        value = (value as string).replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
      }
    }

    return value as string;
  }

  get availableLocales(): { code: Locale; name: string; nativeName: string }[] {
    return [
      { code: 'en', name: 'English', nativeName: 'English' },
      { code: 'zh-CN', name: 'Chinese (Simplified)', nativeName: '简体中文' },
    ];
  }

  /**
   * Switch the active locale. The reactive `locale` field is updated
   * synchronously so the UI reflects the choice immediately; the returned
   * promise resolves once the translation chunk is loaded (awaited by tests
   * and callers that need the new strings available). Until the chunk lands
   * the `messages` getter falls back to English, which is always resident.
   */
  async setLocale(locale: Locale): Promise<void> {
    this.locale = locale;
    localStorage.setItem(LOCALE_KEY, locale);
    await loadLocale(locale);
  }

  /**
   * Detect the initial locale (sync) and async-load its translation chunk.
   * `main.ts` awaits this before mounting so the first paint has real strings.
   * The `locale` field is set synchronously so tests that call `init()` without
   * awaiting still observe the correct value.
   */
  async init(): Promise<void> {
    const saved = localStorage.getItem(LOCALE_KEY) as Locale | null;
    if (saved === 'en' || saved === 'zh-CN') {
      this.locale = saved;
    } else if (typeof navigator !== 'undefined' && navigator.language?.startsWith('zh')) {
      this.locale = 'zh-CN';
    }
    await loadLocale(this.locale);
  }
}

export const i18n = new I18nStore();
export const t = i18n.t.bind(i18n);
export type { Locale };
