export const languages = { en: 'EN', es: 'ES' } as const;

export type Lang = keyof typeof languages;

export const defaultLang: Lang = 'en';

export const ui = {
	en: {
		'meta.description':
			'Reverbic is an ultra-light terminal player for global Radio, Spotify and YouTube. Built in Rust for Windows, macOS and Linux.',
		'nav.download': 'Download',
		'hero.title': 'The terminal has never sounded',
		'hero.title_accent': 'this good',
		'hero.subtitle':
			'An ultra-light player for global Radio, Spotify and YouTube, straight from your console. Built in Rust for Windows, macOS and Linux.',
		'cta.download': 'Download',
		'cta.source': 'View source',
		'carousel.aria': 'Reverbic screenshots',
		'carousel.prev': 'Previous screenshot',
		'carousel.next': 'Next screenshot',
		'carousel.slide_role': 'slide',
		'carousel.of': 'of',
		'carousel.dot_label': 'Go to screenshot',
		'features.title': 'Key Features',
		'feature.radio.title': 'Global Radio',
		'feature.radio.desc':
			'Search and play thousands of radio stations by name, genre or country, instantly.',
		'feature.spotify.title': 'Spotify Control',
		'feature.spotify.desc':
			'Search, play, pause and transfer between devices via remote control (Premium required for playback).',
		'feature.youtube.title': 'YouTube',
		'feature.youtube.desc':
			'Search and play audio straight from YouTube without leaving the terminal.',
		'feature.overlay.title': 'Gaming Overlay',
		'feature.overlay.desc':
			'Floating widget with automatic game detection to see what is playing without interrupting your matches.',
		'feature.light.title': 'Light & Fast',
		'feature.light.p1': 'Built in Rust. Uses just',
		'feature.light.p2': 'of RAM and',
		'feature.light.p3': 'CPU at idle. No installers.',
		'feature.themes.title': 'Themes your way',
		'feature.themes.desc':
			'Over 20 dark themes with live preview: Nord, Gruvbox, Tokyo Night, Night Owl and more.',
		'feature.login.title': 'Sign in',
		'feature.login.desc':
			'Connect your Spotify account via OAuth and load your YouTube cookies to access restricted content.',
		'feature.sponsorblock.title': 'SponsorBlock',
		'feature.sponsorblock.desc':
			'Automatically skips intros, ads and non-music segments in YouTube videos.',
		'footer.made_by': 'Made with care by',
		'shot.player.alt':
			'Reverbic interface playing music: clock, progress bar, visualizer and recent tracks.',
		'shot.player.title': 'Live player',
		'shot.player.desc': 'Clock, track progress, visualizer and your recent history at a glance.',
		'shot.radio.alt': 'Radio search in Reverbic showing Tomorrowland One World Radio stations.',
		'shot.radio.title': 'Global radio',
		'shot.radio.desc': 'Thousands of stations searchable instantly by name, genre or country.',
		'shot.spotify.alt': 'Spotify Top Tracks list in Reverbic with a connected Premium account.',
		'shot.spotify.title': 'Spotify control',
		'shot.spotify.desc':
			'Play, transfer between devices and check your Top Tracks via remote control.',
		'shot.youtube.alt': 'YouTube search in Reverbic with song results and keyboard shortcuts.',
		'shot.youtube.title': 'YouTube search',
		'shot.youtube.desc': 'Find and play YouTube audio without leaving the terminal.',
		'shot.theme.alt': 'Reverbic theme picker with Rose, Nord, Gruvbox and Night Owl palettes.',
		'shot.theme.title': 'Themes your way',
		'shot.theme.desc': 'Rose, Nord, Gruvbox, Tokyo Night and many more built-in palettes.',
		'shot.settings.alt': 'Reverbic settings panel with Spotify and YouTube options.',
		'shot.settings.title': 'Tailored settings',
		'shot.settings.desc':
			'Crossfade, SponsorBlock, game overlay and infinite radio, all configurable.',
	},
	es: {
		'meta.description':
			'Reverbic es un reproductor de terminal ultraligero para Radio global, Spotify y YouTube. Construido en Rust para Windows, macOS y Linux.',
		'nav.download': 'Descargar',
		'hero.title': 'La terminal nunca sonó',
		'hero.title_accent': 'tan bien',
		'hero.subtitle':
			'Un reproductor ultraligero de Radio global, Spotify y YouTube directamente desde tu consola. Construido en Rust para Windows, macOS y Linux.',
		'cta.download': 'Descargar',
		'cta.source': 'Ver código fuente',
		'carousel.aria': 'Capturas de Reverbic',
		'carousel.prev': 'Captura anterior',
		'carousel.next': 'Captura siguiente',
		'carousel.slide_role': 'diapositiva',
		'carousel.of': 'de',
		'carousel.dot_label': 'Ir a la captura',
		'features.title': 'Características Principales',
		'feature.radio.title': 'Radio Global',
		'feature.radio.desc':
			'Busca y reproduce miles de estaciones de radio por nombre, género o país al instante.',
		'feature.spotify.title': 'Control Spotify',
		'feature.spotify.desc':
			'Busca, reproduce, pausa y transfiere de dispositivo mediante control remoto (Premium requerido para reproducir).',
		'feature.youtube.title': 'YouTube',
		'feature.youtube.desc':
			'Busca y reproduce audio directamente desde YouTube sin salir de la terminal.',
		'feature.overlay.title': 'Gaming Overlay',
		'feature.overlay.desc':
			'Widget flotante con detección automática de juegos para ver qué está sonando sin interrumpir tus partidas.',
		'feature.light.title': 'Ligero y Rápido',
		'feature.light.p1': 'Desarrollado en Rust. Consume apenas',
		'feature.light.p2': 'de RAM y',
		'feature.light.p3': 'de CPU en reposo. Sin instaladores.',
		'feature.themes.title': 'Temas a tu gusto',
		'feature.themes.desc':
			'Más de 20 temas oscuros con previsualización en vivo: Nord, Gruvbox, Tokyo Night, Night Owl y más.',
		'feature.login.title': 'Inicia sesión',
		'feature.login.desc':
			'Conecta tu cuenta de Spotify por OAuth y carga tus cookies de YouTube para acceder a contenido restringido.',
		'feature.sponsorblock.title': 'SponsorBlock',
		'feature.sponsorblock.desc':
			'Salta automáticamente intros, anuncios y segmentos no musicales en los videos de YouTube.',
		'footer.made_by': 'Hecho con dedicación por',
		'shot.player.alt':
			'Interfaz de Reverbic reproduciendo música: reloj, barra de progreso, visualizador y canciones recientes.',
		'shot.player.title': 'Reproductor en vivo',
		'shot.player.desc':
			'Reloj, progreso de pista, visualizador y tu historial reciente de un vistazo.',
		'shot.radio.alt':
			'Búsqueda de radio en Reverbic mostrando estaciones de Tomorrowland One World Radio.',
		'shot.radio.title': 'Radio global',
		'shot.radio.desc': 'Miles de estaciones buscables al instante por nombre, género o país.',
		'shot.spotify.alt':
			'Lista de Top Tracks de Spotify en Reverbic con una cuenta Premium conectada.',
		'shot.spotify.title': 'Control de Spotify',
		'shot.spotify.desc':
			'Reproduce, transfiere de dispositivo y revisa tus Top Tracks vía control remoto.',
		'shot.youtube.alt':
			'Búsqueda de YouTube en Reverbic con resultados de canciones y atajos de teclado.',
		'shot.youtube.title': 'Búsqueda en YouTube',
		'shot.youtube.desc': 'Encuentra y reproduce audio de YouTube sin salir de la terminal.',
		'shot.theme.alt':
			'Selector de temas de Reverbic con paletas Rose, Nord, Gruvbox y Night Owl.',
		'shot.theme.title': 'Temas a tu gusto',
		'shot.theme.desc': 'Rose, Nord, Gruvbox, Tokyo Night y muchas más paletas integradas.',
		'shot.settings.alt': 'Panel de configuración de Reverbic con opciones de Spotify y YouTube.',
		'shot.settings.title': 'Configuración a medida',
		'shot.settings.desc':
			'Crossfade, SponsorBlock, overlay de juego y radio infinita configurables.',
	},
} as const;

export function useTranslations(lang: Lang) {
	const dict = ui[lang] ?? ui[defaultLang];
	return function t(key: keyof (typeof ui)['en']): string {
		return dict[key] ?? ui[defaultLang][key];
	};
}
