<script lang="ts">
	import { enhance } from '$app/forms';
	import CodeEditor from '$lib/components/CodeEditor.svelte';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import { LANGUAGE_LABEL } from '$lib/format';
	import { untrack } from 'svelte';

	let { data, form } = $props();
	let language = $state(untrack(() => form?.language ?? data.defaultLanguage));
	let code = $state(untrack(() => form?.code ?? ''));
	let sending = $state(false);
</script>

<svelte:head><title>{data.problem.id}번 제출 · acc</title></svelte:head>

<form
	method="POST"
	class="grid gap-4"
	use:enhance={() => {
		sending = true;
		return async ({ update }) => {
			await update({ reset: false });
			sending = false;
		};
	}}
>
	<FormMessage {form} />
	<div class="flex flex-wrap items-end gap-3">
		<div>
			<label class="label" for="language">언어</label>
			<select class="input w-44" id="language" name="language" bind:value={language}>
				{#each Object.entries(LANGUAGE_LABEL) as [id, name] (id)}<option value={id}>{name}</option>{/each}
			</select>
		</div>
		<p class="text-xs text-faint">Java는 클래스 이름을 Main으로. 제출은 10초에 한 번 할 수 있습니다.</p>
	</div>
	<CodeEditor bind:value={code} {language} name="code" />
	<div class="flex justify-end">
		<button class="btn btn-primary" disabled={sending || !code.trim()}>{sending ? '제출 중…' : '제출'}</button>
	</div>
</form>
