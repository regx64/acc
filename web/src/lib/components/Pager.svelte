<script lang="ts">
	import { page } from '$app/state';
	let { next }: { next: string | null | undefined } = $props();

	function withCursor(cursor: string | null) {
		const u = new URL(page.url);
		if (cursor) u.searchParams.set('cursor', cursor);
		else u.searchParams.delete('cursor');
		return u.pathname + u.search;
	}
	const hasPrev = $derived(page.url.searchParams.has('cursor'));
</script>

{#if next || hasPrev}
	<nav class="mt-4 flex justify-between gap-2" aria-label="페이지">
		{#if hasPrev}<a class="btn" href={withCursor(null)}>처음으로</a>{:else}<span></span>{/if}
		{#if next}<a class="btn" href={withCursor(next)}>다음 페이지</a>{/if}
	</nav>
{/if}
