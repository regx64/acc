import type { components } from '$lib/api/schema';

type Input = components['schemas']['ProblemInput'];

/** Reads ProblemForm fields into the API body. */
export function readProblemForm(f: FormData): Input {
	let statement: Input['statement'];
	try {
		statement = JSON.parse(String(f.get('statement') ?? '{}'));
	} catch {
		statement = { legend: '', input: '', output: '', samples: [] };
	}
	const code = String(f.get('solution_code') ?? '');
	return {
		title: String(f.get('title') ?? ''),
		time_limit_ms: Number(f.get('time_limit_ms') ?? 1000),
		memory_limit_mb: Number(f.get('memory_limit_mb') ?? 256),
		proposed_level: f.get('proposed_level') === null || f.get('proposed_level') === '' ? null : Number(f.get('proposed_level')),
		statement,
		solution_language: String(f.get('solution_language') ?? '') || null,
		solution_code: code.trim() ? code : null,
		agree_terms: f.get('agree') === 'on'
	};
}
