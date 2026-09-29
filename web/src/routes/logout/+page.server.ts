import { redirect } from '@sveltejs/kit';
import { api, SESSION_COOKIE } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async () => redirect(303, '/');

export const actions: Actions = {
	default: async (event) => {
		await api(event).POST('/auth/logout');
		event.cookies.delete(SESSION_COOKIE, { path: '/' });
		redirect(303, '/');
	}
};
