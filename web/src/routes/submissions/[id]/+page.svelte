<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { enhance } from '$app/forms';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import Verdict from '$lib/components/Verdict.svelte';
	import { dateTime, isPending, LANGUAGE_LABEL, num } from '$lib/format';

	let { data, form } = $props();
	const s = $derived(data.s);

	$effect(() => {
		if (!isPending(s.status)) return;
		const t = setTimeout(() => invalidateAll(), 1500);
		return () => clearTimeout(t);
	});
</script>

<svelte:head><title>제출 {s.id} · acc</title></svelte:head>

<div class="mb-5 flex flex-wrap items-center justify-between gap-3">
	<h1 class="text-2xl font-semibold">제출 <span class="font-mono">{s.id}</span></h1>
	{#if data.user?.role === 'ADMIN'}
		<form method="POST" action="?/rejudge" use:enhance><button class="btn">재채점</button></form>
	{/if}
</div>
<FormMessage {form} />

<div class="table-wrap mb-6">
	<table class="table">
		<thead><tr><th>아이디</th><th>문제</th><th>결과</th><th class="text-right">메모리</th><th class="text-right">시간</th><th>언어</th><th class="text-right">코드 길이</th><th>제출한 시간</th></tr></thead>
		<tbody>
			<tr>
				<td><UserLink handle={s.handle} /></td>
				<td><a class="link" href="/problems/{s.problem_id}"><span class="font-mono">{s.problem_id}</span> {s.problem_title}</a></td>
				<td><Verdict status={s.status} /></td>
				<td class="text-right tabular">{s.memory_kb != null && s.status !== 'CE' ? `${num(s.memory_kb)} KB` : ''}</td>
				<td class="text-right tabular">{s.time_ms != null && s.status !== 'CE' ? `${num(s.time_ms)} ms` : ''}</td>
				<td>{LANGUAGE_LABEL[s.language] ?? s.language}</td>
				<td class="text-right tabular">{num(s.code_length)} B</td>
				<td class="tabular text-muted">{dateTime(s.created_at)}</td>
			</tr>
		</tbody>
	</table>
</div>

{#if s.compile_message}
	<h2 class="mb-2 font-semibold">컴파일 메시지</h2>
	<pre class="sample mb-6 max-h-96 text-bad">{s.compile_message}</pre>
{/if}

{#if s.code != null}
	<h2 class="mb-2 font-semibold">코드</h2>
	<CodeEditor value={s.code} language={s.language} readonly minHeight="8rem" />
{:else}
	<p class="card p-6 text-muted">코드는 제출한 사람과 이 문제를 맞힌 사람만 볼 수 있습니다.</p>
{/if}
