import { describe, it, expect, afterEach } from 'vitest';

/**
 * [contract] templateI18n — New Project dialog label helpers. Verifies
 * that built-in templates resolve through i18n, user-authored templates
 * pass through their `template.toml` strings, and missing i18n keys fall
 * back to the Rust-provided English defaults.
 */

import { i18n } from '$lib/i18n';
import {
  categoryLabel,
  templateName,
  templateDescription,
} from '$lib/utils/templateI18n';
import type { TemplateInfo } from '$lib/ipc/commands';

const builtin = (id: string, overrides: Partial<TemplateInfo> = {}): TemplateInfo => ({
  id,
  name: `Fallback ${id}`,
  description: `Fallback description for ${id}`,
  category: 'fiction',
  builtin: true,
  ...overrides,
});

const userTpl: TemplateInfo = {
  id: 'my-custom',
  name: 'My Custom Template',
  description: 'A user-authored template from template.toml',
  category: 'custom',
  builtin: false,
};

const originalLocale = i18n.locale;

afterEach(() => {
  i18n.locale = originalLocale;
});

describe('[contract] categoryLabel', () => {
  it('returns the English label for a known category under EN', () => {
    i18n.locale = 'en';
    expect(categoryLabel('general')).toBe('General');
    expect(categoryLabel('fiction')).toBe('Fiction');
    expect(categoryLabel('non-fiction')).toBe('Non-fiction');
    expect(categoryLabel('personal')).toBe('Personal');
    expect(categoryLabel('custom')).toBe('Custom');
  });

  it('returns the Chinese label for a known category under zh-CN', () => {
    i18n.locale = 'zh-CN';
    expect(categoryLabel('general')).toBe('通用');
    expect(categoryLabel('fiction')).toBe('虚构');
    expect(categoryLabel('non-fiction')).toBe('非虚构');
    expect(categoryLabel('personal')).toBe('个人');
    expect(categoryLabel('custom')).toBe('自定义');
  });

  it('returns the raw category string when no i18n key matches', () => {
    i18n.locale = 'en';
    expect(categoryLabel('some-unknown-cat')).toBe('some-unknown-cat');
  });
});

describe('[contract] templateName', () => {
  it('looks up the i18n key for built-in templates under EN', () => {
    i18n.locale = 'en';
    expect(templateName(builtin('blank'))).toBe('Blank');
    expect(templateName(builtin('novel'))).toBe('Novel');
    expect(templateName(builtin('long-novel'))).toBe('Long Novel');
    expect(templateName(builtin('short-story'))).toBe('Short Story');
    expect(templateName(builtin('screenplay'))).toBe('Screenplay');
    expect(templateName(builtin('blog'))).toBe('Blog');
    expect(templateName(builtin('journal'))).toBe('Journal');
  });

  it('looks up the i18n key for built-in templates under zh-CN', () => {
    i18n.locale = 'zh-CN';
    expect(templateName(builtin('blank'))).toBe('空白');
    expect(templateName(builtin('novel'))).toBe('小说');
    expect(templateName(builtin('long-novel'))).toBe('长篇小说');
    expect(templateName(builtin('short-story'))).toBe('短篇小说');
    expect(templateName(builtin('screenplay'))).toBe('剧本');
    expect(templateName(builtin('blog'))).toBe('博客');
    expect(templateName(builtin('journal'))).toBe('日记');
  });

  it('falls back to the Rust-provided name when the i18n key is missing', () => {
    i18n.locale = 'en';
    const tpl = builtin('future-template-id', { name: 'Future Template' });
    expect(templateName(tpl)).toBe('Future Template');
  });

  it('uses the user template name verbatim without i18n lookup', () => {
    i18n.locale = 'zh-CN';
    // Even under zh-CN, a user-authored template's name comes straight
    // from its template.toml — no localization.
    expect(templateName(userTpl)).toBe('My Custom Template');
  });
});

describe('[contract] templateDescription', () => {
  it('uses the i18n key for built-in templates under EN', () => {
    i18n.locale = 'en';
    expect(templateDescription(builtin('blank'))).toBe(
      'Empty project with default settings',
    );
    expect(templateDescription(builtin('long-novel'))).toContain('Multi-volume novel');
  });

  it('uses the i18n key for built-in templates under zh-CN', () => {
    i18n.locale = 'zh-CN';
    expect(templateDescription(builtin('blank'))).toBe('使用默认设置的空项目');
    expect(templateDescription(builtin('long-novel'))).toContain('分卷长篇');
  });

  it('falls back to the Rust-provided description when the i18n key is missing', () => {
    i18n.locale = 'en';
    const tpl = builtin('future-template-id', {
      description: 'A future description',
    });
    expect(templateDescription(tpl)).toBe('A future description');
  });

  it('uses the user template description verbatim', () => {
    i18n.locale = 'zh-CN';
    expect(templateDescription(userTpl)).toBe(
      'A user-authored template from template.toml',
    );
  });
});
