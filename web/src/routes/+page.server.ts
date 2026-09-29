import { api } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const client = api(event);
	const [problems, ranking, status] = await Promise.all([
		client.GET('/problems', { params: { query: { sort: 'id', order: 'desc', limit: 10 } } }),
		client.GET('/ranking', { params: { query: { limit: 10 } } }),
		client.GET('/submissions', { params: { query: { limit: 8 } } })
	]);
	return {
		problems: problems.data?.items ?? [],
		ranking: ranking.data?.items ?? [],
		recent: status.data?.items ?? []
	};
};
