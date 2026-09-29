// Server-side API client. The browser only talks to SvelteKit; SvelteKit
// calls the Rust API, forwarding the session cookie and the client's IP.

import { env } from '$env/dynamic/private';
import { dev } from '$app/environment';
import { error, fail, type Cookies, type RequestEvent } from '@sveltejs/kit';
import createClient from 'openapi-fetch';
import type { paths } from '$lib/api/schema';

export const SESSION_COOKIE = 'acc_session';

function apiBase() {
	return (env.API_URL ?? 'http://localhost:8080').replace(/\/$/, '') + '/api/v1';
}

/** Copies the API's session cookie onto the SvelteKit response. */
function relayCookies(res: Response, cookies: Cookies) {
	for (const raw of res.headers.getSetCookie()) {
		const [pair, ...attrs] = raw.split(';');
		const eq = pair.indexOf('=');
		const name = pair.slice(0, eq).trim();
		const value = pair.slice(eq + 1).trim();
		if (name !== SESSION_COOKIE) continue;
		const maxAgeAttr = attrs.map((a) => a.trim()).find((a) => a.toLowerCase().startsWith('max-age='));
		const maxAge = maxAgeAttr ? Number(maxAgeAttr.split('=')[1]) : undefined;
		if (!value || maxAge === 0) {
			cookies.delete(SESSION_COOKIE, { path: '/' });
		} else {
			cookies.set(SESSION_COOKIE, value, {
				path: '/',
				httpOnly: true,
				sameSite: 'lax',
				secure: !dev,
				maxAge
			});
		}
	}
}

export function api(event: Pick<RequestEvent, 'cookies' | 'getClientAddress' | 'fetch' | 'request'>) {
	const session = event.cookies.get(SESSION_COOKIE);
	let ip = '';
	try {
		ip = event.request.headers.get('cf-connecting-ip') ?? event.getClientAddress();
	} catch {
		ip = '';
	}
	const headers: Record<string, string> = {};
	if (session) headers.cookie = `${SESSION_COOKIE}=${session}`;
	if (env.INTERNAL_TOKEN) {
		headers['x-acc-internal'] = env.INTERNAL_TOKEN;
		if (ip) headers['x-acc-client-ip'] = ip;
	}
	const client = createClient<paths>({
		baseUrl: apiBase(),
		headers,
		fetch: async (req: Request) => {
			const res = await fetch(req);
			relayCookies(res, event.cookies);
			return res;
		}
	});
	return client;
}

export type ApiErrorBody = { error?: { code?: string; message?: string; location?: string } };

export function errorMessage(e: unknown, fallback = '요청을 처리하지 못했습니다.'): string {
	const body = e as ApiErrorBody | undefined;
	return body?.error?.message ?? fallback;
}

/** For load functions: data or a SvelteKit error page. */
export function must<T>(r: { data?: T; error?: unknown; response: Response }): T {
	if (r.data !== undefined && !r.error) return r.data as T;
	const body = r.error as ApiErrorBody | undefined;
	error(r.response.status || 500, {
		message: body?.error?.message ?? '요청을 처리하지 못했습니다.',
		code: body?.error?.code
	});
}

/** For form actions: the API error as a `fail` result. */
export function failWith<T extends Record<string, unknown> = Record<never, never>>(
	r: { error?: unknown; response: Response },
	extra?: T
) {
	return fail(r.response.status || 500, { message: errorMessage(r.error), ...(extra ?? ({} as T)) });
}

/** Upload helper for multipart bodies (openapi-fetch sends JSON by default). */
export async function upload(
	event: Parameters<typeof api>[0],
	path: string,
	form: FormData,
	method = 'PUT'
): Promise<{ ok: boolean; status: number; message?: string }> {
	const session = event.cookies.get(SESSION_COOKIE);
	const headers: Record<string, string> = {};
	if (session) headers.cookie = `${SESSION_COOKIE}=${session}`;
	if (env.INTERNAL_TOKEN) headers['x-acc-internal'] = env.INTERNAL_TOKEN;
	const res = await fetch(apiBase() + path, { method, body: form, headers });
	if (res.ok) return { ok: true, status: res.status };
	const body = (await res.json().catch(() => ({}))) as ApiErrorBody;
	return { ok: false, status: res.status, message: body.error?.message ?? '업로드하지 못했습니다.' };
}
