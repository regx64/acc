<script lang="ts">
	import FormMessage from '$lib/components/FormMessage.svelte';
	let { data, form } = $props();
</script>

<svelte:head><title>이메일 인증 · acc</title></svelte:head>

<div class="mx-auto max-w-md py-10 text-center">
	<h1 class="text-2xl font-semibold">이메일 인증</h1>
	<div class="mt-6 grid gap-4">
		<FormMessage {form} />
		{#if data.verified}
			<p class="text-ok">인증했습니다. 이제 제출과 글쓰기를 할 수 있습니다.</p>
			<a class="btn btn-primary mx-auto" href="/problems">문제 보러 가기</a>
		{:else if data.message}
			<p class="text-bad">{data.message}</p>
		{:else if !form}
			<p class="text-muted">메일로 받은 인증 링크를 여세요.</p>
		{/if}
		{#if !data.verified && data.user && !data.user.email_verified}
			<form method="POST" action="?/resend"><button class="btn">인증 메일 다시 보내기</button></form>
		{/if}
	</div>
</div>
