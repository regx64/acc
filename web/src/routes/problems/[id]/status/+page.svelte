<script lang="ts">
	import Pager from '$lib/components/Pager.svelte';
	import StatusFilter from '$lib/components/StatusFilter.svelte';
	import SubmissionTable from '$lib/components/SubmissionTable.svelte';
	let { data } = $props();
</script>

<svelte:head><title>{data.problem.id}번 채점 현황 · acc</title></svelte:head>

<StatusFilter query={data.query} problemField={false} />
<SubmissionTable rows={data.page.items} showProblem={false} />
<Pager next={data.page.next_cursor} />
