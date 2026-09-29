<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import { dateTime, KIND_LABEL, LANGUAGE_LABEL } from '$lib/format';

	let { data, form } = $props();
	let writing = $state(false);
	$effect(() => {
		if (form?.message) writing = true;
	});
</script>

<svelte:head><title>{data.problem.id}번 게시판 · acc</title></svelte:head>

<div class="mb-4 flex items-center justify-between">
	<p class="text-sm text-muted">질문, 반례, 오타 제보. 첨부한 코드는 이 문제를 맞힌 사람에게만 보입니다.</p>
	{#if data.user}
		<button class="btn btn-primary" onclick={() => (writing = !writing)}>{writing ? '닫기' : '글쓰기'}</button>
	{:else}
		<a class="btn" href="/login">로그인하고 글쓰기</a>
	{/if}
</div>

{#if writing}
	<form method="POST" use:enhance class="card mb-6 grid gap-3 p-5">
		<FormMessage {form} />
		<div class="grid gap-3 sm:grid-cols-[10rem_1fr]">
			<div>
				<label class="label" for="kind">종류</label>
				<select class="input" id="kind" name="kind">
					{#each Object.entries(KIND_LABEL) as [k, v] (k)}<option value={k} selected={form?.kind === k}>{v}</option>{/each}
				</select>
			</div>
			<div>
				<label class="label" for="title">제목</label>
				<input class="input" id="title" name="title" maxlength="100" required defaultValue={form?.title ?? ''} />
			</div>
		</div>
		<div>
			<label class="label" for="body">본문 (Markdown, 수식은 $…$)</label>
			<textarea class="input font-sans" id="body" name="body" required>{form?.body ?? ''}</textarea>
		</div>
		<details>
			<summary class="cursor-pointer text-sm text-muted">코드 첨부</summary>
			<div class="mt-2 grid gap-2">
				<select class="input w-40" name="code_language" aria-label="코드 언어">
					<option value="">언어</option>
					{#each Object.entries(LANGUAGE_LABEL) as [id, name] (id)}<option value={id}>{name}</option>{/each}
				</select>
				<textarea class="input" name="code" aria-label="코드">{form?.code ?? ''}</textarea>
			</div>
		</details>
		<div class="flex justify-end"><button class="btn btn-primary">올리기</button></div>
	</form>
{/if}

<div class="table-wrap">
	<table class="table">
		<thead><tr><th>종류</th><th class="w-full">제목</th><th>글쓴이</th><th>작성일</th></tr></thead>
		<tbody>
			{#each data.posts.items as p (p.id)}
				<tr class:opacity-50={p.hidden}>
					<td><span class="text-xs font-medium {p.kind === 'TYPO' ? 'text-warn' : p.kind === 'COUNTEREXAMPLE' ? 'text-ce' : 'text-accent'}">{KIND_LABEL[p.kind]}</span></td>
					<td class="wrap">
						<a class="text-fg no-underline hover:underline" href="/board/{p.id}">{p.title}</a>
						{#if p.comment_count > 0}<span class="ml-1 text-xs text-faint">[{p.comment_count}]</span>{/if}
						{#if p.hidden}<span class="ml-1 text-xs text-bad">숨김</span>{/if}
					</td>
					<td><UserLink handle={p.handle} /></td>
					<td class="tabular text-muted">{dateTime(p.created_at).slice(0, 16)}</td>
				</tr>
			{:else}
				<tr><td colspan="4" class="py-10 text-center text-muted">아직 글이 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<Pager next={data.posts.next_cursor} />
