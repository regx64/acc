import { redirect } from '@sveltejs/kit';
import { api, failWith, must } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	if (!event.locals.user) redirect(303, `/login?next=${encodeURIComponent(event.url.pathname)}`);
	const id = Number(event.params.id);
	return { s: must(await api(event).GET('/submissions/{id}', { params: { path: { id } } })) };
};

export const actions: Actions = {
	rejudge: async (event) => {
		const r = await api(event).POST('/admin/submissions/{id}/rejudge', { params: { path: { id: Number(event.params.id) } } });
		if (!r.response.ok) return failWith(r);
		return { ok: '재채점을 요청했습니다.' };
	}
};
