<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import source from '$lib/legal/privacy.md?raw';
</script>

<svelte:head><title>개인정보처리방침 · acc</title></svelte:head>

<article class="mx-auto max-w-3xl">
	<Markdown {source} />
</article>
