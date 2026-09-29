<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import { num } from '$lib/format';

	let { data, form } = $props();
	const o = $derived(data.o);
	const queueRatio = $derived(o.queue_limit ? o.queue_length / o.queue_limit : 0);
	const tiles = $derived([
		{ label: '채점 대기열', value: `${num(o.queue_length)} / ${num(o.queue_limit)}`, warn: queueRatio > 0.5 },
		{ label: '채점 중·대기 제출', value: num(o.judging), warn: false },
		{ label: '24시간 채점', value: num(o.judged_24h), warn: false },
		{ label: '24시간 채점 오류율', value: `${(o.system_error_rate_24h * 100).toFixed(2)}%`, warn: o.system_error_rate_24h > 0.01 },
		{ label: '검수 대기', value: num(o.pending_reviews), warn: o.pending_reviews > 0, href: '/admin/problems?status=REVIEW' },
		{ label: '처리 안 한 신고', value: num(o.open_reports), warn: o.open_reports > 0, href: '/admin/reports' },
		{ label: '처리 안 한 오타 제보', value: num(o.open_typos), warn: o.open_typos > 0, href: '/admin/typos' },
		{ label: '사용자 / 공개 문제', value: `${num(o.users)} / ${num(o.public_problems)}`, warn: false }
	]);
</script>

<svelte:head><title>관리 · acc</title></svelte:head>

<div class="mb-8 grid grid-cols-2 gap-3 md:grid-cols-4">
	{#each tiles as t (t.label)}
		<a href={t.href ?? '#'} class="card block p-4 text-fg no-underline {t.warn ? 'border-warn' : ''}" class:pointer-events-none={!t.href}>
			<p class="text-xs text-muted">{t.label}</p>
			<p class="mt-1 font-mono text-xl font-semibold tabular {t.warn ? 'text-warn' : ''}">{t.value}</p>
		</a>
	{/each}
</div>

<form method="POST" action="?/banner" use:enhance={() => async ({ update }) => update({ reset: false })} class="card grid max-w-2xl gap-3 p-5">
	<h2 class="font-semibold">점검 공지 배너</h2>
	<FormMessage {form} />
	<input class="input" name="text" value={data.site.banner ?? ''} placeholder="비우면 배너를 내립니다" aria-label="배너 문구" />
	<div><button class="btn">저장</button></div>
</form>
