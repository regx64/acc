import { redirect } from '@sveltejs/kit';
import { api, failWith } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
	if (locals.user) redirect(303, '/');
};

export const actions: Actions = {
	default: async (event) => {
		const f = await event.request.formData();
		const handle = String(f.get('handle') ?? '').trim();
		const email = String(f.get('email') ?? '').trim();
		const r = await api(event).POST('/auth/signup', {
			body: {
				handle,
				email,
				password: String(f.get('password') ?? ''),
				agree_terms: f.get('agree') === 'on'
			}
		});
		if (!r.response.ok) return failWith(r, { handle, email });
		redirect(303, '/signup/done');
	}
};
