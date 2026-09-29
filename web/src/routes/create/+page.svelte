<script lang="ts">
	import FormMessage from '$lib/components/FormMessage.svelte';
	import ProblemForm from '$lib/components/ProblemForm.svelte';

	let { data, form } = $props();
	const isAdmin = $derived(data.user?.role === 'ADMIN');
</script>

<svelte:head><title>출제 · acc</title></svelte:head>

<div class="mb-6">
	<h1 class="text-2xl font-semibold">문제 출제</h1>
	<p class="mt-1 text-sm text-muted">
		초안을 저장한 뒤 테스트 데이터를 올리고 정해 검증을 통과하면 검수를 요청할 수 있습니다. 검수를 통과한 문제만 공개됩니다.
		<a class="link" href="/my/problems">내 문제</a>
	</p>
</div>

<form method="POST" class="grid gap-6">
	<FormMessage {form} />
	<ProblemForm initial={form?.draft ?? {}} />
	{#if !isAdmin}
		<label class="flex items-start gap-2 text-sm">
			<input type="checkbox" name="agree" required class="mt-1" />
			<span>
				직접 만든 문제이며 게시할 권리가 있고, 지문이 CC BY-SA 4.0으로 공개되는 데 동의합니다.
				<a class="link" href="/authoring-terms" target="_blank">출제 약관</a>
			</span>
		</label>
	{/if}
	<div><button class="btn btn-primary">초안 저장</button></div>
</form>
