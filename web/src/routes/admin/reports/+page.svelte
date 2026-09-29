<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import { dateTime } from '$lib/format';
	let { data, form } = $props();
</script>

<svelte:head><title>신고 · acc</title></svelte:head>

<p class="mb-4 text-sm text-muted">서로 다른 사용자 3명이 신고하면 글이 자동으로 숨겨집니다. 처리하면 같은 대상의 신고가 모두 닫힙니다.</p>
<div class="mb-4"><FormMessage {form} /></div>
<ul class="grid gap-3">
	{#each data.rows as r (r.id)}
		<li class="card grid gap-2 p-4">
			<div class="flex flex-wrap items-center justify-between gap-2 text-sm">
				<span>
					<span class="font-medium">{r.target_type === 'POST' ? '글' : '댓글'} #{r.target_id}</span>
					· 신고 {r.report_count}건
					{#if r.target_hidden}<span class="text-bad"> · 숨김 상태</span>{/if}
					{#if r.post_id}· <a class="link" href="/board/{r.post_id}">보기</a>{/if}
				</span>
				<span class="text-xs text-faint"><UserLink handle={r.reporter} /> · {dateTime(r.created_at)}</span>
			</div>
			<p class="text-sm"><span class="text-muted">사유:</span> {r.reason}</p>
			{#if r.excerpt}<p class="rounded bg-surface-2 px-3 py-2 text-sm text-muted">{r.excerpt}</p>{/if}
			<div class="flex gap-2">
				<form method="POST" action="?/resolve" use:enhance><input type="hidden" name="id" value={r.id} /><input type="hidden" name="hide" value="true" /><button class="btn btn-danger py-1 text-sm">숨기고 닫기</button></form>
				<form method="POST" action="?/resolve" use:enhance><input type="hidden" name="id" value={r.id} /><input type="hidden" name="hide" value="false" /><button class="btn py-1 text-sm">문제없음 (보이기)</button></form>
			</div>
		</li>
	{:else}
		<li class="card p-8 text-center text-muted">처리할 신고가 없습니다.</li>
	{/each}
</ul>
