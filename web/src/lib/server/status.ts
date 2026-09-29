import type { RequestEvent } from '@sveltejs/kit';
import { api, must } from './api';

/** Shared loader for the global and per-problem status boards. */
export async function loadStatus(event: RequestEvent, problemId?: number) {
	const q = event.url.searchParams;
	const pick = (k: string) => q.get(k)?.trim() || undefined;
	const query = {
		problem_id: problemId ?? (pick('problem_id') ? Number(pick('problem_id')) : undefined),
		handle: pick('handle'),
		status: pick('status'),
		language: pick('language'),
		cursor: pick('cursor')
	};
	const page = must(await api(event).GET('/submissions', { params: { query } }));
	return {
		page,
		query: {
			problem_id: query.problem_id?.toString(),
			handle: query.handle,
			status: query.status,
			language: query.language
		}
	};
}
