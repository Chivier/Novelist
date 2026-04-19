import { describe, it, expect } from 'vitest';
import { applyFoldLevel, type MindmapNode } from '../../../app/lib/utils/mindmap';

function makeTree(): MindmapNode {
  // depth 0: root
  //   depth 1: A  (has children)
  //     depth 2: A1  (has children)
  //       depth 3: A1a
  //     depth 2: A2  (leaf)
  //   depth 1: B  (leaf)
  return {
    depth: 0,
    payload: {},
    children: [
      {
        depth: 1,
        payload: {},
        children: [
          {
            depth: 2,
            payload: {},
            children: [{ depth: 3, payload: {} }],
          },
          { depth: 2, payload: {} },
        ],
      },
      { depth: 1, payload: {} },
    ],
  };
}

describe('applyFoldLevel', () => {
  it('fully expands every node when level is null', () => {
    const root = makeTree();
    applyFoldLevel(root, null);
    expect(root.payload?.fold).toBe(0);
    expect(root.children?.[0].payload?.fold).toBe(0);
    expect(root.children?.[0].children?.[0].payload?.fold).toBe(0);
    expect(root.children?.[0].children?.[0].children?.[0].payload?.fold).toBe(0);
    expect(root.children?.[1].payload?.fold).toBe(0);
  });

  it('folds every node at or deeper than the given level when it has children', () => {
    const root = makeTree();
    applyFoldLevel(root, 1);
    // root (depth 0) < 1 → expanded
    expect(root.payload?.fold).toBe(0);
    // A (depth 1) has children → folded
    expect(root.children?.[0].payload?.fold).toBe(1);
    // B (depth 1) leaf → no fold applied (stays 0 because payload existed)
    expect(root.children?.[1].payload?.fold).toBe(0);
  });

  it('leaves deeper nodes expanded but folds the first node above them', () => {
    const root = makeTree();
    applyFoldLevel(root, 2);
    // depth 0,1 → expanded
    expect(root.payload?.fold).toBe(0);
    expect(root.children?.[0].payload?.fold).toBe(0);
    // A1 (depth 2) has a child → folded
    expect(root.children?.[0].children?.[0].payload?.fold).toBe(1);
    // A2 (depth 2) leaf → stays expanded
    expect(root.children?.[0].children?.[1].payload?.fold).toBe(0);
  });

  it('never folds leaf nodes regardless of their depth', () => {
    const root: MindmapNode = {
      depth: 0,
      payload: {},
      children: [
        { depth: 1, payload: {} },
        { depth: 1, payload: {} },
      ],
    };
    applyFoldLevel(root, 1);
    expect(root.children?.[0].payload?.fold).toBe(0);
    expect(root.children?.[1].payload?.fold).toBe(0);
  });

  it('creates a payload object on nodes that lack one before folding', () => {
    const root: MindmapNode = {
      depth: 0,
      children: [{ depth: 1, children: [{ depth: 2 }] }],
    };
    applyFoldLevel(root, 1);
    expect(root.children?.[0].payload).toBeDefined();
    expect(root.children?.[0].payload?.fold).toBe(1);
  });

  it('returns the root for chaining', () => {
    const root = makeTree();
    const returned = applyFoldLevel(root, 2);
    expect(returned).toBe(root);
  });
});
