import type { PageLoad } from './$types';

export type RegistryPlugin = {
	id: string;
	name: string;
	repo: string;
	description: string;
};

export const load: PageLoad = async ({ fetch }) => {
	const res = await fetch('/registry/plugins.json');
	if (!res.ok) {
		return { schema_version: 0, plugins: [] as RegistryPlugin[] };
	}
	const data = (await res.json()) as { schema_version?: number; plugins?: RegistryPlugin[] };
	return {
		schema_version: data.schema_version ?? 0,
		plugins: data.plugins ?? []
	};
};
