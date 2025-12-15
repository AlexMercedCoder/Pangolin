import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';

// Key for localStorage
export const TENANT_STORAGE_KEY = 'pangolin_selected_tenant';

export interface TenantState {
	selectedTenantId: string | null;
	selectedTenantName: string | null;
}

function createTenantStore() {
	// Initialize from localStorage if available
	let initialId: string | null = null;
	let initialName: string | null = null;

	if (browser) {
		initialId = localStorage.getItem(TENANT_STORAGE_KEY);
		initialName = localStorage.getItem(`${TENANT_STORAGE_KEY}_name`);
	}

	const { subscribe, set, update } = writable<TenantState>({
		selectedTenantId: initialId,
		selectedTenantName: initialName
	});

	return {
		subscribe,
		selectTenant: (id: string, name: string) => {
			if (browser) {
				localStorage.setItem(TENANT_STORAGE_KEY, id);
				localStorage.setItem(`${TENANT_STORAGE_KEY}_name`, name);
			}
			set({ selectedTenantId: id, selectedTenantName: name });
		},
		clearTenant: () => {
			if (browser) {
				localStorage.removeItem(TENANT_STORAGE_KEY);
				localStorage.removeItem(`${TENANT_STORAGE_KEY}_name`);
			}
			set({ selectedTenantId: null, selectedTenantName: null });
		}
	};
}

export const tenantStore = createTenantStore();
