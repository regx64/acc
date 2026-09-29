<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	let { form } = $props();
</script>

<svelte:head><title>가입 · acc</title></svelte:head>

<div class="mx-auto max-w-sm">
	<h1 class="mb-6 text-2xl font-semibold">가입</h1>
	<form method="POST" use:enhance class="card grid gap-4 p-6">
		<FormMessage {form} />
		<div>
			<label class="label" for="handle">핸들</label>
			<input class="input font-mono" id="handle" name="handle" required minlength="3" maxlength="20" pattern="[a-z0-9_]{'{'}3,20{'}'}" autocomplete="username" defaultValue={form?.handle ?? ''} />
			<p class="mt-1 text-xs text-faint">영문 소문자, 숫자, 밑줄 3~20자. 90일에 한 번 바꿀 수 있습니다.</p>
		</div>
		<div>
			<label class="label" for="email">이메일</label>
			<input class="input" id="email" name="email" type="email" required autocomplete="email" defaultValue={form?.email ?? ''} />
		</div>
		<div>
			<label class="label" for="password">비밀번호</label>
			<input class="input" id="password" name="password" type="password" minlength="8" required autocomplete="new-password" />
			<p class="mt-1 text-xs text-faint">8자 이상</p>
		</div>
		<label class="flex items-start gap-2 text-sm">
			<input type="checkbox" name="agree" required class="mt-1" />
			<span><a class="link" href="/terms" target="_blank">이용약관</a>과 <a class="link" href="/privacy" target="_blank">개인정보처리방침</a>에 동의합니다.</span>
		</label>
		<button class="btn btn-primary">가입하기</button>
	</form>
</div>
