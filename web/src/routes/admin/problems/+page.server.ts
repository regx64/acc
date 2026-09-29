import { api, must } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const status = event.url.searchParams.get('status') ?? undefined;
	return { rows: must(await api(event).GET('/admin/problems', { params: { query: { status } } })), status };
};
