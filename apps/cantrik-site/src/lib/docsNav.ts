export type DocNavEntry = {
	href: string;
	label: string;
	children?: DocNavEntry[];
};

/** Sidebar + mobile nav for /docs/* — pass each `href` through `resolve()` in Svelte. */
export const docsNavTree: DocNavEntry[] = [
	{ href: '/docs', label: 'Overview' },
	{ href: '/docs/start', label: 'Start (5 menit)' },
	{ href: '/docs/concepts', label: 'Konsep' },
	{ href: '/docs/install', label: 'Install' },
	{ href: '/docs/configure', label: 'Configure' },
	{
		href: '/docs/tools',
		label: 'Tools desktop & editor',
		children: [
			{ href: '/docs/tools/tray', label: 'cantrik-tray' },
			{ href: '/docs/tools/tauri', label: 'cantrik-tauri' },
			{ href: '/docs/tools/vscode', label: 'VS Code' }
		]
	},
	{
		href: '/docs/features',
		label: 'Fitur',
		children: [
			{ href: '/docs/features/semantic-index', label: 'Semantic index' },
			{ href: '/docs/features/session', label: 'Session memory' },
			{ href: '/docs/features/begawan', label: 'Begawan & agent' }
		]
	},
	{ href: '/docs/safety', label: 'Safety & approve' },
	{ href: '/docs/registry', label: 'Registry & recipes' },
	{ href: '/docs/agent-harness', label: 'Agent harness (CI)' }
];
