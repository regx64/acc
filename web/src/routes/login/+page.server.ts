import { redirect } from '@sveltejs/kit';
import { api, failWith } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

function safeNext(next: string | null): string {
	return next && next.startsWith('/') && !next.startsWith('//') ? next : '/';
}

export const load: PageServerLoad = async ({ locals, url }) => {
	if (locals.user) redirect(303, safeNext(url.searchParams.get('next')));
};

export const actions: Actions = {
	default: async (event) => {
		const f = await event.request.formData();
		const login = String(f.get('login') ?? '');
		const r = await api(event).POST('/auth/login', { body: { login, password: String(f.get('password') ?? '') } });
		if (!r.response.ok) return failWith(r, { login });
		redirect(303, safeNext(event.url.searchParams.get('next')));
	}
};
