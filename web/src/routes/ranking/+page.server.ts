import { api, must } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const cursor = event.url.searchParams.get('cursor') ?? undefined;
	return { page: must(await api(event).GET('/ranking', { params: { query: { cursor } } })) };
};
