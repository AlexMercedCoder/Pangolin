import { writable } from 'svelte/store';

export interface Notification {
	id: string;
	type: 'success' | 'error' | 'warning' | 'info';
	message: string;
	timeout?: number;
}

function createNotificationStore() {
	const { subscribe, update } = writable<Notification[]>([]);

	function add(notification: Omit<Notification, 'id'>) {
		const id = Math.random().toString(36).substring(7);
		const newNotification: Notification = {
			...notification,
			id,
			timeout: notification.timeout ?? 5000,
		};

		update((notifications) => [...notifications, newNotification]);

		if (newNotification.timeout) {
			setTimeout(() => {
				remove(id);
			}, newNotification.timeout);
		}

		return id;
	}

	function remove(id: string) {
		update((notifications) => notifications.filter((n) => n.id !== id));
	}

	function success(message: string, timeout?: number) {
		return add({ type: 'success', message, timeout });
	}

	function error(message: string, timeout?: number) {
		return add({ type: 'error', message, timeout });
	}

	function warning(message: string, timeout?: number) {
		return add({ type: 'warning', message, timeout });
	}

	function info(message: string, timeout?: number) {
		return add({ type: 'info', message, timeout });
	}

	return {
		subscribe,
		add,
		remove,
		success,
		error,
		warning,
		info,
	};
}

export const notifications = createNotificationStore();
