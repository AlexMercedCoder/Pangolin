import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import UserEditPage from './+page.svelte';
import { usersApi } from '$lib/api/users';
import { goto } from '$app/navigation';

vi.mock('$lib/api/users');
vi.mock('$app/navigation');
vi.mock('$app/stores', () => ({
	page: {
		subscribe: (fn: any) => {
			fn({ params: { id: 'user-123' } });
			return () => {};
		}
	}
}));

describe('User Edit Page', () => {
	const mockUser = {
		id: 'user-123',
		username: 'testuser',
		email: 'test@example.com',
		role: 'TenantUser' as const,
		created_at: '2024-01-01T00:00:00Z'
	};

	beforeEach(() => {
		vi.resetAllMocks();
		vi.mocked(usersApi.get).mockResolvedValue(mockUser);
		vi.mocked(usersApi.update).mockResolvedValue(mockUser);
	});

	it('loads and displays user data', async () => {
		render(UserEditPage);

		await waitFor(() => {
			expect(usersApi.get).toHaveBeenCalledWith('user-123');
		});

		await waitFor(() => {
			expect(screen.getByDisplayValue('test@example.com')).toBeInTheDocument();
			expect(screen.getByDisplayValue('testuser')).toBeInTheDocument();
		});
	});

	it('validates email format', async () => {
		render(UserEditPage);

		await waitFor(() => expect(usersApi.get).toHaveBeenCalled());

		const emailInput = screen.getByLabelText(/Email/i);
		await fireEvent.input(emailInput, { target: { value: 'invalid-email' } });

		const submitButton = screen.getByRole('button', { name: /Update User/i });
		await fireEvent.click(submitButton);

		// Should not call update if validation fails
		expect(usersApi.update).not.toHaveBeenCalled();
	});

	it('submits update without password change', async () => {
		render(UserEditPage);

		await waitFor(() => expect(usersApi.get).toHaveBeenCalled());

		const emailInput = screen.getByLabelText(/Email/i);
		await fireEvent.input(emailInput, { target: { value: 'newemail@example.com' } });

		// Find select by its current value
		const roleSelect = screen.getByDisplayValue('TenantUser');
		await fireEvent.change(roleSelect, { target: { value: 'TenantAdmin' } });

		const submitButton = screen.getByRole('button', { name: /Update User/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(usersApi.update).toHaveBeenCalledWith('user-123', {
				email: 'newemail@example.com',
				role: 'TenantAdmin'
				// password should not be included
			});
		});

		expect(goto).toHaveBeenCalledWith('/users/user-123');
	});

	it('validates password length when changing password', async () => {
		render(UserEditPage);

		await waitFor(() => expect(usersApi.get).toHaveBeenCalled());

		const passwordInput = screen.getByLabelText(/New Password/i);
		await fireEvent.input(passwordInput, { target: { value: 'short' } });

		const submitButton = screen.getByRole('button', { name: /Update User/i });
		await fireEvent.click(submitButton);

		// Should not call update if password is too short
		expect(usersApi.update).not.toHaveBeenCalled();
	});

	it('validates password confirmation match', async () => {
		render(UserEditPage);

		await waitFor(() => expect(usersApi.get).toHaveBeenCalled());

		const passwordInput = screen.getByLabelText(/New Password/i);
		await fireEvent.input(passwordInput, { target: { value: 'newpassword123' } });

		// Confirm password field should appear
		await waitFor(() => {
			expect(screen.getByLabelText(/Confirm New Password/i)).toBeInTheDocument();
		});

		const confirmInput = screen.getByLabelText(/Confirm New Password/i);
		await fireEvent.input(confirmInput, { target: { value: 'differentpassword' } });

		const submitButton = screen.getByRole('button', { name: /Update User/i });
		await fireEvent.click(submitButton);

		// Should not call update if passwords don't match
		expect(usersApi.update).not.toHaveBeenCalled();
	});

	it('submits update with password change', async () => {
		render(UserEditPage);

		await waitFor(() => expect(usersApi.get).toHaveBeenCalled());

		const passwordInput = screen.getByLabelText(/New Password/i);
		await fireEvent.input(passwordInput, { target: { value: 'newpassword123' } });

		await waitFor(() => {
			expect(screen.getByLabelText(/Confirm New Password/i)).toBeInTheDocument();
		});

		const confirmInput = screen.getByLabelText(/Confirm New Password/i);
		await fireEvent.input(confirmInput, { target: { value: 'newpassword123' } });

		const submitButton = screen.getByRole('button', { name: /Update User/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(usersApi.update).toHaveBeenCalledWith('user-123', expect.objectContaining({
				email: 'test@example.com',
				role: 'TenantUser',
				password: 'newpassword123'
			}));
		});

		expect(goto).toHaveBeenCalledWith('/users/user-123');
	});

	it('handles update errors', async () => {
		vi.mocked(usersApi.update).mockRejectedValue(new Error('Update failed'));

		render(UserEditPage);

		await waitFor(() => expect(usersApi.get).toHaveBeenCalled());

		const submitButton = screen.getByRole('button', { name: /Update User/i });
		await fireEvent.click(submitButton);

		await waitFor(() => {
			expect(usersApi.update).toHaveBeenCalled();
		});

		expect(goto).not.toHaveBeenCalled();
	});

	it('allows canceling edit', async () => {
		render(UserEditPage);

		await waitFor(() => expect(usersApi.get).toHaveBeenCalled());

		const cancelButton = screen.getByRole('button', { name: /Cancel/i });
		await fireEvent.click(cancelButton);

		expect(goto).toHaveBeenCalledWith('/users/user-123');
		expect(usersApi.update).not.toHaveBeenCalled();
	});
});
