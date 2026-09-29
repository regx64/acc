import { fail, redirect } from '@sveltejs/kit';
import { api, errorMessage, SESSION_COOKIE } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) redirect(303, '/login?next=/settings');
};

export const actions: Actions = {
	language: async (event) => {
		const f = await event.request.formData();
		const v = String(f.get('default_language') ?? '');
		const r = await api(event).PATCH('/me/settings', { body: { default_language: v || null } });
		if (!r.response.ok) return fail(r.response.status, { section: 'language', message: errorMessage(r.error) });
		return { section: 'language', ok: '저장했습니다.' };
	},
	password: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).POST('/me/password', {
			body: { current_password: String(f.get('current') ?? ''), new_password: String(f.get('new') ?? '') }
		});
		if (!r.response.ok) return fail(r.response.status, { section: 'password', message: errorMessage(r.error) });
		return { section: 'password', ok: '비밀번호를 바꿨습니다. 다른 기기에서는 로그아웃됩니다.' };
	},
	handle: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).POST('/me/handle', { body: { handle: String(f.get('handle') ?? '').trim() } });
		if (!r.response.ok) return fail(r.response.status, { section: 'handle', message: errorMessage(r.error) });
		return { section: 'handle', ok: '핸들을 바꿨습니다.' };
	},
	delete: async (event) => {
		const f = await event.request.formData();
		if (f.get('confirm') !== '탈퇴') return fail(400, { section: 'delete', message: '확인란에 "탈퇴"를 입력하세요.' });
		const r = await api(event).POST('/me/delete', { body: { password: String(f.get('password') ?? '') } });
		if (!r.response.ok) return fail(r.response.status, { section: 'delete', message: errorMessage(r.error) });
		event.cookies.delete(SESSION_COOKIE, { path: '/' });
		redirect(303, '/?deleted=1');
	}
};
