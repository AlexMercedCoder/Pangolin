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

export const authApi = {
	async login(credentials: LoginRequest): Promise<LoginResponse> {
		return apiClient.post<LoginResponse>('/api/v1/login', credentials);
	},

	async logout(): Promise<void> {
		return apiClient.post<void>('/api/v1/logout');
	},

	async getCurrentUser(): Promise<User> {
		return apiClient.get<User>('/api/v1/me');
	},

	async getOAuthProviders(): Promise<OAuthProvider[]> {
		return apiClient.get<OAuthProvider[]>('/api/v1/oauth/providers');
	},

	async initiateOAuth(provider: string): Promise<{ url: string }> {
		return apiClient.get<{ url: string }>(`/api/v1/oauth/${provider}`);
	},
};
