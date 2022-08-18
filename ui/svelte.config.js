import preprocess from 'svelte-preprocess';
import { adapter } from 'sveltekit-adapter-aws';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://github.com/sveltejs/svelte-preprocess
	// for more information about preprocessors
	preprocess: preprocess(),

	kit: {
		adapter: adapter({
			artifactPath: 'build'
		})
	}
};

export default config;
