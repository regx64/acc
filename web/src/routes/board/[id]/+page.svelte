<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import Markdown from '$lib/components/Markdown.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import { dateTime, KIND_LABEL, LANGUAGE_LABEL } from '$lib/format';

	let { data, form } = $props();
	const p = $derived(data.post);
	const admin = $derived(data.user?.role === 'ADMIN');
</script>

<svelte:head><title>{p.title} · acc</title></svelte:head>

{#snippet code(c: string | null | undefined, lang: string | null | undefined, hidden: boolean)}
	{#if c}
		<div class="mt-3">
			<p class="mb-1 text-xs text-muted">첨부 코드{lang ? ` · ${LANGUAGE_LABEL[lang] ?? lang}` : ''}</p>
			<pre class="sample">{c}</pre>
		</div>
	{:else if hidden}
		<p class="mt-3 rounded-md bg-surface-2 px-3 py-2 text-sm text-muted">첨부 코드는 이 문제를 맞힌 사람에게만 보입니다.</p>
	{/if}
{/snippet}

{#snippet report(type: 'POST' | 'COMMENT', id: number)}
	{#if data.user}
		<details class="inline">
			<summary class="cursor-pointer text-xs text-faint hover:text-fg">신고</summary>
			<form method="POST" action="?/report" use:enhance class="mt-2 flex gap-2">
				<input type="hidden" name="target_type" value={type} />
				<input type="hidden" name="target_id" value={id} />
				<input class="input py-1 text-sm" name="reason" placeholder="사유" maxlength="500" />
				<button class="btn py-1 text-sm">신고</button>
			</form>
		</details>
	{/if}
{/snippet}

<p class="mb-2 text-sm"><a class="link" href="/problems/{p.problem_id}/board">← {p.problem_id}번 {p.problem_title} 게시판</a></p>
<FormMessage {form} />

<article class="card mt-3 p-6" class:opacity-60={p.hidden}>
	<header class="mb-4 border-b border-line pb-3">
		<p class="text-xs font-medium text-accent">{KIND_LABEL[p.kind]}{#if p.kind === 'TYPO'} · {p.resolved ? '처리됨' : '처리 전'}{/if}{#if p.hidden} · <span class="text-bad">숨김</span>{/if}</p>
		<h1 class="mt-1 text-xl font-semibold">{p.title}</h1>
		<p class="mt-1 text-sm text-muted"><UserLink handle={p.handle} /> · {dateTime(p.created_at)}</p>
	</header>
	<Markdown source={p.body} />
	{@render code(p.code, p.code_language, p.code_hidden)}
	<footer class="mt-5 flex flex-wrap items-center gap-3">
		{@render report('POST', p.id)}
		{#if p.is_mine || admin}
			<form method="POST" action="?/deletePost" use:enhance><button class="text-xs text-faint hover:text-bad">삭제</button></form>
		{/if}
		{#if admin}
			<form method="POST" action="?/visibility" use:enhance>
				<input type="hidden" name="target" value="post" /><input type="hidden" name="id" value={p.id} />
				<input type="hidden" name="hidden" value={String(!p.hidden)} />
				<button class="text-xs text-faint hover:text-fg">{p.hidden ? '다시 보이기' : '숨기기'}</button>
			</form>
			{#if p.kind === 'TYPO' && !p.resolved}
				<form method="POST" action="?/resolve" use:enhance><button class="text-xs text-faint hover:text-fg">처리 완료</button></form>
			{/if}
		{/if}
	</footer>
</article>

<section class="mt-6">
	<h2 class="mb-3 font-semibold">댓글 {p.comments.length}</h2>
	<ul class="grid gap-3">
		{#each p.comments as c (c.id)}
			<li class="card p-4" class:opacity-60={c.hidden}>
				<p class="mb-2 text-sm text-muted"><UserLink handle={c.handle} /> · {dateTime(c.created_at)}{#if c.hidden} · <span class="text-bad">숨김</span>{/if}</p>
				<Markdown source={c.body} />
				{@render code(c.code, c.code_language, c.code_hidden)}
				<div class="mt-2 flex gap-3">
					{@render report('COMMENT', c.id)}
					{#if c.is_mine || admin}
						<form method="POST" action="?/deleteComment" use:enhance><input type="hidden" name="id" value={c.id} /><button class="text-xs text-faint hover:text-bad">삭제</button></form>
					{/if}
					{#if admin}
						<form method="POST" action="?/visibility" use:enhance>
							<input type="hidden" name="target" value="comment" /><input type="hidden" name="id" value={c.id} />
							<input type="hidden" name="hidden" value={String(!c.hidden)} />
							<button class="text-xs text-faint hover:text-fg">{c.hidden ? '다시 보이기' : '숨기기'}</button>
						</form>
					{/if}
				</div>
			</li>
		{/each}
	</ul>

	{#if data.user}
		<form method="POST" action="?/comment" use:enhance class="card mt-4 grid gap-3 p-4">
			<label class="label" for="c-body">댓글</label>
			<textarea class="input font-sans" id="c-body" name="body" required></textarea>
			<details>
				<summary class="cursor-pointer text-sm text-muted">코드 첨부</summary>
				<div class="mt-2 grid gap-2">
					<select class="input w-40" name="code_language" aria-label="코드 언어">
						<option value="">언어</option>
						{#each Object.entries(LANGUAGE_LABEL) as [id, name] (id)}<option value={id}>{name}</option>{/each}
					</select>
					<textarea class="input" name="code" aria-label="코드"></textarea>
				</div>
			</details>
			<div class="flex justify-end"><button class="btn btn-primary">댓글 달기</button></div>
		</form>
	{:else}
		<p class="mt-4 text-sm text-muted"><a class="link" href="/login">로그인</a>하면 댓글을 달 수 있습니다.</p>
	{/if}
</section>
