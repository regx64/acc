<script lang="ts">
	import Badge from '$lib/components/Badge.svelte';
	import { date, num } from '$lib/format';
	import { GROUPS, longLabel, tierThreshold } from '$lib/levels';

	let { data } = $props();
	const p = $derived(data.profile);
	const lo = $derived(tierThreshold(p.tier));
	const hi = $derived(p.next_tier_rating ?? lo);
	const progress = $derived(hi > lo ? Math.min(1, (p.rating - lo) / (hi - lo)) : 1);
	const maxCount = $derived(Math.max(1, ...p.level_counts));
	// Solved problems per group (levels 1–5, 6–10, …).
	const groupCounts = $derived(GROUPS.map((_, g) => p.level_counts.slice(g * 5 + 1, g * 5 + 6).reduce((a, b) => a + b, 0)));
</script>

<svelte:head><title>{p.handle} · acc</title></svelte:head>

<section class="card mb-6 flex flex-wrap items-center gap-6 p-6">
	<Badge level={p.tier} size="lg" />
	<div class="min-w-0 flex-1">
		<h1 class="text-3xl font-semibold">{p.handle}</h1>
		<p class="mt-1 text-muted">{p.tier > 0 ? longLabel(p.tier) : '티어 없음'} · {num(p.rank)}위 · {date(p.joined_at)} 가입</p>
		<div class="mt-4 max-w-md">
			<div class="flex justify-between text-sm">
				<span>레이팅 <b class="font-mono tabular">{num(p.rating)}</b></span>
				{#if p.next_tier_rating}<span class="text-muted tabular">다음 티어까지 {num(p.next_tier_rating - p.rating)}</span>{/if}
			</div>
			<div class="mt-1 h-2 overflow-hidden rounded-full bg-surface-2" role="progressbar" aria-valuenow={Math.round(progress * 100)} aria-valuemin="0" aria-valuemax="100">
				<div class="h-full rounded-full bg-accent" style="width: {progress * 100}%"></div>
			</div>
		</div>
	</div>
	<dl class="grid grid-cols-2 gap-x-8 gap-y-1 text-sm">
		<dt class="text-muted">맞힌 문제</dt><dd class="text-right font-mono tabular">{num(p.solved.length)}</dd>
		<dt class="text-muted">시도한 문제</dt><dd class="text-right font-mono tabular">{num(p.failed.length)}</dd>
		<dt class="text-muted">제출</dt><dd class="text-right font-mono tabular">{num(p.submission_count)}</dd>
		<dt class="text-muted"><a class="link" href="/status?handle={p.handle}">제출 보기</a></dt><dd></dd>
	</dl>
</section>

<section class="mb-8">
	<h2 class="mb-3 text-lg font-semibold">레벨 분포</h2>
	<div class="card p-5">
		<div class="flex h-36 items-end gap-[3px]" role="img" aria-label="레벨별 맞힌 문제 수">
			{#each p.level_counts as c, lv (lv)}
				<div class="flex h-full flex-1 flex-col justify-end" title="{lv === 0 ? 'Unrated' : longLabel(lv)}: {c}문제">
					<div
						class="rounded-t-sm"
						style="height: {(c / maxCount) * 100}%; min-height: {c > 0 ? 2 : 0}px; background: var(--t-{['u', 'n', 'z', 'q', 'r', 'c', 'h'][lv === 0 ? 0 : Math.ceil(lv / 5)]})"
					></div>
				</div>
			{/each}
		</div>
		<div class="mt-2 grid grid-cols-6 gap-2 border-t border-line pt-3 text-center text-xs">
			{#each GROUPS as g, i (g.key)}
				<div class="flex flex-col items-center gap-1">
					<Badge level={i * 5 + 1} size="xs" title={false} />
					<span class="text-muted">{g.name}</span>
					<span class="font-mono tabular">{groupCounts[i]}</span>
				</div>
			{/each}
		</div>
	</div>
</section>

<div class="grid gap-6 md:grid-cols-2">
	<section>
		<h2 class="mb-3 text-lg font-semibold">맞힌 문제</h2>
		<div class="card flex flex-wrap gap-x-3 gap-y-1 p-4 font-mono text-sm">
			{#each p.solved as s (s.id)}
				<a class="text-ok no-underline hover:underline" href="/problems/{s.id}" title={s.title}>{s.id}</a>
			{:else}
				<span class="font-sans text-muted">없음</span>
			{/each}
		</div>
	</section>
	<section>
		<h2 class="mb-3 text-lg font-semibold">시도했지만 맞히지 못한 문제</h2>
		<div class="card flex flex-wrap gap-x-3 gap-y-1 p-4 font-mono text-sm">
			{#each p.failed as s (s.id)}
				<a class="text-bad no-underline hover:underline" href="/problems/{s.id}" title={s.title}>{s.id}</a>
			{:else}
				<span class="font-sans text-muted">없음</span>
			{/each}
		</div>
	</section>
</div>
