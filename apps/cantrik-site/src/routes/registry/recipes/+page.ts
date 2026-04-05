import type { PageLoad } from './$types';

export type RegistryRecipe = {
	id: string;
	title: string;
	description: string;
	init_template: string;
	/** Set only by maintainers after manual verification (see CONTRIBUTING § Registry recipes). */
	verified?: boolean;
};

export const load: PageLoad = async ({ fetch }) => {
	const res = await fetch('/registry/recipes.json');
	if (!res.ok) {
		return { schema_version: 0, recipes: [] as RegistryRecipe[] };
	}
	const data = (await res.json()) as {
		schema_version?: number;
		recipes?: RegistryRecipe[];
	};
	return {
		schema_version: data.schema_version ?? 0,
		recipes: data.recipes ?? []
	};
};
