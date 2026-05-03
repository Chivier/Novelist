/**
 * Reactive updater state for the in-app update flow.
 *
 * The lifecycle:
 *   idle → available → downloading → installing → ready → idle
 *                  ↘ error  ↗ error             ↘ idle (user defers restart)
 *
 * The progress modal mounts when `phase` is downloading / installing / ready
 * / error; the avilable-banner mounts when `phase === 'available'`.
 */

export type UpdatePhase =
  | 'idle'
  | 'available'
  | 'downloading'
  | 'installing'
  | 'ready'
  | 'error';

class UpdaterState {
  phase = $state<UpdatePhase>('idle');
  /** Pending version string, e.g. "0.2.3". */
  version = $state<string | null>(null);
  /** Release notes body — already plain text from the updater payload. */
  notes = $state<string | null>(null);
  /** Bytes downloaded so far (downloading phase). */
  downloaded = $state(0);
  /** Total bytes to download (downloading phase). 0 = unknown. */
  total = $state(0);
  /** Last error message — populated only when `phase === 'error'`. */
  error = $state<string | null>(null);

  /** 0–100 integer; 0 when total is unknown. */
  get percent(): number {
    if (this.total <= 0) return 0;
    return Math.min(100, Math.round((this.downloaded / this.total) * 100));
  }

  setAvailable(version: string, notes: string | null) {
    this.phase = 'available';
    this.version = version;
    this.notes = notes;
    this.downloaded = 0;
    this.total = 0;
    this.error = null;
  }

  startDownload(total: number) {
    this.phase = 'downloading';
    this.downloaded = 0;
    this.total = total;
    this.error = null;
  }

  recordChunk(chunkLength: number) {
    this.downloaded += chunkLength;
  }

  setInstalling() {
    this.phase = 'installing';
  }

  setReady() {
    this.phase = 'ready';
  }

  setError(message: string) {
    this.phase = 'error';
    this.error = message;
  }

  reset() {
    this.phase = 'idle';
    this.version = null;
    this.notes = null;
    this.downloaded = 0;
    this.total = 0;
    this.error = null;
  }
}

export const updaterState = new UpdaterState();
