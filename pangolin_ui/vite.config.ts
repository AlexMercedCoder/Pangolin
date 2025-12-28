import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
    server: {
        proxy: {
            '/api': 'http://localhost:8085',
            '/v1': 'http://localhost:8085',
            '/oauth': 'http://localhost:8085'
        }
    }
});
