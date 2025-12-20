import { apiClient } from './client';

export interface SystemSettings {
    allow_public_signup?: boolean;
    default_warehouse_bucket?: string;
    default_retention_days?: number;
    smtp_host?: string;
    smtp_port?: number;
    smtp_user?: string;
    smtp_password?: string;
}

const BASE_PATH = '/api/v1/config/settings';

export const systemConfigApi = {
    getSettings: async (): Promise<SystemSettings> => {
        const response = await apiClient.get<SystemSettings>(BASE_PATH);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    },

    updateSettings: async (settings: SystemSettings): Promise<SystemSettings> => {
        const response = await apiClient.put<SystemSettings>(BASE_PATH, settings);
        if (response.error) throw new Error(response.error.message);
        return response.data!;
    }
};
