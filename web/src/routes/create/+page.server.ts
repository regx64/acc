import { redirect } from '@sveltejs/kit';
import { api, failWith } from '$lib/server/api';
import { readProblemForm } from '$lib/server/problemForm';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) redirect(303, '/login?next=/create');
};

export const actions: Actions = {
	default: async (event) => {
		const body = readProblemForm(await event.request.formData());
		const r = await api(event).POST('/my/problems', { body });
		if (!r.response.ok) return failWith(r, { draft: body });
		redirect(303, `/my/problems/${r.data!.id}`);
	}
};
