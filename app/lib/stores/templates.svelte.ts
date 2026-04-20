/**
 * Snippet-template store — mirrors the Rust `template_files` commands into a
 * reactive `$state` array of summaries and exposes the high-level execute +
 * CRUD actions the panel / command palette call into.
 *
 * The store does NOT subscribe to `projectStore`; callers pass the current
 * project dir explicitly so the store stays easy to mock in tests.
 */

import { commands, type TemplateFileSummary, type TemplateMode, type TemplateSource } from '$lib/ipc/commands';

class TemplatesStore {
  summaries = $state<TemplateFileSummary[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  /** Monotonic tick bumped after any successful mutation so UIs can re-animate. */
  revision = $state(0);

  async refresh(projectDir: string | null): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const res = await commands.listTemplateFiles(projectDir);
      if (res.status === 'ok') {
        this.summaries = res.data;
      } else {
        this.error = String(res.error);
      }
    } catch (e: any) {
      this.error = e?.message ?? String(e);
    } finally {
      this.loading = false;
    }
  }

  bundled(): TemplateFileSummary[] {
    return this.summaries.filter(s => s.source === 'bundled');
  }
  project(): TemplateFileSummary[] {
    return this.summaries.filter(s => s.source === 'project');
  }

  async read(source: TemplateSource, id: string, projectDir: string | null) {
    const res = await commands.readTemplateFile(source, id, projectDir);
    if (res.status === 'ok') return res.data;
    throw new Error(String(res.error));
  }

  async create(
    projectDir: string,
    id: string,
    input: { name: string; mode: TemplateMode; description: string | null; defaultFilename: string | null },
    body: string
  ): Promise<TemplateFileSummary> {
    const res = await commands.writeTemplateFile(projectDir, id, input, body);
    if (res.status !== 'ok') throw new Error(String(res.error));
    this.revision++;
    await this.refresh(projectDir);
    return res.data;
  }

  async rename(projectDir: string, oldId: string, newId: string): Promise<TemplateFileSummary> {
    const res = await commands.renameTemplateFile(projectDir, oldId, newId);
    if (res.status !== 'ok') throw new Error(String(res.error));
    this.revision++;
    await this.refresh(projectDir);
    return res.data;
  }

  async remove(projectDir: string, id: string): Promise<void> {
    const res = await commands.deleteTemplateFile(projectDir, id);
    if (res.status !== 'ok') throw new Error(String(res.error));
    this.revision++;
    await this.refresh(projectDir);
  }

  async duplicateBundled(projectDir: string, bundledId: string, newId: string | null): Promise<TemplateFileSummary> {
    const res = await commands.duplicateBundledTemplate(projectDir, bundledId, newId);
    if (res.status !== 'ok') throw new Error(String(res.error));
    this.revision++;
    await this.refresh(projectDir);
    return res.data;
  }
}

export const templatesStore = new TemplatesStore();
