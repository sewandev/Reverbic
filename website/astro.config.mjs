// @ts-check
import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
	site: 'https://sewandev.github.io',
	base: '/Reverbic',
	i18n: {
		locales: ['en', 'es'],
		defaultLocale: 'en',
		routing: {
			prefixDefaultLocale: false,
		},
	},
});
