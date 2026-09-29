import { redirect } from '@sveltejs/kit';
import { api, failWith, must } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const id = Number(event.params.id);
	const cursor = event.url.searchParams.get('cursor') ?? undefined;
	return { posts: must(await api(event).GET('/problems/{id}/posts', { params: { path: { id }, query: { cursor } } })) };
};

export const actions: Actions = {
	default: async (event) => {
		const f = await event.request.formData();
		const body = {
			kind: String(f.get('kind') ?? 'QUESTION'),
			title: String(f.get('title') ?? ''),
			body: String(f.get('body') ?? ''),
			code: String(f.get('code') ?? '') || null,
			code_language: String(f.get('code_language') ?? '') || null
		};
		const r = await api(event).POST('/problems/{id}/posts', { params: { path: { id: Number(event.params.id) } }, body });
		if (!r.response.ok) return failWith(r, body);
		redirect(303, `/board/${r.data!.id}`);
	}
};
