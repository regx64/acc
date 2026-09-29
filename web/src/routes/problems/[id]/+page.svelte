<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import { num } from '$lib/format';

	let { data } = $props();
	const p = $derived(data.problem);
	let copied = $state<number | null>(null);

	async function copy(text: string, i: number) {
		try {
			await navigator.clipboard.writeText(text);
			copied = i;
			setTimeout(() => (copied = null), 1200);
		} catch {
			copied = null;
		}
	}
</script>

<svelte:head><title>{p.id}번: {p.title} · acc</title></svelte:head>

<div class="table-wrap mb-8">
	<table class="table text-center">
		<thead><tr><th class="text-center">시간 제한</th><th class="text-center">메모리 제한</th><th class="text-center">제출</th><th class="text-center">맞힌 사람</th><th class="text-center">정답 비율</th></tr></thead>
		<tbody>
			<tr class="tabular">
				<td>{p.time_limit_ms / 1000}초</td>
				<td>{Math.round(p.memory_limit_kb / 1024)} MB</td>
				<td>{num(p.submission_count)}</td>
				<td>{num(p.solved_count)}</td>
				<td>{(p.accept_rate * 100).toFixed(1)}%</td>
			</tr>
		</tbody>
	</table>
</div>

<div class="grid max-w-3xl gap-8">
	<section>
		<h2 class="mb-2 border-b border-line pb-1 text-lg font-semibold">문제</h2>
		<Markdown source={p.statement.legend} />
	</section>
	<section>
		<h2 class="mb-2 border-b border-line pb-1 text-lg font-semibold">입력</h2>
		<Markdown source={p.statement.input} />
	</section>
	<section>
		<h2 class="mb-2 border-b border-line pb-1 text-lg font-semibold">출력</h2>
		<Markdown source={p.statement.output} />
	</section>
	{#each p.statement.samples ?? [] as s, i (i)}
		<section class="grid gap-4 md:grid-cols-2">
			<div class="min-w-0">
				<h3 class="mb-2 flex items-center justify-between font-semibold">
					예제 입력 {i + 1}
					<button class="text-xs font-normal text-muted hover:text-fg" onclick={() => copy(s.input, i * 2)}>{copied === i * 2 ? '복사함' : '복사'}</button>
				</h3>
				<pre class="sample">{s.input}</pre>
			</div>
			<div class="min-w-0">
				<h3 class="mb-2 flex items-center justify-between font-semibold">
					예제 출력 {i + 1}
					<button class="text-xs font-normal text-muted hover:text-fg" onclick={() => copy(s.output, i * 2 + 1)}>{copied === i * 2 + 1 ? '복사함' : '복사'}</button>
				</h3>
				<pre class="sample">{s.output}</pre>
			</div>
		</section>
	{/each}
	{#if p.statement.hint}
		<section>
			<h2 class="mb-2 border-b border-line pb-1 text-lg font-semibold">힌트</h2>
			<Markdown source={p.statement.hint} />
		</section>
	{/if}
	<div>
		<a class="btn btn-primary" href="/problems/{p.id}/submit">제출하기</a>
	</div>
</div>
