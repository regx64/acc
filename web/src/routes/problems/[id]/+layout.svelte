<script lang="ts">
	import { page } from '$app/state';
	import Badge from '$lib/components/Badge.svelte';
	import { longLabel } from '$lib/levels';

	let { data, children } = $props();
	const p = $derived(data.problem);
	const tabs = $derived([
		{ href: `/problems/${p.id}`, label: '문제' },
		{ href: `/problems/${p.id}/submit`, label: '제출' },
		{ href: `/problems/${p.id}/status`, label: '채점 현황' },
		{ href: `/problems/${p.id}/board`, label: '게시판' }
	]);
</script>

<div class="mb-6">
	{#if p.status !== 'PUBLIC'}
		<p class="mb-3 rounded-md bg-warn/15 px-3 py-2 text-sm">
			공개되지 않은 문제입니다({p.status === 'REVIEW' ? '검수 중' : '초안'}). 출제자와 관리자만 볼 수 있습니다.
		</p>
	{/if}
	<div class="flex flex-wrap items-center gap-3">
		<Badge level={p.level} size="md" />
		<h1 class="text-2xl font-semibold"><span class="mr-2 font-mono text-faint">{p.id}</span>{p.title}</h1>
	</div>
	<p class="mt-1 text-sm text-muted">{longLabel(p.level)}{#if p.source === 'USER' && p.author} · 출제 <a class="link" href="/u/{p.author}">{p.author}</a>{/if}</p>
	<nav class="mt-4 flex gap-1 overflow-x-auto border-b border-line" aria-label="문제 메뉴">
		{#each tabs as t (t.href)}
			<a
				href={t.href}
				class="-mb-px border-b-2 px-3 py-2 text-sm whitespace-nowrap no-underline {page.url.pathname === t.href ? 'border-accent text-fg' : 'border-transparent text-muted hover:text-fg'}"
				>{t.label}</a
			>
		{/each}
	</nav>
</div>

{@render children()}
