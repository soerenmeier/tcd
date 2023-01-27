import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import replace from '@rollup/plugin-replace';

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
	return {
		plugins: [
			svelte(),
			replace({
				preventAssignment: true,
				IN_DEBUG: mode !== 'production'
			})
		]
	};
});