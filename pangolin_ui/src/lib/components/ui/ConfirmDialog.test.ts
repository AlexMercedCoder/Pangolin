import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import ConfirmDialog from './ConfirmDialog.svelte';

describe('ConfirmDialog', () => {
  it('renders correctly when open', () => {
    const { getByText } = render(ConfirmDialog, { open: true, title: 'Test Title', message: 'Test Message' });
    expect(getByText('Test Title')).toBeTruthy();
    expect(getByText('Test Message')).toBeTruthy();
  });

  it('dispatches confirm event on click', async () => {
    const confirmHandler = vi.fn();
    const { getByText } = render(ConfirmDialog, { 
        open: true, 
        confirmText: 'Yes',
        onConfirm: confirmHandler
    });

    await fireEvent.click(getByText('Yes'));
    expect(confirmHandler).toHaveBeenCalled();
  });

  it('dispatches cancel event on click', async () => {
    const cancelHandler = vi.fn();
    const { getByText } = render(ConfirmDialog, { 
        open: true, 
        cancelText: 'No',
        onCancel: cancelHandler
    });

    await fireEvent.click(getByText('No'));
    expect(cancelHandler).toHaveBeenCalled();
  });
});
