import { apiClient } from './client';

export interface TokenInfo {
	id: string;
	user_id: string;
	created_at: string;
	expires_at: string;
	is_valid: boolean;
}

export const tokensApi = {
	async listUserTokens(userId: string): Promise<TokenInfo[]> {
		const response = await apiClient.get<TokenInfo[]>(`/api/v1/users/${userId}/tokens`);
		if (response.error) throw new Error(response.error.message);
		return response.data || [];
	},

	async deleteToken(tokenId: string): Promise<void> {
		const response = await apiClient.delete<void>(`/api/v1/tokens/${tokenId}`);
		if (response.error) throw new Error(response.error.message);
	},
};
