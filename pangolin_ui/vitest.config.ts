import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
	plugins: [sveltekit()],
	// B46: there used to be a `define: { 'import.meta.env': {...} }` here, which
	// replaced `import.meta.env` *wholesale*. SvelteKit's virtual
	// `$env/dynamic/public` module reads `import.meta.env.<X>` off the real
	// object, so the override left it undefined and every suite importing the
	// API client failed to load at all with "Cannot read properties of undefined
	// (reading 'env')". `test.env` below sets the same value without clobbering
	// anything.
	resolve: {
		conditions: ['browser']
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		globals: true,
		environment: 'jsdom',
		setupFiles: ['./src/tests/setup.ts'],
    env: {
        PUBLIC_API_URL: 'http://localhost:8080'
    },
		coverage: {
			provider: 'v8',
			reporter: ['text', 'json', 'html'],
			exclude: [
				'node_modules/',
				'src/tests/',
				'**/*.spec.ts',
				'**/*.test.ts'
			]
		}
	}
});
