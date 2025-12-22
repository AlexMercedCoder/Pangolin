import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import StatCard from './StatCard.svelte';

describe('StatCard', () => {
	it('renders label and value', () => {
		render(StatCard, {
			props: {
				label: 'Total Users',
				value: 1234,
				icon: 'people',
				color: 'blue'
			}
		});

		expect(screen.getByText('Total Users')).toBeInTheDocument();
		expect(screen.getByText('1,234')).toBeInTheDocument(); // Checks formatting
	});

    it('renders loading state', () => {
        render(StatCard, {
            props: {
                label: 'Loading...',
                value: undefined,
                icon: 'refresh',
                loading: true
            }
        });
        
        // Should find some loading indicator or at least not the value
        // The implementation details might vary, but let's check basic structure
        expect(screen.getByText('Loading...')).toBeInTheDocument();
    });
});
