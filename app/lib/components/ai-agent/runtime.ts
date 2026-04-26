import {
  detectClaudeCli,
  killClaudeSession,
  listenClaudeStream,
  parseClaudeLine,
  sendToClaude,
  spawnClaudeSession,
  userInputLine,
  type ClaudeStreamEvent,
  type DetectedCli,
  type ParsedStreamEvent,
  type SpawnArgs,
} from './host';
import type { UnlistenFn } from '@tauri-apps/api/event';

export type AgentProviderId = 'claude' | 'codex';
export type AgentMode = 'act' | 'plan';

export type AgentRuntime = {
  providerId: AgentProviderId;
  detect(): Promise<DetectedCli | null>;
  spawn(args: SpawnArgs): Promise<string>;
  send(sessionId: string, text: string): Promise<void>;
  cancel(sessionId: string): Promise<void>;
  kill(sessionId: string): Promise<void>;
  resume(args: SpawnArgs): Promise<string>;
  listen(sessionId: string, handler: (event: ClaudeStreamEvent) => void): Promise<UnlistenFn>;
  parseEvent(line: string): ParsedStreamEvent | null;
};

export const ClaudeRuntime: AgentRuntime = {
  providerId: 'claude',
  detect: detectClaudeCli,
  spawn: spawnClaudeSession,
  send: (sessionId, text) => sendToClaude(sessionId, userInputLine(text)),
  cancel: killClaudeSession,
  kill: killClaudeSession,
  resume: spawnClaudeSession,
  listen: listenClaudeStream,
  parseEvent: parseClaudeLine,
};
