import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { notifications } from '$lib/stores/notifications';

describe('Notifications Store', () => {
	it('adds success notification', () => {
		notifications.success('Test success');
		
		const current = get(notifications);
		expect(current.length).toBe(1);
		expect(current[0].type).toBe('success');
		expect(current[0].message).toBe('Test success');
	});

	it('adds error notification', () => {
		notifications.clear();
		notifications.error('Test error');
		
		const current = get(notifications);
		expect(current.length).toBe(1);
		expect(current[0].type).toBe('error');
		expect(current[0].message).toBe('Test error');
	});

	it('adds info notification', () => {
		notifications.clear();
		notifications.info('Test info');
		
		const current = get(notifications);
		expect(current.length).toBe(1);
		expect(current[0].type).toBe('info');
		expect(current[0].message).toBe('Test info');
	});

	it('removes notification by id', () => {
		notifications.clear();
		notifications.success('Test 1');
		notifications.success('Test 2');
		
		let current = get(notifications);
		expect(current.length).toBe(2);
		
		const firstId = current[0].id;
		notifications.remove(firstId);
		
		current = get(notifications);
		expect(current.length).toBe(1);
		expect(current[0].message).toBe('Test 2');
	});

	it('clears all notifications', () => {
		notifications.success('Test 1');
		notifications.error('Test 2');
		notifications.info('Test 3');
		
		let current = get(notifications);
		expect(current.length).toBe(3);
		
		notifications.clear();
		
		current = get(notifications);
		expect(current.length).toBe(0);
	});

	it('assigns unique IDs to notifications', () => {
		notifications.clear();
		notifications.success('Test 1');
		notifications.success('Test 2');
		
		const current = get(notifications);
		expect(current[0].id).not.toBe(current[1].id);
	});
});
