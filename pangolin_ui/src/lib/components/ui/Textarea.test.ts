import { describe, it, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Textarea from './Textarea.svelte';

describe('Textarea', () => {
	it('renders with label', () => {
		const { getByText } = render(Textarea, { label: 'Description' });
		expect(getByText('Description')).toBeTruthy();
	});

	it('shows required indicator when required', () => {
		const { container } = render(Textarea, { label: 'Required Field', required: true });
		const asterisk = container.querySelector('.text-red-500');
		expect(asterisk).toBeTruthy();
		expect(asterisk?.textContent).toBe('*');
	});

	it('displays help text when provided', () => {
		const { getByText } = render(Textarea, { 
			helpText: 'Enter a detailed description' 
		});
		expect(getByText('Enter a detailed description')).toBeTruthy();
	});

	it('displays error message when error is provided', () => {
		const { getByText } = render(Textarea, { 
			error: 'This field is required' 
		});
		expect(getByText('This field is required')).toBeTruthy();
	});

	it('shows character counter when maxlength is set', () => {
		const { getByText } = render(Textarea, { 
			value: 'Hello',
			maxlength: 100 
		});
		expect(getByText('5/100')).toBeTruthy();
	});

	it('updates character count as user types', async () => {
		const { container, getByText } = render(Textarea, { 
			value: '',
			maxlength: 50 
		});
		
		const textarea = container.querySelector('textarea');
		await fireEvent.input(textarea!, { target: { value: 'Test message' } });
		expect(getByText('12/50')).toBeTruthy();
	});

	it('applies error styling when error is present', () => {
		const { container } = render(Textarea, { 
			error: 'Invalid input' 
		});
		const textarea = container.querySelector('textarea');
		expect(textarea?.className).toContain('border-red-500');
	});

	it('can be disabled', () => {
		const { container } = render(Textarea, { disabled: true });
		const textarea = container.querySelector('textarea');
		expect(textarea?.disabled).toBe(true);
	});

	it('respects rows prop', () => {
		const { container } = render(Textarea, { rows: 10 });
		const textarea = container.querySelector('textarea');
		expect(textarea?.rows).toBe(10);
	});
});
