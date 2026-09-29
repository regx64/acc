import { fail, redirect } from '@sveltejs/kit';
import { api, errorMessage, failWith, must } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const id = Number(event.params.id);
	return { post: must(await api(event).GET('/posts/{id}', { params: { path: { id } } })) };
};

export const actions: Actions = {
	comment: async (event) => {
		const f = await event.request.formData();
		const body = {
			body: String(f.get('body') ?? ''),
			code: String(f.get('code') ?? '') || null,
			code_language: String(f.get('code_language') ?? '') || null
		};
		const r = await api(event).POST('/posts/{id}/comments', { params: { path: { id: Number(event.params.id) } }, body });
		if (!r.response.ok) return failWith(r, body);
		return { ok: '댓글을 달았습니다.' };
	},
	report: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).POST('/reports', {
			body: {
				target_type: String(f.get('target_type')),
				target_id: Number(f.get('target_id')),
				reason: String(f.get('reason') ?? '').trim() || '부적절한 내용'
			}
		});
		if (!r.response.ok) return fail(r.response.status, { message: errorMessage(r.error) });
		return { ok: '신고했습니다.' };
	},
	deletePost: async (event) => {
		const client = api(event);
		const id = Number(event.params.id);
		const post = await client.GET('/posts/{id}', { params: { path: { id } } });
		const r = await client.DELETE('/posts/{id}', { params: { path: { id } } });
		if (!r.response.ok) return failWith(r);
		redirect(303, post.data ? `/problems/${post.data.problem_id}/board` : '/');
	},
	deleteComment: async (event) => {
		const f = await event.request.formData();
		const r = await api(event).DELETE('/comments/{id}', { params: { path: { id: Number(f.get('id')) } } });
		if (!r.response.ok) return failWith(r);
		return { ok: '댓글을 지웠습니다.' };
	},
	visibility: async (event) => {
		const f = await event.request.formData();
		const hidden = f.get('hidden') === 'true';
		const id = Number(f.get('id'));
		const client = api(event);
		const r =
			f.get('target') === 'comment'
				? await client.POST('/admin/comments/{id}/visibility', { params: { path: { id } }, body: { hidden } })
				: await client.POST('/admin/posts/{id}/visibility', { params: { path: { id } }, body: { hidden } });
		if (!r.response.ok) return failWith(r);
		return { ok: hidden ? '숨겼습니다.' : '다시 보이게 했습니다.' };
	},
	resolve: async (event) => {
		const r = await api(event).POST('/admin/posts/{id}/resolve', { params: { path: { id: Number(event.params.id) } } });
		if (!r.response.ok) return failWith(r);
		return { ok: '처리 완료로 표시했습니다.' };
	}
};
