/**
 * Markmap tree helpers used by `MindmapOverlay`. Kept framework-free so they
 * can be unit-tested against raw `markmap-lib` output.
 */

export interface MindmapNode {
  depth?: number;
  children?: MindmapNode[];
  payload?: { fold?: number } & Record<string, unknown>;
}

/**
 * Set each node's `payload.fold` so the tree renders expanded up to `level`.
 *
 * - `level === null` → fully expanded (all `fold = 0`).
 * - `level === N`    → nodes at depth ≥ N with children are folded (`fold = 1`);
 *                      shallower nodes stay expanded (`fold = 0`).
 *
 * Mutates the input tree in place and returns it for chaining.
 */
export function applyFoldLevel(root: MindmapNode, level: number | null): MindmapNode {
  function walk(node: MindmapNode) {
    if (!node) return;
    const depth = node.depth ?? 0;
    if (level === null) {
      if (node.payload) node.payload.fold = 0;
    } else if (depth >= level && node.children && node.children.length > 0) {
      node.payload = { ...(node.payload ?? {}), fold: 1 };
    } else if (node.payload) {
      node.payload.fold = 0;
    }
    (node.children ?? []).forEach(walk);
  }
  walk(root);
  return root;
}
