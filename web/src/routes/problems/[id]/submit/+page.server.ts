import { redirect } from '@sveltejs/kit';
import { api, failWith } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, url }) => {
	if (!locals.user) redirect(303, `/login?next=${encodeURIComponent(url.pathname)}`);
	return { defaultLanguage: locals.user.default_language ?? 'cpp17' };
};

export const actions: Actions = {
	default: async (event) => {
		const f = await event.request.formData();
		const language = String(f.get('language') ?? '');
		const code = String(f.get('code') ?? '');
		const id = Number(event.params.id);
		const r = await api(event).POST('/problems/{id}/submit', { params: { path: { id } }, body: { language, code } });
		if (!r.response.ok) return failWith(r, { language, code });
		const handle = event.locals.user?.handle ?? '';
		redirect(303, `/problems/${id}/status?handle=${encodeURIComponent(handle)}`);
	}
};
