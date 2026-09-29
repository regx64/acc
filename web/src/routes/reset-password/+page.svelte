<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	let { data, form } = $props();
</script>

<svelte:head><title>새 비밀번호 · acc</title></svelte:head>

<div class="mx-auto max-w-sm">
	<h1 class="mb-6 text-2xl font-semibold">새 비밀번호</h1>
	<form method="POST" use:enhance class="card grid gap-4 p-6">
		<FormMessage {form} />
		<input type="hidden" name="token" value={data.token} />
		<div>
			<label class="label" for="password">새 비밀번호</label>
			<input class="input" id="password" name="password" type="password" minlength="8" required autocomplete="new-password" />
		</div>
		<p class="text-xs text-faint">바꾸면 모든 기기에서 로그아웃됩니다.</p>
		<button class="btn btn-primary">비밀번호 바꾸기</button>
	</form>
</div>
