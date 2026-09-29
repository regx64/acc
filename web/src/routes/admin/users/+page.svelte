<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import UserLink from '$lib/components/UserLink.svelte';
	import { date, num } from '$lib/format';
	let { data, form } = $props();
</script>

<svelte:head><title>사용자 관리 · acc</title></svelte:head>

<form method="GET" class="mb-4 flex gap-2">
	<input class="input w-64" name="q" value={data.q} placeholder="핸들 또는 이메일" aria-label="사용자 검색" />
	<button class="btn">검색</button>
</form>
<div class="mb-4"><FormMessage {form} /></div>
<div class="table-wrap">
	<table class="table">
		<thead><tr><th>ID</th><th>핸들</th><th>이메일</th><th>권한</th><th class="text-right">레이팅</th><th>가입일</th><th>정지</th></tr></thead>
		<tbody>
			{#each data.rows as u (u.id)}
				<tr>
					<td class="font-mono text-muted">{u.id}</td>
					<td><UserLink handle={u.handle} /></td>
					<td class="text-muted">{u.email}{u.email_verified ? '' : ' (미인증)'}</td>
					<td class="text-xs">{u.role}</td>
					<td class="text-right tabular">{num(u.rating)}</td>
					<td class="tabular text-muted">{date(u.created_at)}</td>
					<td class="wrap">
						{#if u.suspended}
							<form method="POST" action="?/unsuspend" use:enhance class="flex items-center gap-2">
								<input type="hidden" name="id" value={u.id} />
								<span class="text-sm text-bad">{u.suspend_reason}</span>
								<button class="btn py-0.5 text-xs">해제</button>
							</form>
						{:else if u.role !== 'ADMIN'}
							<form method="POST" action="?/suspend" use:enhance class="flex gap-2">
								<input type="hidden" name="id" value={u.id} />
								<input class="input py-0.5 text-sm" name="reason" placeholder="사유" required aria-label="정지 사유" />
								<button class="btn btn-danger py-0.5 text-xs">정지</button>
							</form>
						{/if}
					</td>
				</tr>
			{:else}
				<tr><td colspan="7" class="py-10 text-center text-muted">사용자가 없습니다.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
