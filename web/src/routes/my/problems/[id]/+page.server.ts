import { fail, redirect } from '@sveltejs/kit';
import { api, errorMessage, failWith, must, upload } from '$lib/server/api';
import { readProblemForm } from '$lib/server/problemForm';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	if (!event.locals.user) redirect(303, `/login?next=${encodeURIComponent(event.url.pathname)}`);
	const id = Number(event.params.id);
	return { p: must(await api(event).GET('/my/problems/{id}', { params: { path: { id } } })) };
};

const path = (event: { params: { id: string } }) => ({ params: { path: { id: Number(event.params.id) } } });

export const actions: Actions = {
	save: async (event) => {
		const body = readProblemForm(await event.request.formData());
		const r = await api(event).PUT('/my/problems/{id}', { ...path(event), body });
		if (!r.response.ok) return failWith(r);
		return { ok: '저장했습니다.' };
	},
	upload: async (event) => {
		const f = await event.request.formData();
		const file = f.get('file');
		if (!(file instanceof File) || file.size === 0) return fail(400, { message: 'zip 파일을 고르세요.' });
		const form = new FormData();
		form.set('file', file, file.name);
		const r = await upload(event, `/my/problems/${event.params.id}/testcases`, form);
		if (!r.ok) return fail(r.status, { message: r.message });
		return { ok: '테스트 데이터를 올렸습니다. 정해 검증을 다시 실행하세요.' };
	},
	validate: async (event) => {
		const r = await api(event).POST('/my/problems/{id}/validate', path(event));
		if (!r.response.ok) return failWith(r);
		return { ok: '정해 검증을 시작했습니다.' };
	},
	review: async (event) => {
		const r = await api(event).POST('/my/problems/{id}/request-review', path(event));
		if (!r.response.ok) return failWith(r);
		return { ok: '검수를 요청했습니다.' };
	},
	withdraw: async (event) => {
		const r = await api(event).POST('/my/problems/{id}/withdraw', path(event));
		if (!r.response.ok) return failWith(r);
		return { ok: '검수 요청을 취소했습니다.' };
	},
	decide: async (event) => {
		const f = await event.request.formData();
		const approve = f.get('approve') === 'true';
		const level = f.get('level') ? Number(f.get('level')) : null;
		const comment = String(f.get('comment') ?? '') || null;
		const r = await api(event).POST('/admin/problems/{id}/decision', { ...path(event), body: { approve, level, comment } });
		if (!r.response.ok) return failWith(r);
		return { ok: approve ? '공개했습니다.' : '반려했습니다.' };
	},
	patch: async (event) => {
		const f = await event.request.formData();
		const level = f.get('level') ? Number(f.get('level')) : null;
		const status = String(f.get('status') ?? '') || null;
		const r = await api(event).PATCH('/admin/problems/{id}', { ...path(event), body: { level, status } });
		if (!r.response.ok) return failWith(r);
		return { ok: `저장했습니다. 레이팅 ${r.data!.users}명 재계산.` };
	},
	rejudge: async (event) => {
		const r = await api(event).POST('/admin/problems/{id}/rejudge', path(event));
		if (!r.response.ok) return fail(r.response.status, { message: errorMessage(r.error) });
		return { ok: `제출 ${r.data!.submissions}개를 재채점합니다.` };
	}
};
