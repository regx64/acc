import { api, failWith, must } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	return { o: must(await api(event).GET('/admin/overview')) };
};

export const actions: Actions = {
	banner: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).PUT('/admin/banner', { body: { text: String(f.get('text') ?? '') } });
		if (!r.response.ok) return failWith(r);
		return { ok: '공지를 저장했습니다. 30초 안에 반영됩니다.' };
	}
};
