import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("$lib/ipc/commands", () => ({
  commands: {
    isPortableMode: vi.fn(),
  },
}));

import { commands } from "$lib/ipc/commands";
import {
  getPortableInfo,
  portableInfoSync,
  __resetPortableCacheForTests,
} from "$lib/services/portable";

describe("portable service", () => {
  beforeEach(() => {
    __resetPortableCacheForTests();
    vi.clearAllMocks();
  });

  it("returns enabled + data_root from IPC", async () => {
    (commands.isPortableMode as ReturnType<typeof vi.fn>).mockResolvedValue({
      enabled: true,
      data_root: "C:\\Users\\x\\Novelist\\data",
    });
    const info = await getPortableInfo();
    expect(info.enabled).toBe(true);
    expect(info.dataRoot).toBe("C:\\Users\\x\\Novelist\\data");
  });

  it("caches the IPC result", async () => {
    (commands.isPortableMode as ReturnType<typeof vi.fn>).mockResolvedValue({
      enabled: false,
      data_root: "/home/x/.novelist",
    });
    await getPortableInfo();
    await getPortableInfo();
    expect(commands.isPortableMode).toHaveBeenCalledTimes(1);
  });

  it("portableInfoSync returns null before resolution", () => {
    expect(portableInfoSync()).toBeNull();
  });

  it("portableInfoSync returns cached value after resolution", async () => {
    (commands.isPortableMode as ReturnType<typeof vi.fn>).mockResolvedValue({
      enabled: true,
      data_root: "/foo",
    });
    await getPortableInfo();
    expect(portableInfoSync()).toEqual({ enabled: true, dataRoot: "/foo" });
  });

  it("clears inflight on IPC rejection so retries work", async () => {
    const fn = commands.isPortableMode as ReturnType<typeof vi.fn>;
    fn.mockRejectedValueOnce(new Error("backend not ready"));
    await expect(getPortableInfo()).rejects.toThrow("backend not ready");
    // Second call should retry (not return the cached rejected promise).
    fn.mockResolvedValueOnce({ enabled: false, data_root: "/x" });
    const info = await getPortableInfo();
    expect(info.dataRoot).toBe("/x");
  });
});
