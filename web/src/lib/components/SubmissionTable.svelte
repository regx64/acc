<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import UserLink from './UserLink.svelte';
	import Verdict from './Verdict.svelte';
	import { anyPending, dateTime, LANGUAGE_LABEL, num } from '$lib/format';
	import type { components } from '$lib/api/schema';

	let { rows, showProblem = true }: { rows: components['schemas']['SubmissionRow'][]; showProblem?: boolean } = $props();

	// Refresh while anything on screen is still waiting or being judged.
	$effect(() => {
		if (!anyPending(rows)) return;
		const t = setTimeout(() => invalidateAll(), 1500);
		return () => clearTimeout(t);
	});
</script>

<div class="table-wrap">
	<table class="table">
		<thead>
			<tr>
				<th>제출 번호</th>
				<th>아이디</th>
				{#if showProblem}<th>문제</th>{/if}
				<th>결과</th>
				<th class="text-right">메모리</th>
				<th class="text-right">시간</th>
				<th>언어</th>
				<th class="text-right">코드 길이</th>
				<th>제출한 시간</th>
			</tr>
		</thead>
		<tbody>
			{#each rows as s (s.id)}
				<tr>
					<td class="font-mono tabular"><a class="link" href="/submissions/{s.id}">{s.id}</a></td>
					<td><UserLink handle={s.handle} /></td>
					{#if showProblem}
						<td class="max-w-56 truncate"><a class="link font-mono" href="/problems/{s.problem_id}" title={s.problem_title}>{s.problem_id}</a></td>
					{/if}
					<td><Verdict status={s.status} /></td>
					<td class="text-right tabular">{s.memory_kb != null && s.status !== 'CE' ? `${num(s.memory_kb)} KB` : ''}</td>
					<td class="text-right tabular">{s.time_ms != null && s.status !== 'CE' ? `${num(s.time_ms)} ms` : ''}</td>
					<td>{LANGUAGE_LABEL[s.language] ?? s.language}</td>
					<td class="text-right tabular">{num(s.code_length)} B</td>
					<td class="text-muted tabular">{dateTime(s.created_at)}</td>
				</tr>
			{:else}
				<tr><td colspan="9" class="py-10 text-center text-muted">제출이 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
