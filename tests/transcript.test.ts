import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import TranscriptPane from '../src/components/transcript/TranscriptPane.svelte';
import EmptyState from '../src/components/transcript/EmptyState.svelte';
import NoApiKey from '../src/components/transcript/NoApiKey.svelte';

describe('TranscriptPane', () => {
  it('renders both columns with header labels', () => {
    const { getByText } = render(TranscriptPane, {
      props: {
        mode: 'conversation',
        a: { code: 'EN', name: 'English' },
        b: { code: 'JA', name: 'Japanese' },
        mine: 'a',
        original: [{ ts_ms: 100, status: 'original', text: 'Hello', language: 'en', chip: 'A', live: false }],
        translation: [{ ts_ms: 100, status: 'translation', text: 'こんにちは', language: 'ja', chip: 'A', live: false }],
        liveOriginal: '', liveTranslation: '',
      },
    });
    expect(getByText('Original')).toBeTruthy();
    expect(getByText('Translation')).toBeTruthy();
    expect(getByText('Hello')).toBeTruthy();
    expect(getByText('こんにちは')).toBeTruthy();
  });
});

describe('EmptyState', () => {
  it('shows mode-specific copy', () => {
    const { getByText } = render(EmptyState, { props: { mode: 'meeting' } });
    expect(getByText(/Ready to translate a meeting/)).toBeTruthy();
  });
});

describe('NoApiKey', () => {
  it('lists the Add API key CTA', () => {
    const { getByText } = render(NoApiKey, { props: { onaddkey: () => {} } });
    expect(getByText('Add API key')).toBeTruthy();
  });
});
