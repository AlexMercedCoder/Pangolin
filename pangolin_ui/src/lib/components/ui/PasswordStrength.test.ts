import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import PasswordStrength from './PasswordStrength.svelte';

describe('PasswordStrength', () => {
	it('renders nothing when password is empty', () => {
		const { container } = render(PasswordStrength, { password: '' });
		expect(container.querySelector('.mb-4')).toBeNull();
	});

	it('shows weak strength for short password', () => {
		const { getByText } = render(PasswordStrength, { password: 'abc' });
		expect(getByText('Weak')).toBeTruthy();
	});

	it('shows medium strength for password with some requirements', () => {
		// Meets 3 requirements: Length, Uppercase, Lowercase (60%)
		const { getByText } = render(PasswordStrength, { password: 'Abcdefgh' });
		expect(getByText('Medium')).toBeTruthy();
	});

	it('shows strong strength for password meeting all requirements', () => {
		const { getByText } = render(PasswordStrength, { password: 'Abc123!@#' });
		expect(getByText('Strong')).toBeTruthy();
	});

	it('displays all password requirements', () => {
		const { getByText } = render(PasswordStrength, { password: 'test' });
		
		expect(getByText('At least 8 characters')).toBeTruthy();
		expect(getByText('Contains uppercase letter')).toBeTruthy();
		expect(getByText('Contains lowercase letter')).toBeTruthy();
		expect(getByText('Contains number')).toBeTruthy();
		expect(getByText('Contains special character')).toBeTruthy();
	});

	it('marks requirements as met when password satisfies them', () => {
		const { container } = render(PasswordStrength, { password: 'Abc123!@#' });
		
		// All requirements should have green checkmarks (green icon + green text)
		// The component uses text-green-500/600.
		// Let's count SVG elements with text-green-500
		const greenIcons = container.querySelectorAll('svg.text-green-500');
		expect(greenIcons.length).toBe(5);
	});

	it('updates strength dynamically as password changes', async () => {
		const { component, getByText, queryByText, rerender } = render(PasswordStrength, { password: 'weak' });
		
		// Initially weak (1 requirement met: lowercase -> 20%)
		expect(getByText('Weak')).toBeTruthy();
		
		// Update to strong password
		await rerender({ password: 'StrongP@ss123' });
		
		// Should now be strong
		expect(queryByText('Weak')).toBeNull();
		expect(getByText('Strong')).toBeTruthy();
	});
});
