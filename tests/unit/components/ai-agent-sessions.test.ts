import { describe, expect, it, beforeEach, vi } from 'vitest';

const { killClaudeSession } = vi.hoisted(() => ({
  killClaudeSession: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/components/ai-agent/host', () => ({
  killClaudeSession,
}));

import { aiAgentSessions } from '$lib/components/ai-agent/sessions.svelte';

function resetStore() {
  localStorage.clear();
  aiAgentSessions.sessions = [];
  aiAgentSessions.activeId = null;
  killClaudeSession.mockClear();
}

describe('[contract] aiAgentSessions store', () => {
  beforeEach(() => {
    resetStore();
  });

  it('creates Claude act sessions by default', () => {
    const id = aiAgentSessions.create();
    expect(aiAgentSessions.activeId).toBe(id);
    expect(aiAgentSessions.active?.providerId).toBe('claude');
    expect(aiAgentSessions.active?.mode).toBe('act');
    expect(aiAgentSessions.active?.turns).toEqual([]);
  });

  it('switches agent mode without changing the session uuid', () => {
    const id = aiAgentSessions.create();
    const uuid = aiAgentSessions.active?.sessionUuid;
    aiAgentSessions.setMode(id, 'plan');
    expect(aiAgentSessions.active?.mode).toBe('plan');
    expect(aiAgentSessions.active?.sessionUuid).toBe(uuid);
  });

  it('forks transcript into a new session with provider state', () => {
    const id = aiAgentSessions.create();
    aiAgentSessions.updateTurns(id, [
      { role: 'user', text: 'one' },
      { role: 'assistant', text: 'two', cards: [] },
      { role: 'user', text: 'three' },
    ]);
    const forkId = aiAgentSessions.fork(id, 1);
    expect(forkId).toBeTruthy();
    expect(aiAgentSessions.activeId).toBe(forkId);
    expect(aiAgentSessions.active?.turns).toHaveLength(2);
    expect(aiAgentSessions.active?.providerState).toEqual({ forkedFrom: id });
    expect(aiAgentSessions.active?.sessionUuid).not.toBe(
      aiAgentSessions.sessions.find((s) => s.id === id)?.sessionUuid,
    );
  });

  it('compacts the active transcript into a single assistant summary', () => {
    const id = aiAgentSessions.create();
    aiAgentSessions.updateTurns(id, [
      { role: 'user', text: 'question' },
      { role: 'assistant', text: 'answer', cards: [] },
    ]);
    aiAgentSessions.compactActive('summary');
    expect(aiAgentSessions.active?.turns).toEqual([
      { role: 'assistant', text: 'summary', cards: [] },
    ]);
  });

  it('clearTurns resets transcript and rotates Claude session uuid', () => {
    const id = aiAgentSessions.create();
    const uuid = aiAgentSessions.active?.sessionUuid;
    aiAgentSessions.updateTurns(id, [{ role: 'user', text: 'hello' }]);
    aiAgentSessions.clearTurns(id);
    expect(aiAgentSessions.active?.turns).toEqual([]);
    expect(aiAgentSessions.active?.sessionUuid).not.toBe(uuid);
    expect(killClaudeSession).toHaveBeenCalledWith(uuid);
  });
});
