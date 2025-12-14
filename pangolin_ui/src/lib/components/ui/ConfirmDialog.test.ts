import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';

describe('ConfirmDialog Component', () => {
	it('renders when open is true', () => {
		render(ConfirmDialog, {
			props: {
				open: true,
				title: 'Confirm Action',
				message: 'Are you sure?',
				variant: 'danger'
			}
		});

		expect(screen.getByText('Confirm Action')).toBeInTheDocument();
		expect(screen.getByText('Are you sure?')).toBeInTheDocument();
	});

	it('does not render when open is false', () => {
		render(ConfirmDialog, {
			props: {
				open: false,
				title: 'Confirm Action',
				message: 'Are you sure?'
			}
		});

		expect(screen.queryByText('Confirm Action')).not.toBeInTheDocument();
	});

	it('shows correct button text', () => {
		render(ConfirmDialog, {
			props: {
				open: true,
				title: 'Delete Item',
				message: 'This cannot be undone',
				confirmText: 'Delete',
				cancelText: 'Keep It'
			}
		});

		expect(screen.getByText('Delete')).toBeInTheDocument();
		expect(screen.getByText('Keep It')).toBeInTheDocument();
	});

	it('applies danger variant styling', () => {
		render(ConfirmDialog, {
			props: {
				open: true,
				title: 'Delete',
				message: 'Confirm delete',
				variant: 'danger'
			}
		});

		const confirmButton = screen.getByText('Confirm');
		expect(confirmButton.className).toContain('bg-red');
	});

	it('shows loading state', () => {
		render(ConfirmDialog, {
			props: {
				open: true,
				title: 'Delete',
				message: 'Confirm delete',
				loading: true
			}
		});

		const confirmButton = screen.getByText('Confirm');
		expect(confirmButton).toBeDisabled();
	});
});
