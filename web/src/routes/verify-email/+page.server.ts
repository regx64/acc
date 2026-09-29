import { fail } from '@sveltejs/kit';
import { api, errorMessage } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const token = event.url.searchParams.get('token');
	if (!token) return { verified: false, message: null };
	const r = await api(event).POST('/auth/verify-email', { body: { token } });
	return r.error ? { verified: false, message: errorMessage(r.error) } : { verified: true, message: null };
};

export const actions: Actions = {
	resend: async (event) => {
		const r = await api(event).POST('/auth/resend-verification');
		if (!r.response.ok) return fail(r.response.status, { message: errorMessage(r.error) });
		return { ok: '인증 메일을 다시 보냈습니다.' };
	}
};
