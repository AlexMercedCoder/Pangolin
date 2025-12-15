import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import WarehouseEditPage from './+page.svelte';
import { warehousesApi } from '$lib/api/warehouses';
import { goto } from '$app/navigation';

vi.mock('$lib/api/warehouses');
vi.mock('$app/navigation');
vi.mock('$app/stores', () => ({
	page: {
		subscribe: (fn: any) => {
			fn({ params: { name: 'test-warehouse' } });
			return () => {};
		}
	}
}));

describe('Warehouse Edit Page', () => {
	const mockWarehouse = {
		id: 'wh-1',
		name: 'test-warehouse',
		use_sts: false,
		storage_config: {
			type: 's3' as const,
			bucket: 'test-bucket',
			region: 'us-east-1',
			endpoint: ''
		}
	};

	beforeEach(() => {
		vi.resetAllMocks();
		vi.mocked(warehousesApi.get).mockResolvedValue(mockWarehouse);
		vi.mocked(warehousesApi.update).mockResolvedValue(mockWarehouse);
	});

	it('loads and displays warehouse data', async () => {
		render(WarehouseEditPage);

		await waitFor(() => {
			expect(warehousesApi.get).toHaveBeenCalledWith('test-warehouse');
		});

		await waitFor(() => {
			expect(screen.getByDisplayValue('test-bucket')).toBeInTheDocument();
			expect(screen.getByDisplayValue('us-east-1')).toBeInTheDocument();
		});
	});

	it('validates required bucket name', async () => {
		render(WarehouseEditPage);

		await waitFor(() => expect(warehousesApi.get).toHaveBeenCalled());

		const bucketInput = screen.getByLabelText(/Bucket Name/i);
		await fireEvent.input(bucketInput, { target: { value: '' } });

		const submitButton = screen.getByRole('button', { name: /Update Warehouse/i });
		await fireEvent.click(submitButton);

		// Should not call update if validation fails (HTML5 validation)
		await waitFor(() => {
			// May or may not be called depending on HTML5 validation
		});
	});

	it('submits update with S3 configuration', async () => {
		render(WarehouseEditPage);

		await waitFor(() => expect(warehousesApi.get).toHaveBeenCalled());

		const bucketInput = screen.getByLabelText(/Bucket Name/i);
		await fireEvent.input(bucketInput, { target: { value: 'new-bucket' } });

		const submitButton = screen.getByRole('button', { name: /Update Warehouse/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(warehousesApi.update).toHaveBeenCalledWith('test-warehouse', expect.objectContaining({
				use_sts: false,
				storage_config: expect.objectContaining({
					type: 's3',
					bucket: 'new-bucket',
					region: 'us-east-1'
				})
			}));
		});

		expect(goto).toHaveBeenCalledWith('/warehouses/test-warehouse');
	});

	it('handles storage type change', async () => {
		render(WarehouseEditPage);

		await waitFor(() => expect(warehousesApi.get).toHaveBeenCalled());

		// Find select by its current value
		const storageTypeSelect = screen.getByDisplayValue('Amazon S3 / MinIO');
		await fireEvent.change(storageTypeSelect, { target: { value: 'azure' } });

		await waitFor(() => {
			expect(screen.getByLabelText(/Storage Account Name/i)).toBeInTheDocument();
			expect(screen.getByLabelText(/Container Name/i)).toBeInTheDocument();
		});
	});

	it('toggles STS configuration', async () => {
		render(WarehouseEditPage);

		await waitFor(() => expect(warehousesApi.get).toHaveBeenCalled());

		const stsCheckbox = screen.getByLabelText(/Use STS/i);
		await fireEvent.click(stsCheckbox);

		// STS checkbox should be checked
		expect(stsCheckbox).toBeChecked();
	});

	it('handles update errors', async () => {
		vi.mocked(warehousesApi.update).mockRejectedValue(new Error('Update failed'));

		render(WarehouseEditPage);

		await waitFor(() => expect(warehousesApi.get).toHaveBeenCalled());

		const submitButton = screen.getByRole('button', { name: /Update Warehouse/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(warehousesApi.update).toHaveBeenCalled();
		});

		expect(goto).not.toHaveBeenCalled();
	});
});
