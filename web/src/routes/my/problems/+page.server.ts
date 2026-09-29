import { redirect } from '@sveltejs/kit';
import { api, must } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	if (!event.locals.user) redirect(303, '/login?next=/my/problems');
	return { problems: must(await api(event).GET('/my/problems')) };
};
