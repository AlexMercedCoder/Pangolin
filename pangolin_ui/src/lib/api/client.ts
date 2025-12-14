// API client utility for Pangolin backend
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export interface ApiError {
	message: string;
	status: number;
}

export class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string = API_URL) {
		this.baseUrl = baseUrl;
	}

	private async request<T>(
		endpoint: string,
		options: RequestInit = {}
	): Promise<T> {
		const token = localStorage.getItem('auth_token');
		
		const headers: HeadersInit = {
			'Content-Type': 'application/json',
			...options.headers,
		};

		if (token) {
			headers['Authorization'] = `Bearer ${token}`;
		}

		const response = await fetch(`${this.baseUrl}${endpoint}`, {
			...options,
			headers,
		});

		if (!response.ok) {
			const error: ApiError = {
				message: await response.text() || response.statusText,
				status: response.status,
			};
			throw error;
		}

		// Handle empty responses
		const contentType = response.headers.get('content-type');
		if (!contentType || !contentType.includes('application/json')) {
			return {} as T;
		}

		return response.json();
	}

	async get<T>(endpoint: string): Promise<T> {
		return this.request<T>(endpoint, { method: 'GET' });
	}

	async post<T>(endpoint: string, data?: unknown): Promise<T> {
		return this.request<T>(endpoint, {
			method: 'POST',
			body: data ? JSON.stringify(data) : undefined,
		});
	}

	async put<T>(endpoint: string, data?: unknown): Promise<T> {
		return this.request<T>(endpoint, {
			method: 'PUT',
			body: data ? JSON.stringify(data) : undefined,
		});
	}

	async delete<T>(endpoint: string): Promise<T> {
		return this.request<T>(endpoint, { method: 'DELETE' });
	}
}

export const apiClient = new ApiClient();
