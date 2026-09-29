<script lang="ts">
	import { STATUS_LABEL, isPending } from '$lib/format';
	let { status }: { status: string } = $props();
	const tone = $derived(
		status === 'AC' ? 'text-ok' : status === 'WA' || status === 'SE' ? 'text-bad' : status === 'CE' ? 'text-ce' : isPending(status) ? 'text-muted' : 'text-warn'
	);
</script>

<span class="font-semibold {tone}" class:animate-pulse={isPending(status)}>{STATUS_LABEL[status] ?? status}</span>
