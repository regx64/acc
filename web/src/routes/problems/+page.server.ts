import { api, must } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const q = event.url.searchParams;
	const query = {
		sort: q.get('sort') ?? undefined,
		order: q.get('order') ?? undefined,
		group: q.get('group') ?? undefined,
		q: q.get('q') ?? undefined,
		cursor: q.get('cursor') ?? undefined
	};
	const page = must(await api(event).GET('/problems', { params: { query } }));
	return { page, query };
};
