import type { PageLoad } from './$types';

export type ExtensionKind =
	| 'skill_pack'
	| 'lua_plugin'
	| 'wasm_plugin'
	| 'mcp_preset'
	| 'recipe_ref';

export type RegistryExtension = {
	id: string;
	name: string;
	description: string;
	kind: ExtensionKind;
	source: string;
	install_hint: string;
	verified?: boolean;
	recipe_id?: string;
};

export const load: PageLoad = async ({ fetch }) => {
	const res = await fetch('/registry/extensions.json');
	if (!res.ok) {
		return { schema_version: 0, extensions: [] as RegistryExtension[] };
	}
	const data = (await res.json()) as {
		schema_version?: number;
		extensions?: RegistryExtension[];
	};
	return {
		schema_version: data.schema_version ?? 0,
		extensions: data.extensions ?? []
	};
};
