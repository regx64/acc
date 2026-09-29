import { loadStatus } from '$lib/server/status';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = (event) => loadStatus(event, Number(event.params.id));
