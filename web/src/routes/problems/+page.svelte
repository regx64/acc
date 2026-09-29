<script lang="ts">
	import { page } from '$app/state';
	import Badge from '$lib/components/Badge.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import { num } from '$lib/format';
	import { GROUPS } from '$lib/levels';

	let { data } = $props();

	function href(changes: Record<string, string | null>) {
		const u = new URL(page.url);
		u.searchParams.delete('cursor');
		for (const [k, v] of Object.entries(changes)) {
			if (v === null) u.searchParams.delete(k);
			else u.searchParams.set(k, v);
		}
		return u.pathname + u.search;
	}
	function sortHref(key: string) {
		const cur = data.query.sort ?? 'id';
		const order = cur === key && (data.query.order ?? 'asc') === 'asc' ? 'desc' : key === 'solved' ? 'desc' : 'asc';
		return href({ sort: key, order });
	}
	function arrow(key: string) {
		if ((data.query.sort ?? 'id') !== key) return '';
		return (data.query.order ?? 'asc') === 'asc' ? ' ▲' : ' ▼';
	}
</script>

<svelte:head><title>문제 · acc</title></svelte:head>

<div class="mb-5 flex flex-wrap items-end justify-between gap-4">
	<h1 class="text-2xl font-semibold">문제</h1>
	<form method="GET" class="flex gap-2">
		{#if data.query.group}<input type="hidden" name="group" value={data.query.group} />{/if}
		<input class="input w-56" name="q" placeholder="제목 또는 번호" value={data.query.q ?? ''} aria-label="검색" />
		<button class="btn">검색</button>
	</form>
</div>

<div class="mb-4 flex flex-wrap gap-1.5 text-sm">
	<a href={href({ group: null })} class="btn py-1 {data.query.group ? '' : 'border-accent text-accent'}">전체</a>
	{#each GROUPS as g, i (g.key)}
		<a href={href({ group: g.key })} class="btn py-1 {data.query.group === g.key ? 'border-accent text-accent' : ''}">
			<Badge level={i * 5 + 1} size="xs" title={false} />
			{g.name}
		</a>
	{/each}
	<a href={href({ group: 'unrated' })} class="btn py-1 {data.query.group === 'unrated' ? 'border-accent text-accent' : ''}">
		<Badge level={0} size="xs" title={false} /> Unrated
	</a>
</div>

<div class="table-wrap">
	<table class="table">
		<thead>
			<tr>
				<th><a class="text-muted no-underline" href={sortHref('id')}>번호{arrow('id')}</a></th>
				<th><a class="text-muted no-underline" href={sortHref('level')}>레벨{arrow('level')}</a></th>
				<th class="w-full">제목</th>
				<th class="text-right"><a class="text-muted no-underline" href={sortHref('solved')}>맞힌 사람{arrow('solved')}</a></th>
				<th class="text-right">제출</th>
			</tr>
		</thead>
		<tbody>
			{#each data.page.items as p (p.id)}
				<tr>
					<td class="font-mono tabular"><a class="link" href="/problems/{p.id}">{p.id}</a></td>
					<td><Badge level={p.level} /></td>
					<td class="wrap">
						<a class="text-fg no-underline hover:underline" href="/problems/{p.id}">{p.title}</a>
						{#if p.my_state === 'SOLVED'}<span class="ml-2 text-xs font-medium text-ok">맞음</span>
						{:else if p.my_state === 'TRIED'}<span class="ml-2 text-xs font-medium text-bad">시도</span>{/if}
						{#if p.source === 'USER'}<span class="ml-2 text-xs text-faint">사용자 출제</span>{/if}
					</td>
					<td class="text-right tabular">{num(p.solved_count)}</td>
					<td class="text-right tabular text-muted">{num(p.submission_count)}</td>
				</tr>
			{:else}
				<tr><td colspan="5" class="py-10 text-center text-muted">조건에 맞는 문제가 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<Pager next={data.page.next_cursor} />
