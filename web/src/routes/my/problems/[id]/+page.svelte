<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import Badge from '$lib/components/Badge.svelte';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import ProblemForm from '$lib/components/ProblemForm.svelte';
	import ProblemState from '$lib/components/ProblemState.svelte';
	import { bytes, dateTime } from '$lib/format';

	let { data, form } = $props();
	const p = $derived(data.p);
	const admin = $derived(data.user?.role === 'ADMIN');
	const editable = $derived(admin || p.status === 'DRAFT');
	const dataCases = $derived(p.testcases.filter((t) => !t.is_sample));
	const sampleCases = $derived(p.testcases.filter((t) => t.is_sample));
	const totalBytes = $derived(dataCases.reduce((a, t) => a + t.input_size + t.output_size, 0));
	const decisionText: Record<string, string> = { SUBMITTED: '검수 요청', APPROVED: '승인', REJECTED: '반려' };

	$effect(() => {
		if (p.validation_status !== 'PENDING') return;
		const t = setTimeout(() => invalidateAll(), 1500);
		return () => clearTimeout(t);
	});

	const keep = () => async ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });
</script>

<svelte:head><title>{p.id}번 편집 · acc</title></svelte:head>

<div class="mb-6 flex flex-wrap items-start justify-between gap-4">
	<div>
		<p class="text-sm"><a class="link" href="/my/problems">← 내 문제</a></p>
		<h1 class="mt-1 text-2xl font-semibold"><span class="mr-2 font-mono text-faint">{p.id}</span>{p.title}</h1>
		<div class="mt-2 flex items-center gap-3"><ProblemState status={p.status} validation={p.validation_status} /> <a class="link text-sm" href="/problems/{p.id}">문제 페이지 보기</a></div>
	</div>
</div>

<div class="mb-6"><FormMessage {form} /></div>

<div class="grid gap-6 lg:grid-cols-[1fr_20rem]">
	<form method="POST" action="?/save" use:enhance={keep} class="min-w-0">
		{#key p.id}
			<ProblemForm
				initial={{
					title: p.title,
					time_limit_ms: p.time_limit_ms,
					memory_limit_mb: p.memory_limit_mb,
					proposed_level: p.proposed_level ?? 0,
					statement: p.statement,
					solution_language: p.solution_language,
					solution_code: p.solution_code
				}}
				disabled={!editable}
			/>
		{/key}
		{#if editable}
			<div class="mt-5"><button class="btn btn-primary">저장</button></div>
		{:else}
			<p class="mt-5 text-sm text-muted">검수 중이거나 공개된 문제는 수정할 수 없습니다.</p>
		{/if}
	</form>

	<aside class="grid min-w-0 content-start gap-4">
		<section class="card grid gap-3 p-5">
			<h2 class="font-semibold">테스트 데이터</h2>
			<p class="text-sm text-muted">예제 {sampleCases.length}개 + 데이터 {dataCases.length}개 · {bytes(totalBytes)}</p>
			{#if editable}
				<form method="POST" action="?/upload" enctype="multipart/form-data" use:enhance class="grid gap-2">
					<input class="w-full text-sm" type="file" name="file" accept=".zip,application/zip" required aria-label="테스트 zip" />
					<p class="text-xs text-faint">1.in, 1.out, 2.in, 2.out … 을 담은 zip. 100개, 50MB 이하. 올리면 기존 데이터를 바꿉니다.</p>
					<button class="btn">올리기</button>
				</form>
			{/if}
		</section>

		<section class="card grid gap-3 p-5">
			<h2 class="font-semibold">정해 검증</h2>
			{#if p.validation_status === 'PENDING'}
				<p class="animate-pulse text-sm text-muted">검증 중…</p>
			{:else if p.validation_message}
				<p class="max-h-60 overflow-auto rounded-md border border-line bg-surface-2 p-3 font-mono text-xs break-all whitespace-pre-wrap {p.validation_status === 'PASSED' ? 'text-ok' : 'text-bad'}">{p.validation_message}</p>
				<p class="text-xs text-faint">{dateTime(p.validated_at)}</p>
			{:else}
				<p class="text-sm text-muted">정해가 모든 테스트를 시간 제한의 절반 안에 통과하는지 확인합니다.</p>
			{/if}
			{#if editable}
				<form method="POST" action="?/validate" use:enhance><button class="btn w-full" disabled={p.validation_status === 'PENDING'}>검증 실행</button></form>
			{/if}
		</section>

		{#if p.status === 'DRAFT' && p.source === 'USER'}
			<section class="card grid gap-3 p-5">
				<h2 class="font-semibold">검수 요청</h2>
				<p class="text-sm text-muted">검증을 통과해야 요청할 수 있습니다. 검수 중에는 수정할 수 없습니다.</p>
				<form method="POST" action="?/review" use:enhance><button class="btn btn-primary w-full" disabled={p.validation_status !== 'PASSED'}>검수 요청</button></form>
			</section>
		{:else if p.status === 'REVIEW' && !admin}
			<section class="card grid gap-3 p-5">
				<h2 class="font-semibold">검수 중</h2>
				<form method="POST" action="?/withdraw" use:enhance><button class="btn w-full">요청 취소하고 수정하기</button></form>
			</section>
		{/if}

		{#if admin}
			<section class="card grid gap-3 border-accent/50 p-5">
				<h2 class="font-semibold">관리</h2>
				{#if p.status !== 'PUBLIC'}
					<form method="POST" action="?/decide" use:enhance class="grid gap-2">
						<input type="hidden" name="approve" value="true" />
						<label class="label" for="a-level">확정 레벨 (제안: {p.proposed_level ?? '없음'})</label>
						<div class="flex gap-2">
							<input class="input tabular" id="a-level" name="level" type="number" min="0" max="30" value={p.proposed_level ?? 0} required />
							<button class="btn btn-primary">승인·공개</button>
						</div>
					</form>
					<form method="POST" action="?/decide" use:enhance class="grid gap-2">
						<input type="hidden" name="approve" value="false" />
						<textarea class="input min-h-16 font-sans" name="comment" placeholder="반려 사유" required aria-label="반려 사유"></textarea>
						<button class="btn btn-danger">반려</button>
					</form>
				{:else}
					<form method="POST" action="?/patch" use:enhance class="grid gap-2">
						<label class="label" for="p-level">레벨 <Badge level={p.level} /></label>
						<div class="flex gap-2">
							<input class="input tabular" id="p-level" name="level" type="number" min="0" max="30" value={p.level} />
							<button class="btn">변경</button>
						</div>
						<p class="text-xs text-faint">바꾸면 푼 사용자의 레이팅을 다시 계산합니다.</p>
					</form>
					<form method="POST" action="?/patch" use:enhance>
						<input type="hidden" name="status" value="DRAFT" />
						<button class="btn w-full">비공개로 돌리기</button>
					</form>
					<form method="POST" action="?/rejudge" use:enhance><button class="btn w-full">전체 재채점</button></form>
				{/if}
			</section>
		{/if}

		{#if p.reviews.length}
			<section class="card grid gap-2 p-5">
				<h2 class="font-semibold">검수 기록</h2>
				<ol class="grid gap-2 text-sm">
					{#each p.reviews as r, i (i)}
						<li>
							<span class="font-medium {r.decision === 'APPROVED' ? 'text-ok' : r.decision === 'REJECTED' ? 'text-bad' : ''}">{decisionText[r.decision] ?? r.decision}</span>
							<span class="text-xs text-faint">{dateTime(r.created_at)}{r.reviewer ? ` · ${r.reviewer}` : ''}</span>
							{#if r.comment}<p class="text-muted">{r.comment}</p>{/if}
						</li>
					{/each}
				</ol>
			</section>
		{/if}
	</aside>
</div>
