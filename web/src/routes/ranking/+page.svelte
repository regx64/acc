<script lang="ts">
	import Badge from '$lib/components/Badge.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import { num } from '$lib/format';
	let { data } = $props();
</script>

<svelte:head><title>랭킹 · acc</title></svelte:head>

<h1 class="mb-2 text-2xl font-semibold">랭킹</h1>
<p class="mb-5 text-sm text-muted">레이팅은 맞힌 문제 중 레벨이 높은 100문제의 (레벨)² 합. 레벨 L 문제 90개를 풀면 티어 L에 닿습니다.</p>
<div class="table-wrap">
	<table class="table">
		<thead><tr><th class="text-right">순위</th><th>티어</th><th class="w-full">아이디</th><th class="text-right">레이팅</th><th class="text-right">맞힌 문제</th></tr></thead>
		<tbody>
			{#each data.page.items as r (r.id)}
				<tr>
					<td class="text-right font-mono tabular">{r.rank}</td>
					<td><Badge level={r.tier} /></td>
					<td><UserLink handle={r.handle} /></td>
					<td class="text-right font-mono tabular">{num(r.rating)}</td>
					<td class="text-right tabular">{num(r.solved_count)}</td>
				</tr>
			{:else}
				<tr><td colspan="5" class="py-10 text-center text-muted">아직 순위가 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<Pager next={data.page.next_cursor} />
