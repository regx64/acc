<script lang="ts">
	import Badge from '$lib/components/Badge.svelte';
	import ProblemState from '$lib/components/ProblemState.svelte';
	import { date } from '$lib/format';
	let { data } = $props();
</script>

<svelte:head><title>내 문제 · acc</title></svelte:head>

<div class="mb-5 flex items-center justify-between">
	<h1 class="text-2xl font-semibold">내 문제</h1>
	<a class="btn btn-primary" href="/create">새 문제</a>
</div>
<div class="table-wrap">
	<table class="table">
		<thead><tr><th>번호</th><th>레벨</th><th class="w-full">제목</th><th>상태</th><th>최근 검수</th><th>만든 날</th></tr></thead>
		<tbody>
			{#each data.problems as p (p.id)}
				<tr>
					<td class="font-mono"><a class="link" href="/my/problems/{p.id}">{p.id}</a></td>
					<td><Badge level={p.status === 'PUBLIC' ? p.level : (p.proposed_level ?? 0)} /></td>
					<td class="wrap"><a class="text-fg no-underline hover:underline" href="/my/problems/{p.id}">{p.title}</a></td>
					<td><ProblemState status={p.status} validation={p.validation_status} /></td>
					<td class="wrap max-w-64 text-sm text-muted">{p.last_review ?? ''}</td>
					<td class="tabular text-muted">{date(p.created_at)}</td>
				</tr>
			{:else}
				<tr><td colspan="6" class="py-10 text-center text-muted">아직 만든 문제가 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
