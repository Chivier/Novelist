import { describe, it, expect, vi, beforeEach } from 'vitest';

/**
 * [contract] services/publish — orchestrator for publishing.
 *
 * Rules under test:
 *   1. stripFrontMatter removes leading `---\n...\n---\n`.
 *   2. extractLocalImageRefs returns local image paths only (skips http/data).
 *   3. rewriteBodyWithUrlMap replaces only the URL portion of `![](...)`
 *      keeping alt text and optional title intact.
 *   4. dispatchPublish routes platform-specific calls correctly:
 *      Medium → no Pandoc conversion; others → Pandoc-converted HTML.
 *   5. Pre-publish image upload uploads each local ref via the
 *      platform's media endpoint and rewrites the body URLs.
 *   6. Cover image upload sets feature_image_url; WP also sets
 *      featured_media_id when attachment_id > 0.
 */

const { calls, mockSettings, mockBytes } = vi.hoisted(() => {
  const calls: Array<{ name: string; args: unknown[] }> = [];
  const mockSettings = {
    channels: [] as Array<{ id: string; name: string; platform: string; [k: string]: unknown }>,
  };
  const mockBytes = new Map<string, number[]>();
  return { calls, mockSettings, mockBytes };
});

vi.mock('$lib/ipc/commands', () => {
  const ok = <T>(data: T) => Promise.resolve({ status: 'ok', data });
  const err = (e: string) => Promise.resolve({ status: 'error', error: e });

  function makeUpload(platform: string) {
    return (bytes: number[], filename: string, _mime: string, _config: { platform: string }) => {
      calls.push({ name: `uploadPostImage_${platform}`, args: [bytes.length, filename] });
      return ok({
        url: `https://${platform}.example.com/${filename}`,
        attachment_id: platform === 'wordpress_self_hosted' ? 99 : 0,
      });
    };
  }
  function makePublish(platform: string) {
    return (input: unknown, _config: { platform: string }) => {
      calls.push({ name: `publish_${platform}`, args: [input] });
      return ok({ url: `https://${platform}.example.com/post`, remote_id: '1' });
    };
  }

  return {
    commands: {
      getPublishSettings: () => {
        calls.push({ name: 'getPublishSettings', args: [] });
        return ok(mockSettings);
      },
      readImageBytes: (path: string) => {
        const bytes = mockBytes.get(path);
        if (!bytes) return err(`not found: ${path}`);
        return ok(bytes);
      },
      uploadPostImageGhost: makeUpload('ghost'),
      uploadPostImageWordpressSelfHosted: makeUpload('wordpress_self_hosted'),
      uploadPostImageWordpressCom: makeUpload('wordpress_com'),
      uploadPostImageMedium: makeUpload('medium'),
      publishToGhost: makePublish('ghost'),
      publishToWordpressSelfHosted: makePublish('wordpress_self_hosted'),
      publishToWordpressCom: makePublish('wordpress_com'),
      publishToMedium: makePublish('medium'),
      convertMarkdownToHtml: (markdown: string) => {
        calls.push({ name: 'convertMarkdownToHtml', args: [markdown] });
        return ok(`<html>${markdown}</html>`);
      },
    },
  };
});

beforeEach(() => {
  calls.length = 0;
  mockSettings.channels = [];
  mockBytes.clear();
});

describe('stripFrontMatter', () => {
  it('removes leading YAML front-matter block', async () => {
    const { stripFrontMatter } = await import('$lib/services/publish');
    const out = stripFrontMatter('---\ntitle: x\ntags: [a, b]\n---\n# Heading\n\nBody.');
    expect(out).toBe('# Heading\n\nBody.');
  });
  it('passes through unchanged when no front-matter', async () => {
    const { stripFrontMatter } = await import('$lib/services/publish');
    expect(stripFrontMatter('# Heading\n')).toBe('# Heading\n');
  });
  it('handles CRLF line endings', async () => {
    const { stripFrontMatter } = await import('$lib/services/publish');
    expect(stripFrontMatter('---\r\ntitle: x\r\n---\r\nbody')).toBe('body');
  });
});

describe('extractLocalImageRefs', () => {
  it('returns only local references', async () => {
    const { extractLocalImageRefs } = await import('$lib/services/publish');
    const refs = extractLocalImageRefs(
      '![local](./a.png) ![remote](https://x/b.png) ![root](/c.png) ![alt with space](d.gif "title")',
    );
    expect(refs).toEqual(['./a.png', '/c.png', 'd.gif']);
  });
});

describe('rewriteBodyWithUrlMap', () => {
  it('replaces matching URLs and leaves alt+title intact', async () => {
    const { rewriteBodyWithUrlMap } = await import('$lib/services/publish');
    const map = new Map([['./a.png', 'https://cdn.example.com/x.png']]);
    expect(
      rewriteBodyWithUrlMap('![alt text](./a.png "t") prose ![other](b.png)', map),
    ).toBe('![alt text](https://cdn.example.com/x.png "t") prose ![other](b.png)');
  });
});

describe('dispatchPublish', () => {
  it('routes Medium without Pandoc conversion', async () => {
    mockSettings.channels = [
      { id: 'm', name: 'Medium', platform: 'medium', token: 't' },
    ];
    const { dispatchPublish } = await import('$lib/services/publish');
    await dispatchPublish(
      mockSettings.channels[0] as never,
      {
        title: 'Hi',
        tags: [],
        status: 'public',
      },
      { dir: '/p', text: 'plain' },
    );
    expect(calls.find(c => c.name === 'convertMarkdownToHtml')).toBeUndefined();
    const pub = calls.find(c => c.name === 'publish_medium');
    expect(pub).toBeDefined();
    expect((pub!.args[0] as { body_format: string }).body_format).toBe('markdown');
  });

  it('routes Ghost via Pandoc → HTML', async () => {
    mockSettings.channels = [
      { id: 'g', name: 'Ghost', platform: 'ghost', admin_url: 'x', api_key: 'a:b' },
    ];
    const { dispatchPublish } = await import('$lib/services/publish');
    await dispatchPublish(
      mockSettings.channels[0] as never,
      { title: 'Hi', tags: ['rust'], status: 'draft' },
      { dir: '/p', text: 'hello' },
    );
    expect(calls.find(c => c.name === 'convertMarkdownToHtml')).toBeDefined();
    const pub = calls.find(c => c.name === 'publish_ghost');
    expect(pub).toBeDefined();
    expect((pub!.args[0] as { body_format: string }).body_format).toBe('html');
    expect((pub!.args[0] as { body: string }).body).toBe('<html>hello</html>');
  });

  it('uploads local images and rewrites the body before submission', async () => {
    mockSettings.channels = [
      { id: 'g', name: 'Ghost', platform: 'ghost', admin_url: 'x', api_key: 'a:b' },
    ];
    mockBytes.set('/p/a.png', [1, 2, 3]);
    const { dispatchPublish } = await import('$lib/services/publish');
    await dispatchPublish(
      mockSettings.channels[0] as never,
      { title: 'Hi', tags: [], status: 'draft' },
      { dir: '/p', text: '![](./a.png)' },
    );
    expect(calls.find(c => c.name === 'uploadPostImage_ghost')).toBeDefined();
    const conv = calls.find(c => c.name === 'convertMarkdownToHtml');
    expect((conv!.args[0] as string)).toContain('https://ghost.example.com/a.png');
  });

  it('cover image upload sets feature_image_url and (WP) featured_media_id', async () => {
    mockSettings.channels = [
      {
        id: 'wp',
        name: 'WP',
        platform: 'wordpress_self_hosted',
        site_url: 'x',
        username: 'u',
        app_password: 'p',
      },
    ];
    const { dispatchPublish } = await import('$lib/services/publish');
    await dispatchPublish(
      mockSettings.channels[0] as never,
      {
        title: 'Hi',
        tags: [],
        status: 'draft',
        coverImage: {
          bytes: new Uint8Array([1]),
          filename: 'cover.png',
          mime: 'image/png',
        },
      },
      { dir: '/p', text: 'body' },
    );
    const pub = calls.find(c => c.name === 'publish_wordpress_self_hosted');
    expect(pub).toBeDefined();
    const inp = pub!.args[0] as { feature_image_url?: string; featured_media_id?: number };
    expect(inp.feature_image_url).toBeTruthy();
    expect(inp.featured_media_id).toBe(99);
  });

  it('skips remote URLs in the body when uploading images', async () => {
    mockSettings.channels = [
      { id: 'g', name: 'Ghost', platform: 'ghost', admin_url: 'x', api_key: 'a:b' },
    ];
    const { dispatchPublish } = await import('$lib/services/publish');
    await dispatchPublish(
      mockSettings.channels[0] as never,
      { title: 'Hi', tags: [], status: 'draft' },
      { dir: '/p', text: '![remote](https://cdn.example.com/x.png)' },
    );
    expect(calls.filter(c => c.name === 'uploadPostImage_ghost')).toHaveLength(0);
  });

  it('strips front-matter before submitting', async () => {
    mockSettings.channels = [
      { id: 'm', name: 'Medium', platform: 'medium', token: 't' },
    ];
    const { dispatchPublish } = await import('$lib/services/publish');
    await dispatchPublish(
      mockSettings.channels[0] as never,
      { title: 'Hi', tags: [], status: 'public' },
      { dir: '/p', text: '---\nx: y\n---\nhello body' },
    );
    const pub = calls.find(c => c.name === 'publish_medium');
    expect((pub!.args[0] as { body: string }).body).toBe('hello body');
  });

  it('image read failure aborts publish with descriptive error', async () => {
    mockSettings.channels = [
      { id: 'g', name: 'Ghost', platform: 'ghost', admin_url: 'x', api_key: 'a:b' },
    ];
    // mockBytes.get('/p/missing.png') will return undefined → err
    const { dispatchPublish } = await import('$lib/services/publish');
    await expect(
      dispatchPublish(
        mockSettings.channels[0] as never,
        { title: 'Hi', tags: [], status: 'draft' },
        { dir: '/p', text: '![](./missing.png)' },
      ),
    ).rejects.toThrow(/missing\.png/);
  });
});
