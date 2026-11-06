import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [
        tailwindcss(),
        sveltekit({
            preprocess: vitePreprocess(),
            adapter: adapter(),
            // Same behaviour as the original frontend (`csrf.checkOrigin: false`):
            // form actions accept posts from any origin. Tighten this to your
            // public origin(s) once the deployment's ORIGIN is known.
            csrf: {
                trustedOrigins: ['*']
            }
        })
    ],
    // The backend's CORS allow-list accepts http://localhost:5173 (and :3000).
    server: { port: 5173 },
    preview: { port: 5173 }
});
