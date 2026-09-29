import { api, failWith, must } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const q = event.url.searchParams.get('q') ?? '';
	return { rows: must(await api(event).GET('/admin/users', { params: { query: { q } } })), q };
};

export const actions: Actions = {
	suspend: async (event) => {
		const f = await event.request.formData();
		const id = Number(f.get('id'));
		const r = await api(event).POST('/admin/users/{id}/suspend', { params: { path: { id } }, body: { reason: String(f.get('reason') ?? '') } });
		if (!r.response.ok) return failWith(r);
		return { ok: '정지했습니다.' };
	},
	unsuspend: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).POST('/admin/users/{id}/unsuspend', { params: { path: { id: Number(f.get('id')) } } });
		if (!r.response.ok) return failWith(r);
		return { ok: '정지를 풀었습니다.' };
	}
};
