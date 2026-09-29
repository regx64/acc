import { redirect } from '@sveltejs/kit';
import { api, failWith } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url }) => ({ token: url.searchParams.get('token') ?? '' });

export const actions: Actions = {
	default: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).POST('/auth/password-reset/confirm', {
			body: { token: String(f.get('token') ?? ''), password: String(f.get('password') ?? '') }
		});
		if (!r.response.ok) return failWith(r);
		redirect(303, '/login?reset=1');
	}
};
