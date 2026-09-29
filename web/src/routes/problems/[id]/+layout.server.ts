import { api, must } from '$lib/server/api';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	const id = Number(event.params.id);
	const problem = must(await api(event).GET('/problems/{id}', { params: { path: { id } } }));
	return { problem };
};
