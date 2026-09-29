<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import source from '$lib/legal/terms.md?raw';
</script>

<svelte:head><title>이용약관 · acc</title></svelte:head>

<article class="mx-auto max-w-3xl">
	<Markdown {source} />
</article>
