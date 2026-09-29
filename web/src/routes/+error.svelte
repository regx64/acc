<script lang="ts">
	import { page } from '$app/state';
</script>

<svelte:head><title>{page.status} · acc</title></svelte:head>

<div class="mx-auto max-w-md py-16 text-center">
	<p class="font-mono text-5xl font-semibold text-faint">{page.status}</p>
	<p class="mt-4 text-lg">{page.error?.message ?? '문제가 생겼습니다.'}</p>
	<div class="mt-6 flex justify-center gap-2">
		<a class="btn" href="/">처음으로</a>
		{#if page.status === 401}<a class="btn btn-primary" href="/login?next={encodeURIComponent(page.url.pathname)}">로그인</a>{/if}
	</div>
</div>
