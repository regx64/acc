<script lang="ts">
	import Badge from '$lib/components/Badge.svelte';
	import ProblemState from '$lib/components/ProblemState.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import { date, num } from '$lib/format';
	let { data } = $props();
	const filters = [
		{ v: undefined, label: '전체' },
		{ v: 'REVIEW', label: '검수 중' },
		{ v: 'DRAFT', label: '초안' },
		{ v: 'PUBLIC', label: '공개' }
	];
</script>

<svelte:head><title>문제 관리 · acc</title></svelte:head>

<div class="mb-4 flex flex-wrap items-center justify-between gap-3">
	<div class="flex gap-1.5">
		{#each filters as f (f.label)}
			<a class="btn py-1 text-sm {data.status === f.v ? 'border-accent text-accent' : ''}" href={f.v ? `?status=${f.v}` : '?'}>{f.label}</a>
		{/each}
	</div>
	<a class="btn btn-primary" href="/create">공식 문제 만들기</a>
</div>
<div class="table-wrap">
	<table class="table">
		<thead><tr><th>번호</th><th>레벨</th><th class="w-full">제목</th><th>출처</th><th>출제자</th><th>상태</th><th class="text-right">테스트</th><th class="text-right">맞힌 사람</th><th>만든 날</th></tr></thead>
		<tbody>
			{#each data.rows as p (p.id)}
				<tr>
					<td class="font-mono"><a class="link" href="/my/problems/{p.id}">{p.id}</a></td>
					<td><Badge level={p.status === 'PUBLIC' ? p.level : (p.proposed_level ?? 0)} /></td>
					<td class="wrap"><a class="text-fg no-underline hover:underline" href="/my/problems/{p.id}">{p.title}</a></td>
					<td class="text-xs">{p.source === 'OFFICIAL' ? '공식' : '사용자'}</td>
					<td><UserLink handle={p.author} /></td>
					<td><ProblemState status={p.status} validation={p.validation_status} /></td>
					<td class="text-right tabular">{num(p.testcase_count)}</td>
					<td class="text-right tabular">{num(p.solved_count)}</td>
					<td class="tabular text-muted">{date(p.created_at)}</td>
				</tr>
			{:else}
				<tr><td colspan="9" class="py-10 text-center text-muted">문제가 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
