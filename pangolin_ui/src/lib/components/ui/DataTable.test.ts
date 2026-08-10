import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import DataTable from '$lib/components/ui/DataTable.svelte';

describe('DataTable Component', () => {
	const mockColumns = [
		{ key: 'name', label: 'Name', sortable: true },
		{ key: 'type', label: 'Type', sortable: false }
	];

	const mockData = [
		{ name: 'Item 1', type: 'TypeA' },
		{ name: 'Item 2', type: 'TypeB' },
		{ name: 'Item 3', type: 'TypeA' }
	];

	it('renders table with data', () => {
		render(DataTable, {
			props: {
				columns: mockColumns,
				data: mockData,
				loading: false
			}
		});

		expect(screen.getByText('Name')).toBeInTheDocument();
		expect(screen.getByText('Type')).toBeInTheDocument();
		expect(screen.getByText('Item 1')).toBeInTheDocument();
		expect(screen.getByText('Item 2')).toBeInTheDocument();
	});

	it('shows loading state', () => {
		render(DataTable, {
			props: {
				columns: mockColumns,
				data: [],
				loading: true
			}
		});

		expect(screen.getByText(/loading/i)).toBeInTheDocument();
	});

	it('shows empty message when no data', () => {
		const emptyMessage = 'No items found';
		render(DataTable, {
			props: {
				columns: mockColumns,
				data: [],
				loading: false,
				emptyMessage
			}
		});

		expect(screen.getByText(emptyMessage)).toBeInTheDocument();
	});

	it('filters data based on search', async () => {
		const { component } = render(DataTable, {
			props: {
				columns: mockColumns,
				data: mockData,
				loading: false,
				searchPlaceholder: 'Search...'
			}
		});

		const searchInput = screen.getByPlaceholderText('Search...');
		expect(searchInput).toBeInTheDocument();
	});

	// This was a stub: it subscribed with `component.$on('rowClick', ...)`,
	// which Svelte 5 removed, and then asserted nothing at all - so it neither
	// passed nor tested anything. `onRowClick` is a plain prop, so the spy goes
	// in with the rest of them and the click can actually be made.
	it('calls onRowClick with the row that was clicked', async () => {
		const onRowClick = vi.fn();
		render(DataTable, {
			props: {
				columns: mockColumns,
				data: mockData,
				loading: false,
				onRowClick
			}
		});

		await fireEvent.click(screen.getByText('Item 1'));

		expect(onRowClick).toHaveBeenCalledTimes(1);
		expect(onRowClick).toHaveBeenCalledWith(mockData[0]);
	});

	it('does not fail when no onRowClick is supplied', async () => {
		render(DataTable, {
			props: { columns: mockColumns, data: mockData, loading: false }
		});

		// The callback is optional; clicking without one must be a no-op rather
		// than a TypeError.
		await fireEvent.click(screen.getByText('Item 1'));
	});
});
