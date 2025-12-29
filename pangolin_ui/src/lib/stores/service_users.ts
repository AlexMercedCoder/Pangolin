import { writable, derived, get } from 'svelte/store';
import { serviceUsersApi, type ServiceUser, type CreateServiceUserRequest, type UpdateServiceUserRequest } from '$lib/api/service_users';
import { notifications } from './notifications';
import { authStore } from './auth';

function createServiceUserStore() {
    const { subscribe, set, update } = writable<ServiceUser[]>([]);
    
    // Search state
    const searchQuery = writable('');

    return {
        subscribe,
        searchQuery,
        
        async load(limit = 100, offset = 0) {
            try {
                // Only load if authorized
                // Only load if authorized (Let API handle granular permissions, just check auth presence)
                const auth = get(authStore);
                if (!auth.token) {
                    set([]);
                    return;
                }

                const users = await serviceUsersApi.list(limit, offset);
                set(users || []);
            } catch (error) {
                console.error('Failed to load service users:', error);
                // Don't show notification for 403s on load, just empty list
                set([]); 
            }
        },

        async create(data: CreateServiceUserRequest) {
            try {
                const response = await serviceUsersApi.create(data);
                await this.load(); // Reload list
                notifications.success(`Service User '${data.name}' created`);
                return response; // Return full response with API Key
            } catch (error) {
                notifications.error(error instanceof Error ? error.message : 'Failed to create service user');
                throw error;
            }
        },

        async updateUser(id: string, data: UpdateServiceUserRequest) {
            try {
                await serviceUsersApi.update(id, data);
                update(users => users.map(u => u.id === id ? { ...u, ...data } : u));
                notifications.success('Service User updated');
            } catch (error) {
                notifications.error(error instanceof Error ? error.message : 'Failed to update service user');
                throw error;
            }
        },

        async rotateKey(id: string) {
            try {
                const response = await serviceUsersApi.rotateApiKey(id);
                notifications.success('API Key rotated successfully');
                return response;
            } catch (error) {
                notifications.error(error instanceof Error ? error.message : 'Failed to rotate API key');
                alert(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`); // Fail-safe
                throw error;
            }
        },

        async deleteUser(id: string) {
            try {
                await serviceUsersApi.delete(id);
                update(users => users.filter(u => u.id !== id));
                notifications.success('Service User deleted');
            } catch (error) {
                notifications.error(error instanceof Error ? error.message : 'Failed to delete service user');
                throw error;
            }
        }
    };
}

export const serviceUserStore = createServiceUserStore();

export const filteredServiceUsers = derived(
    [serviceUserStore, serviceUserStore.searchQuery],
    ([$users, $query]) => {
        if (!$query) return $users;
        const q = $query.toLowerCase();
        return $users.filter(u => 
            u.name.toLowerCase().includes(q) || 
            u.id.toLowerCase().includes(q) ||
            u.role.toLowerCase().includes(q)
        );
    }
);
