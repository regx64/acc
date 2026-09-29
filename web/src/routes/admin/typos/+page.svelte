<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import { dateTime } from '$lib/format';
	let { data, form } = $props();
</script>

<svelte:head><title>오타 제보 · acc</title></svelte:head>

<div class="mb-4"><FormMessage {form} /></div>
<div class="table-wrap">
	<table class="table">
		<thead><tr><th>문제</th><th class="w-full">제목</th><th>제보자</th><th>시간</th><th></th></tr></thead>
		<tbody>
			{#each data.rows as t (t.id)}
				<tr>
					<td class="font-mono"><a class="link" href="/my/problems/{t.problem_id}">{t.problem_id}</a></td>
					<td class="wrap"><a class="link" href="/board/{t.id}">{t.title}</a></td>
					<td><UserLink handle={t.handle} /></td>
					<td class="tabular text-muted">{dateTime(t.created_at)}</td>
					<td><form method="POST" action="?/resolve" use:enhance><input type="hidden" name="id" value={t.id} /><button class="btn py-0.5 text-xs">처리 완료</button></form></td>
				</tr>
			{:else}
				<tr><td colspan="5" class="py-10 text-center text-muted">처리할 오타 제보가 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
