/**
 * Focus — Writing mode system for Constellation.
 * Each focus provides a unique writing experience, adapted per culture/script.
 */

export type FocusType = 'none' | 'blankPage' | 'typewriter' | 'manuscript' | 'flow';

export interface FocusConfig {
	/** CSS class applied to the editor wrapper */
	className: string;
	/** Whether to hide the toolbar */
	hideToolbar: boolean;
	/** Whether to hide the breadcrumb */
	hideBreadcrumb: boolean;
	/** Whether to hide the properties panel */
	hideProperties: boolean;
	/** Whether to hide the sidebar */
	hideSidebar: boolean;
	/** Whether to hide the tab bar */
	hideTabBar: boolean;
	/** Whether to hide the status bar */
	hideStatusBar: boolean;
	/** Whether to enable typewriter scrolling (keep cursor line centered) */
	typewriterScroll: boolean;
	/** Max content width (CSS value) */
	maxWidth: string;
	/** Font family override per script */
	fonts: Record<string, string>;
	/** Line height override */
	lineHeight: string;
	/** Extra padding */
	padding: string;
	/** Background style */
	background: string;
}

/** Culture-adaptive font maps for each focus */
const FOCUS_FONTS: Record<FocusType, Record<string, string>> = {
	none: {},
	blankPage: {
		latin: 'Inter, -apple-system, sans-serif',
		arabic: "'Noto Naskh Arabic', 'Segoe UI', sans-serif",
		hebrew: "'Noto Sans Hebrew', 'Segoe UI', sans-serif",
		cjk: "'Noto Sans CJK SC', 'Microsoft YaHei', sans-serif",
		devanagari: "'Noto Sans Devanagari', sans-serif",
		cyrillic: 'Inter, -apple-system, sans-serif',
	},
	typewriter: {
		latin: "'Courier Prime', 'Courier New', Courier, monospace",
		arabic: "'Traditional Arabic', 'Sakkal Majalla', monospace",
		hebrew: "'Cousine', 'Courier New', monospace",
		cjk: "'Noto Sans Mono CJK SC', monospace",
		devanagari: "'Noto Sans Devanagari', monospace",
		cyrillic: "'Courier Prime', 'Courier New', monospace",
	},
	manuscript: {
		latin: "'EB Garamond', Georgia, 'Times New Roman', serif",
		arabic: "'Amiri', 'Traditional Arabic', serif",
		hebrew: "'Frank Ruhl Libre', 'Noto Serif Hebrew', serif",
		cjk: "'Noto Serif CJK SC', 'SimSun', serif",
		devanagari: "'Noto Serif Devanagari', serif",
		cyrillic: "'EB Garamond', Georgia, serif",
	},
	flow: {
		latin: 'Inter, -apple-system, sans-serif',
		arabic: "'Noto Sans Arabic', 'Segoe UI', sans-serif",
		hebrew: "'Noto Sans Hebrew', 'Segoe UI', sans-serif",
		cjk: "'Noto Sans CJK SC', sans-serif",
		devanagari: "'Noto Sans Devanagari', sans-serif",
		cyrillic: 'Inter, -apple-system, sans-serif',
	},
};

/** Get the focus configuration */
export function getFocusConfig(type: FocusType): FocusConfig {
	switch (type) {
		case 'blankPage':
			return {
				className: 'focus-blank-page',
				hideToolbar: true,
				hideBreadcrumb: true,
				hideProperties: true,
				hideSidebar: true,
				hideTabBar: true,
				hideStatusBar: true,
				typewriterScroll: false,
				maxWidth: '720px',
				fonts: FOCUS_FONTS.blankPage,
				lineHeight: '2',
				padding: '80px 40px',
				background: 'var(--background-primary)',
			};
		case 'typewriter':
			return {
				className: 'focus-typewriter',
				hideToolbar: true,
				hideBreadcrumb: true,
				hideProperties: false,
				hideSidebar: true,
				hideTabBar: true,
				hideStatusBar: false,
				typewriterScroll: true,
				maxWidth: '680px',
				fonts: FOCUS_FONTS.typewriter,
				lineHeight: '1.8',
				padding: '60px 40px',
				background: 'var(--background-primary)',
			};
		case 'manuscript':
			return {
				className: 'focus-manuscript',
				hideToolbar: true,
				hideBreadcrumb: true,
				hideProperties: false,
				hideSidebar: true,
				hideTabBar: true,
				hideStatusBar: false,
				typewriterScroll: false,
				maxWidth: '560px',
				fonts: FOCUS_FONTS.manuscript,
				lineHeight: '2.2',
				padding: '80px 60px',
				background: 'var(--background-primary)',
			};
		case 'flow':
			return {
				className: 'focus-flow',
				hideToolbar: true,
				hideBreadcrumb: true,
				hideProperties: true,
				hideSidebar: true,
				hideTabBar: true,
				hideStatusBar: true,
				typewriterScroll: false,
				maxWidth: '100%',
				fonts: FOCUS_FONTS.flow,
				lineHeight: '1.9',
				padding: '40px 60px',
				background: 'var(--background-primary)',
			};
		default:
			return {
				className: '',
				hideToolbar: false,
				hideBreadcrumb: false,
				hideProperties: false,
				hideSidebar: false,
				hideTabBar: false,
				hideStatusBar: false,
				typewriterScroll: false,
				maxWidth: '100%',
				fonts: {},
				lineHeight: '1.7',
				padding: '0',
				background: 'var(--background-primary)',
			};
	}
}

/** Get the font for a specific focus and script */
export function getFocusFont(type: FocusType, script: string): string {
	return FOCUS_FONTS[type]?.[script] || '';
}
