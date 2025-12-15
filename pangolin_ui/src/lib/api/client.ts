const API_URL = import.meta.env.VITE_API_URL || 'http://127.0.0.1:8080';

export interface ApiError {
	message: string;
	status: number;
	details?: any;
}

export interface ApiResponse<T> {
	data?: T;
	error?: ApiError;
}

class ApiClient {
	private async request<T>(
		method: string,
		path: string,
		data?: any
	): Promise<ApiResponse<T>> {
		try {
			const token = localStorage.getItem('auth_token');
			const headers: HeadersInit = {
				'Content-Type': 'application/json',
			};

			if (token && token !== 'no-auth-mode') {
				headers['Authorization'] = `Bearer ${token}`;
			}

			const options: RequestInit = {
				method,
				headers,
			};

			if (data && (method === 'POST' || method === 'PUT' || method === 'PATCH')) {
				options.body = JSON.stringify(data);
			}

			const response = await fetch(`${API_URL}${path}`, options);

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({}));
				return {
					error: {
						message: errorData.error || errorData.message || response.statusText,
						status: response.status,
						details: errorData,
					},
				};
			}

			// Handle 204 No Content
			if (response.status === 204) {
				return { data: undefined as T };
			}

			const responseData = await response.json();
			return { data: responseData };
		} catch (error: any) {
			return {
				error: {
					message: error.message || 'Network error',
					status: 0,
					details: error,
				},
			};
		}
	}

	async get<T>(path: string): Promise<ApiResponse<T>> {
		return this.request<T>('GET', path);
	}

	async post<T>(path: string, data?: any): Promise<ApiResponse<T>> {
		return this.request<T>('POST', path, data);
	}

	async put<T>(path: string, data?: any): Promise<ApiResponse<T>> {
		return this.request<T>('PUT', path, data);
	}

	async patch<T>(path: string, data?: any): Promise<ApiResponse<T>> {
		return this.request<T>('PATCH', path, data);
	}

	async delete<T>(path: string): Promise<ApiResponse<T>> {
		return this.request<T>('DELETE', path);
	}
}

export const apiClient = new ApiClient();
