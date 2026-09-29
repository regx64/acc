<script lang="ts">
	let { form }: { form: { message?: string; ok?: string } | null | undefined } = $props();
</script>

{#if form?.message}
	<p class="rounded-md border border-bad/40 bg-bad/10 px-3 py-2 text-sm text-bad" role="alert">{form.message}</p>
{:else if form?.ok}
	<p class="rounded-md border border-ok/40 bg-ok/10 px-3 py-2 text-sm text-ok" role="status">{form.ok}</p>
{/if}
