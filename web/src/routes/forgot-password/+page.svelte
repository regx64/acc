<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	let { form } = $props();
</script>

<svelte:head><title>비밀번호 재설정 · acc</title></svelte:head>

<div class="mx-auto max-w-sm">
	<h1 class="mb-6 text-2xl font-semibold">비밀번호 재설정</h1>
	<form method="POST" use:enhance class="card grid gap-4 p-6">
		<FormMessage {form} />
		<div>
			<label class="label" for="email">가입한 이메일</label>
			<input class="input" id="email" name="email" type="email" required autocomplete="email" />
		</div>
		<button class="btn btn-primary">재설정 링크 받기</button>
	</form>
</div>
