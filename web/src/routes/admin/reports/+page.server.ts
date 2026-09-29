import { api, failWith, must } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => ({ rows: must(await api(event).GET('/admin/reports')) });

export const actions: Actions = {
	resolve: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).POST('/admin/reports/{id}/resolve', {
			params: { path: { id: Number(f.get('id')) } },
			body: { hide: f.get('hide') === 'true' }
		});
		if (!r.response.ok) return failWith(r);
		return { ok: '처리했습니다.' };
	}
};
