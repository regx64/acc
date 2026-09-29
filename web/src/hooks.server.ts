import type { Handle } from '@sveltejs/kit';
import { api, SESSION_COOKIE } from '$lib/server/api';
import type { components } from '$lib/api/schema';

type SiteInfo = components['schemas']['SiteInfo'];

const FALLBACK_SITE: SiteInfo = { banner: null, languages: [], groups: [], tier_coefficient: 90 };
let siteCache: { at: number; value: SiteInfo } | null = null;

async function siteInfo(client: ReturnType<typeof api>): Promise<SiteInfo> {
	if (siteCache && Date.now() - siteCache.at < 30_000) return siteCache.value;
	try {
		const { data } = await client.GET('/site');
		if (data) {
			siteCache = { at: Date.now(), value: data };
			return data;
		}
	} catch {
		// API down: render with defaults rather than failing every page.
	}
	return siteCache?.value ?? FALLBACK_SITE;
}

export const handle: Handle = async ({ event, resolve }) => {
	const client = api(event);
	event.locals.user = null;
	if (event.cookies.get(SESSION_COOKIE)) {
		try {
			const { data } = await client.GET('/me');
			event.locals.user = data ?? null;
		} catch {
			event.locals.user = null;
		}
	}
	event.locals.site = await siteInfo(client);
	return resolve(event);
};
