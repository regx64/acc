import { error, redirect } from '@sveltejs/kit';
import { api, type ApiErrorBody } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const r = await api(event).GET('/users/{handle}', { params: { path: { handle: event.params.handle } } });
	if (r.data) return { profile: r.data };
	const body = r.error as ApiErrorBody;
	if (body?.error?.code === 'HANDLE_MOVED' && body.error.location) {
		redirect(301, `/u/${body.error.location}`);
	}
	error(r.response.status, { message: body?.error?.message ?? '사용자를 찾을 수 없습니다.' });
};
