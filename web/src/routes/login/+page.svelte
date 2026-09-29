<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	let { form } = $props();
</script>

<svelte:head><title>로그인 · acc</title></svelte:head>

<div class="mx-auto max-w-sm">
	<h1 class="mb-6 text-2xl font-semibold">로그인</h1>
	<form method="POST" use:enhance class="card grid gap-4 p-6">
		<FormMessage {form} />
		<div>
			<label class="label" for="login">핸들 또는 이메일</label>
			<input class="input" id="login" name="login" autocomplete="username" required defaultValue={form?.login ?? ''} />
		</div>
		<div>
			<label class="label" for="password">비밀번호</label>
			<input class="input" id="password" name="password" type="password" autocomplete="current-password" required />
		</div>
		<button class="btn btn-primary">로그인</button>
		<p class="flex justify-between text-sm">
			<a class="link" href="/forgot-password">비밀번호를 잊었어요</a>
			<a class="link" href="/signup">가입하기</a>
		</p>
	</form>
</div>
