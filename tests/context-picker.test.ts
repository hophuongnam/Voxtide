import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ContextPicker from '../src/components/toolbar/ContextPicker.svelte';
import type { ContextPreset } from '../src/types';

const standup: ContextPreset = { id: 'p1', name: 'Standup', text: 'Speakers: Nam, Yuki.' };
const acme: ContextPreset = { id: 'p2', name: 'Client Acme', text: 'Acme Corp. Topic: Q3 renewal.' };
const blank: ContextPreset = { id: 'p3', name: '', text: 'no name set' };

// The trigger is always the first <button> in the tree — the dropdown panel
// (if any) only ever adds MORE buttons after it, so this is unambiguous
// whether the panel is open or closed.
function trigger(container: HTMLElement): HTMLButtonElement {
  return container.querySelector('button') as HTMLButtonElement;
}

describe('ContextPicker', () => {
  it('opens to show "No context", one button per preset, and "Edit contexts…"', async () => {
    const { container, getAllByRole } = render(ContextPicker, {
      props: { contexts: [standup, acme], activeId: null, disabled: false, onpick: vi.fn(), onedit: vi.fn() },
    });
    await fireEvent.click(trigger(container));

    const buttons = getAllByRole('button');
    // trigger + "No context" + 2 presets + "Edit contexts…"
    expect(buttons).toHaveLength(5);
    expect(buttons[1]!.textContent?.trim()).toBe('No context');
    expect(buttons[2]!.textContent?.trim()).toBe('Standup');
    expect(buttons[3]!.textContent?.trim()).toBe('Client Acme');
    expect(buttons[4]!.textContent?.trim()).toBe('Edit contexts…');
  });

  it('shows only "No context" + "Edit contexts…" when there are no presets', async () => {
    const { container, getAllByRole } = render(ContextPicker, {
      props: { contexts: [], activeId: null, disabled: false, onpick: vi.fn(), onedit: vi.fn() },
    });
    await fireEvent.click(trigger(container));
    const buttons = getAllByRole('button');
    expect(buttons).toHaveLength(3);
    expect(buttons[1]!.textContent?.trim()).toBe('No context');
    expect(buttons[2]!.textContent?.trim()).toBe('Edit contexts…');
  });

  it('trigger label reflects the active preset', () => {
    const { container } = render(ContextPicker, {
      props: { contexts: [standup, acme], activeId: 'p2', disabled: false, onpick: vi.fn(), onedit: vi.fn() },
    });
    expect(trigger(container).textContent?.trim()).toContain('Client Acme');
  });

  it('shows "No context" when activeId is null', () => {
    const { container } = render(ContextPicker, {
      props: { contexts: [standup], activeId: null, disabled: false, onpick: vi.fn(), onedit: vi.fn() },
    });
    expect(trigger(container).textContent?.trim()).toContain('No context');
  });

  it('shows "No context" when activeId names a preset absent from contexts (dangling / deleted-active)', () => {
    const { container } = render(ContextPicker, {
      props: { contexts: [standup], activeId: 'deleted-id', disabled: false, onpick: vi.fn(), onedit: vi.fn() },
    });
    expect(trigger(container).textContent?.trim()).toContain('No context');
  });

  it('an empty-name preset renders as "Untitled" on both the trigger (when active) and the option row', async () => {
    const { container, getAllByRole } = render(ContextPicker, {
      props: { contexts: [blank], activeId: 'p3', disabled: false, onpick: vi.fn(), onedit: vi.fn() },
    });
    expect(trigger(container).textContent?.trim()).toContain('Untitled');

    await fireEvent.click(trigger(container));
    const buttons = getAllByRole('button');
    // trigger + "No context" + 1 preset ("Untitled") + "Edit contexts…"
    expect(buttons).toHaveLength(4);
    expect(buttons[2]!.textContent?.trim()).toBe('Untitled');
  });

  it('selecting a preset calls onpick with its id and closes the panel', async () => {
    const onpick = vi.fn();
    const { container, getAllByRole, queryByText } = render(ContextPicker, {
      props: { contexts: [standup, acme], activeId: null, disabled: false, onpick, onedit: vi.fn() },
    });
    await fireEvent.click(trigger(container));
    const buttons = getAllByRole('button');
    await fireEvent.click(buttons[2]!); // "Standup"

    expect(onpick).toHaveBeenCalledOnce();
    expect(onpick).toHaveBeenCalledWith('p1');
    expect(queryByText('Edit contexts…')).toBeNull(); // panel closed
  });

  it('selecting "No context" calls onpick(null)', async () => {
    const onpick = vi.fn();
    const { container, getAllByRole } = render(ContextPicker, {
      props: { contexts: [standup], activeId: 'p1', disabled: false, onpick, onedit: vi.fn() },
    });
    await fireEvent.click(trigger(container));
    const buttons = getAllByRole('button');
    await fireEvent.click(buttons[1]!); // "No context"

    expect(onpick).toHaveBeenCalledOnce();
    expect(onpick).toHaveBeenCalledWith(null);
  });

  it('"Edit contexts…" calls onedit and never onpick', async () => {
    const onpick = vi.fn();
    const onedit = vi.fn();
    const { container, getByText, queryByText } = render(ContextPicker, {
      props: { contexts: [standup], activeId: null, disabled: false, onpick, onedit },
    });
    await fireEvent.click(trigger(container));
    await fireEvent.click(getByText('Edit contexts…'));

    expect(onedit).toHaveBeenCalledOnce();
    expect(onpick).not.toHaveBeenCalled();
    expect(queryByText('Edit contexts…')).toBeNull(); // panel closed
  });

  it('re-clicking the trigger closes an open panel', async () => {
    const { container, queryByText } = render(ContextPicker, {
      props: { contexts: [standup], activeId: null, disabled: false, onpick: vi.fn(), onedit: vi.fn() },
    });
    await fireEvent.click(trigger(container));
    expect(queryByText('Edit contexts…')).not.toBeNull();
    await fireEvent.click(trigger(container));
    expect(queryByText('Edit contexts…')).toBeNull();
  });

  it('when disabled, clicking the trigger does not open the panel', async () => {
    const { container, queryByText } = render(ContextPicker, {
      props: { contexts: [standup], activeId: null, disabled: true, onpick: vi.fn(), onedit: vi.fn() },
    });
    await fireEvent.click(trigger(container));
    expect(queryByText('Edit contexts…')).toBeNull();
  });
});
