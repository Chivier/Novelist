export type SelectionSuggestionStatus = 'none' | 'suggested' | 'attached' | 'dismissed';

export type SelectionSuggestionState = {
  snapshotKey: string | null;
  status: SelectionSuggestionStatus;
};

export type SelectionSuggestionEvent =
  | { type: 'selection-changed'; key: string | null }
  | { type: 'attach' }
  | { type: 'dismiss' }
  | { type: 'clear' };

export function reduceSelectionSuggestion(
  state: SelectionSuggestionState,
  event: SelectionSuggestionEvent,
): SelectionSuggestionState {
  if (event.type === 'clear') return { snapshotKey: null, status: 'none' };
  if (event.type === 'attach') {
    return state.snapshotKey ? { ...state, status: 'attached' } : state;
  }
  if (event.type === 'dismiss') {
    return state.snapshotKey ? { ...state, status: 'dismissed' } : state;
  }
  if (!event.key) return { snapshotKey: null, status: 'none' };
  if (event.key === state.snapshotKey) return state;
  return { snapshotKey: event.key, status: 'suggested' };
}
