import type { components } from '$lib/api/schema';

declare global {
	namespace App {
		interface Error {
			message: string;
			code?: string;
		}
		interface Locals {
			user: components['schemas']['Me'] | null;
			site: components['schemas']['SiteInfo'];
		}
		interface Platform {
			env?: Record<string, string>;
		}
	}
}

export {};
