import { commands } from "$lib/ipc/commands";

export type PortableInfo = { enabled: boolean; dataRoot: string };

let cached: PortableInfo | null = null;
let inflight: Promise<PortableInfo> | null = null;

export async function getPortableInfo(): Promise<PortableInfo> {
  if (cached) return cached;
  if (inflight) return inflight;
  inflight = (async () => {
    const info = await commands.isPortableMode();
    cached = { enabled: info.enabled, dataRoot: info.data_root };
    return cached;
  })();
  return inflight;
}

/** Synchronous accessor — returns null if `getPortableInfo()` hasn't resolved yet. */
export function portableInfoSync(): PortableInfo | null {
  return cached;
}

/** Test-only: reset the cache. */
export function __resetPortableCacheForTests() {
  cached = null;
  inflight = null;
}
