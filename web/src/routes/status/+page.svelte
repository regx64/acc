<script lang="ts">
	import Pager from '$lib/components/Pager.svelte';
	import StatusFilter from '$lib/components/StatusFilter.svelte';
	import SubmissionTable from '$lib/components/SubmissionTable.svelte';
	let { data } = $props();
</script>

<svelte:head><title>채점 현황 · acc</title></svelte:head>

<h1 class="mb-5 text-2xl font-semibold">채점 현황</h1>
<StatusFilter query={data.query} />
<SubmissionTable rows={data.page.items} />
<Pager next={data.page.next_cursor} />
