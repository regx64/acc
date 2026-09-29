<script lang="ts">
	import { renderMarkdown } from '$lib/markdown';
	let { source }: { source: string | null | undefined } = $props();
	const html = $derived(renderMarkdown(source));
</script>

<!-- renderMarkdown escapes raw HTML and filters link targets. -->
<div class="prose-acc">{@html html}</div>
