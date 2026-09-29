<script lang="ts">
	import Badge from '$lib/components/Badge.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import Verdict from '$lib/components/Verdict.svelte';
	import { num } from '$lib/format';
	import { GROUPS } from '$lib/levels';

	let { data } = $props();
</script>

<svelte:head><title>acc</title></svelte:head>

<section class="grid gap-8 pb-10 md:grid-cols-[1.2fr_1fr] md:items-end">
	<div>
		<h1 class="font-mono text-6xl font-semibold tracking-tighter md:text-7xl">acc<span class="text-accent">_</span></h1>
		<p class="mt-4 max-w-xl text-lg text-muted">
			한국어로 된 문제를 풀고, 맞힌 문제의 레벨로 레이팅을 쌓는 온라인 저지.
		</p>
		<div class="mt-5 flex flex-wrap gap-2">
			<a class="btn btn-primary" href="/problems">문제 보기</a>
			<a class="btn" href="/create">문제 출제하기</a>
		</div>
	</div>
	<div class="card p-4">
		<p class="mb-3 text-sm text-muted">레벨은 수 체계 여섯 그룹 × 5단계</p>
		<div class="grid grid-cols-6 gap-2 text-center text-xs">
			{#each GROUPS as g, i (g.key)}
				<a href="/problems?group={g.key}" class="flex flex-col items-center gap-1 rounded-md py-2 text-muted no-underline hover:bg-surface-2">
					<Badge level={i * 5 + 3} size="md" />
					<span>{g.name}</span>
				</a>
			{/each}
		</div>
	</div>
</section>

<div class="grid gap-6 lg:grid-cols-3">
	<section class="lg:col-span-2">
		<h2 class="mb-3 text-lg font-semibold">최근 공개된 문제</h2>
		{#if data.problems.length === 0}
			<p class="card p-6 text-muted">아직 공개된 문제가 없습니다.</p>
		{:else}
			<div class="table-wrap">
				<table class="table">
					<thead><tr><th>번호</th><th>레벨</th><th>제목</th><th class="text-right">맞힌 사람</th></tr></thead>
					<tbody>
						{#each data.problems as p (p.id)}
							<tr>
								<td class="font-mono tabular"><a class="link" href="/problems/{p.id}">{p.id}</a></td>
								<td><Badge level={p.level} /></td>
								<td class="wrap"><a class="text-fg no-underline hover:underline" href="/problems/{p.id}">{p.title}</a></td>
								<td class="text-right tabular">{num(p.solved_count)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
	<section>
		<h2 class="mb-3 text-lg font-semibold">상위 사용자</h2>
		{#if data.ranking.length === 0}
			<p class="card p-6 text-muted">아직 순위가 없습니다.</p>
		{:else}
			<ol class="card divide-y divide-line">
				{#each data.ranking as r (r.id)}
					<li class="flex items-center gap-3 px-4 py-2 text-sm">
						<span class="w-6 font-mono text-faint tabular">{r.rank}</span>
						<Badge level={r.tier} size="xs" />
						<span class="flex-1"><UserLink handle={r.handle} /></span>
						<span class="font-mono tabular">{num(r.rating)}</span>
					</li>
				{/each}
			</ol>
		{/if}
		<h2 class="mt-6 mb-3 text-lg font-semibold">방금 채점된 제출</h2>
		<ul class="card divide-y divide-line text-sm">
			{#each data.recent as s (s.id)}
				<li class="flex items-center justify-between gap-2 px-4 py-2">
					<span class="truncate"><UserLink handle={s.handle} /> · <a class="link" href="/problems/{s.problem_id}">{s.problem_id}</a></span>
					<Verdict status={s.status} />
				</li>
			{:else}
				<li class="px-4 py-3 text-muted">아직 제출이 없습니다.</li>
			{/each}
		</ul>
	</section>
</div>
