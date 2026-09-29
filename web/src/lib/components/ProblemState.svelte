<script lang="ts">
	let { status, validation }: { status: string; validation?: string | null } = $props();
	const statusText: Record<string, string> = { DRAFT: '초안', REVIEW: '검수 중', PUBLIC: '공개' };
	const validationText: Record<string, string> = { PENDING: '검증 중', PASSED: '검증 통과', FAILED: '검증 실패' };
</script>

<span class="inline-flex flex-wrap gap-1.5 text-xs">
	<span class="rounded-full border px-2 py-0.5 {status === 'PUBLIC' ? 'border-ok text-ok' : status === 'REVIEW' ? 'border-accent text-accent' : 'border-line-strong text-muted'}">{statusText[status] ?? status}</span>
	{#if validation}
		<span class="rounded-full border px-2 py-0.5 {validation === 'PASSED' ? 'border-ok text-ok' : validation === 'FAILED' ? 'border-bad text-bad' : 'border-line-strong text-muted'}">{validationText[validation] ?? validation}</span>
	{/if}
</span>
