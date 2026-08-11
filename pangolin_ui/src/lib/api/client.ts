import { TENANT_STORAGE_KEY } from '$lib/stores/tenant';

import { browser, dev } from '$app/environment';
import { env } from '$env/dynamic/public';

/**
 * Base URL for the Pangolin API.
 *
 * B31: this read `env.PUBLIC_API_URL`, but nothing ever set it. `.env.example`
 * declared `VITE_API_URL`, `docker-compose.yml` passed `VITE_API_URL`, and the
 * login page read a third variant off `import.meta.env`. So the fallback always
 * won, and every deployed build called `http://localhost:8080` - which is the
 * *end user's* machine, not the server. `PUBLIC_API_URL` is the correct name
 * (SvelteKit's dynamic public env requires the `PUBLIC_` prefix) and is now the
 * only one, set consistently in the compose files and `.env.example`.
 *
 * The default is now the empty string, meaning same-origin: that is the right
 * behaviour behind a reverse proxy, and it fails visibly rather than silently
 * pointing at the visitor's own localhost.
 */
export const API_URL = resolveApiUrl();

function resolveApiUrl(): string {
	const configured = env.PUBLIC_API_URL?.trim();
	if (configured) {
		return configured.replace(/\/$/, '');
	}

	// Local development convenience only: `vite dev` serves the UI on 5173 while
	// the API runs on 8080, so same-origin would not reach it.
	if (dev) {
		return 'http://localhost:8080';
	}

	// Production: same origin, for a reverse-proxy deployment.
	return '';
}

export interface ApiError {
	message: string;
	status: number;
	details?: any;
}

export interface ApiResponse<T> {
	data?: T;
	error?: ApiError;
}

/**
 * Called when the server rejects our credentials.
 *
 * B34: there was no 401 handling anywhere in `src/`. An expired JWT left the
 * user in a permanently broken "authenticated" session: `auth_token` was never
 * cleared, nothing redirected, and every subsequent request failed with an
 * error the UI rendered as a generic message. Registered by the root layout so
 * this module stays free of a store/router import cycle.
 */
type UnauthorizedHandler = () => void;
let onUnauthorized: UnauthorizedHandler | null = null;

export function setUnauthorizedHandler(handler: UnauthorizedHandler | null) {
	onUnauthorized = handler;
}

/** Pull a human-readable message out of whichever error envelope came back. */
function extractMessage(errorData: any, fallback: string): string {
	if (!errorData || typeof errorData !== 'object') {
		return fallback;
	}

	// The Iceberg endpoints use the spec envelope: { error: { message, type, code } }.
	if (errorData.error && typeof errorData.error === 'object') {
		return errorData.error.message || fallback;
	}

	// The management endpoints use a flat { error: "..." }.
	if (typeof errorData.error === 'string') {
		return errorData.error;
	}

	if (typeof errorData.message === 'string') {
		return errorData.message;
	}

	// Anything else would previously have been interpolated into a string as
	// "[object Object]".
	return fallback;
}

class ApiClient {
	private async request<T>(method: string, path: string, data?: any): Promise<ApiResponse<T>> {
		try {
			const token = browser ? localStorage.getItem('auth_token') : null;
			const tenantId = browser ? localStorage.getItem(TENANT_STORAGE_KEY) : null;

			const headers: HeadersInit = {
				'Content-Type': 'application/json'
			};

			if (token && token !== 'no-auth-mode') {
				headers['Authorization'] = `Bearer ${token}`;
			}

			if (tenantId) {
				headers['X-Pangolin-Tenant'] = tenantId;
			}

			const options: RequestInit = {
				method,
				headers
			};

			if (data && (method === 'POST' || method === 'PUT' || method === 'PATCH')) {
				options.body = JSON.stringify(data);
			}

			const response = await fetch(`${API_URL}${path}`, options);

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({}));

				// B34: an expired or revoked token ends the session here rather
				// than leaving the user clicking through a UI that can no longer
				// load anything.
				if (response.status === 401 && onUnauthorized) {
					onUnauthorized();
				}

				return {
					error: {
						message: extractMessage(errorData, response.statusText),
						status: response.status,
						details: errorData
					}
				};
			}

			// Handle 204 No Content
			if (response.status === 204) {
				return { data: undefined as T };
			}

			const text = await response.text();
			if (!text) {
				return { data: undefined as T };
			}

			try {
				const responseData = JSON.parse(text);
				return { data: responseData };
			} catch {
				// The endpoints all return JSON; anything else is a proxy or
				// gateway page, which is worth surfacing verbatim.
				throw new Error('Invalid JSON response: ' + text.substring(0, 100));
			}
		} catch (error: any) {
			return {
				error: {
					message: error.message || 'Network error',
					status: 0,
					details: error
				}
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
