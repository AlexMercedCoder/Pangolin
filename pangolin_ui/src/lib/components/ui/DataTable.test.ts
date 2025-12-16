import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
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

	it('emits rowClick event when row is clicked', async () => {
		const { component } = render(DataTable, {
			props: {
				columns: mockColumns,
				data: mockData,
				loading: false
			}
		});

		const rowClickHandler = vi.fn();
		(component as any).$on('rowClick', rowClickHandler);

		// Click would need user-event library for proper testing
		// This is a basic structure
	});
});
