import { apiClient } from './client';

export interface LoginRequest {
	username: string;
	password: string;
}

export interface LoginResponse {
	token: string;
	user: User;
}

export interface User {
	id: string;
	username: string;
	role: 'Root' | 'TenantAdmin' | 'TenantUser';
	tenant_id?: string;
	tenant_name?: string;
}

export interface OAuthProvider {
	name: string;
	url: string;
}

export interface AppConfig {
	auth_enabled: boolean;
}

export const authApi = {
	async getAppConfig(): Promise<AppConfig> {
		const response = await apiClient.get<AppConfig>('/api/v1/app-config');
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async login(credentials: LoginRequest): Promise<LoginResponse> {
		const response = await apiClient.post<LoginResponse>('/api/v1/users/login', credentials);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async logout(): Promise<void> {
		const response = await apiClient.post<void>('/api/v1/users/logout');
		if (response.error) throw new Error(response.error.message);
		return response.data;
	},

	async getCurrentUser(): Promise<User> {
		const response = await apiClient.get<User>('/api/v1/users/me');
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},

	async getOAuthProviders(): Promise<OAuthProvider[]> {
		const response = await apiClient.get<OAuthProvider[]>('/api/v1/oauth/providers');
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async initiateOAuth(provider: string): Promise<{ url: string }> {
		const response = await apiClient.get<{ url: string }>(`/api/v1/oauth/${provider}`);
		if (response.error) throw new Error(response.error.message);
		return response.data!;
	},
};
