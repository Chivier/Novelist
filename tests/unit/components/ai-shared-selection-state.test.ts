import { describe, expect, it } from 'vitest';
import {
  reduceSelectionSuggestion,
  type SelectionSuggestionState,
} from '$lib/components/ai-shared/selection-state';

const initial: SelectionSuggestionState = {
  snapshotKey: null,
  status: 'none',
};

describe('[contract] AI selection suggestion state', () => {
  it('shows a new selection as suggested rather than attached', () => {
    expect(reduceSelectionSuggestion(initial, { type: 'selection-changed', key: 'a:0:5' })).toEqual({
      snapshotKey: 'a:0:5',
      status: 'suggested',
    });
  });

  it('attaches the current suggestion explicitly', () => {
    const suggested = reduceSelectionSuggestion(initial, { type: 'selection-changed', key: 'a:0:5' });
    expect(reduceSelectionSuggestion(suggested, { type: 'attach' })).toEqual({
      snapshotKey: 'a:0:5',
      status: 'attached',
    });
  });

  it('dismisses a suggestion until the selection changes', () => {
    const dismissed = reduceSelectionSuggestion({ snapshotKey: 'a:0:5', status: 'suggested' }, { type: 'dismiss' });
    expect(dismissed).toEqual({ snapshotKey: 'a:0:5', status: 'dismissed' });
    expect(reduceSelectionSuggestion(dismissed, { type: 'selection-changed', key: 'a:0:5' })).toBe(dismissed);
    expect(reduceSelectionSuggestion(dismissed, { type: 'selection-changed', key: 'a:2:7' })).toEqual({
      snapshotKey: 'a:2:7',
      status: 'suggested',
    });
  });
});
