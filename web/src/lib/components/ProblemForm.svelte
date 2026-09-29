<script lang="ts">
	import CodeEditor from './CodeEditor.svelte';
	import Markdown from './Markdown.svelte';
	import Badge from './Badge.svelte';
	import { LANGUAGE_LABEL } from '$lib/format';
	import { label } from '$lib/levels';
	import type { components } from '$lib/api/schema';
	import { untrack } from 'svelte';

	type Input = components['schemas']['ProblemInput'];
	let { initial, disabled = false }: { initial: Partial<Input>; disabled?: boolean } = $props();
	// The form owns its fields after the first render; parents remount it to reset.
	const init = untrack(() => initial);

	let title = $state(init.title ?? '');
	let timeLimit = $state(init.time_limit_ms ?? 1000);
	let memoryLimit = $state(init.memory_limit_mb ?? 256);
	let proposed = $state(init.proposed_level ?? 0);
	let legend = $state(init.statement?.legend ?? '');
	let input = $state(init.statement?.input ?? '');
	let output = $state(init.statement?.output ?? '');
	let hint = $state(init.statement?.hint ?? '');
	let samples = $state(init.statement?.samples?.length ? init.statement.samples.map((s) => ({ ...s })) : [{ input: '', output: '' }]);
	let solutionLanguage = $state(init.solution_language ?? 'cpp17');
	let solutionCode = $state(init.solution_code ?? '');
	let preview = $state(false);

	const statementJson = $derived(JSON.stringify({ legend, input, output, hint: hint.trim() ? hint : null, samples }));
</script>

<fieldset class="grid gap-5" {disabled}>
	<input type="hidden" name="statement" value={statementJson} />
	<div class="grid gap-4 sm:grid-cols-[1fr_8rem_8rem_9rem]">
		<div>
			<label class="label" for="title">제목</label>
			<input class="input" id="title" name="title" maxlength="60" required bind:value={title} />
		</div>
		<div>
			<label class="label" for="time">시간 제한 (ms)</label>
			<input class="input tabular" id="time" name="time_limit_ms" type="number" min="500" max="10000" step="100" required bind:value={timeLimit} />
		</div>
		<div>
			<label class="label" for="mem">메모리 (MB)</label>
			<input class="input tabular" id="mem" name="memory_limit_mb" type="number" min="32" max="1024" required bind:value={memoryLimit} />
		</div>
		<div>
			<label class="label" for="level">제안 레벨</label>
			<div class="flex items-center gap-2">
				<input class="input tabular" id="level" name="proposed_level" type="number" min="0" max="30" bind:value={proposed} />
				<Badge level={proposed} />
			</div>
			<p class="mt-1 text-xs text-faint">{label(proposed)} · 관리자가 확정</p>
		</div>
	</div>

	<div class="flex items-center justify-between">
		<p class="text-sm text-muted">Markdown과 KaTeX 수식($…$, $$…$$)을 쓸 수 있습니다. 답이 여러 개인 문제는 받지 않으며, 실수 출력은 자릿수를 정해 주세요.</p>
		<button type="button" class="btn py-1 text-sm" onclick={() => (preview = !preview)}>{preview ? '편집' : '미리보기'}</button>
	</div>

	{#if preview}
		<div class="card grid gap-6 p-6">
			<h2 class="text-xl font-semibold">{title}</h2>
			<section><h3 class="mb-1 font-semibold">문제</h3><Markdown source={legend} /></section>
			<section><h3 class="mb-1 font-semibold">입력</h3><Markdown source={input} /></section>
			<section><h3 class="mb-1 font-semibold">출력</h3><Markdown source={output} /></section>
			{#each samples as s, i (i)}
				<div class="grid gap-3 md:grid-cols-2">
					<div class="min-w-0"><p class="mb-1 text-sm font-semibold">예제 입력 {i + 1}</p><pre class="sample">{s.input}</pre></div>
					<div class="min-w-0"><p class="mb-1 text-sm font-semibold">예제 출력 {i + 1}</p><pre class="sample">{s.output}</pre></div>
				</div>
			{/each}
			{#if hint.trim()}<section><h3 class="mb-1 font-semibold">힌트</h3><Markdown source={hint} /></section>{/if}
		</div>
	{:else}
		<div>
			<label class="label" for="legend">문제</label>
			<textarea class="input min-h-48 font-sans" id="legend" required bind:value={legend}></textarea>
		</div>
		<div class="grid gap-4 md:grid-cols-2">
			<div>
				<label class="label" for="input">입력</label>
				<textarea class="input font-sans" id="input" required bind:value={input}></textarea>
			</div>
			<div>
				<label class="label" for="output">출력</label>
				<textarea class="input font-sans" id="output" required bind:value={output}></textarea>
			</div>
		</div>
		<div class="grid gap-3">
			<p class="label">예제 (채점에도 자동으로 포함됩니다)</p>
			{#each samples as s, i (i)}
				<div class="grid gap-3 md:grid-cols-[1fr_1fr_auto]">
					<textarea class="input min-h-20" placeholder="예제 입력 {i + 1}" aria-label="예제 입력 {i + 1}" bind:value={s.input}></textarea>
					<textarea class="input min-h-20" placeholder="예제 출력 {i + 1}" aria-label="예제 출력 {i + 1}" bind:value={s.output}></textarea>
					<button type="button" class="btn self-start" disabled={samples.length === 1} onclick={() => samples.splice(i, 1)}>삭제</button>
				</div>
			{/each}
			<div><button type="button" class="btn text-sm" disabled={samples.length >= 10} onclick={() => samples.push({ input: '', output: '' })}>예제 추가</button></div>
		</div>
		<div>
			<label class="label" for="hint">힌트 (선택)</label>
			<textarea class="input min-h-20 font-sans" id="hint" bind:value={hint}></textarea>
		</div>
	{/if}

	<div class="grid gap-2">
		<div class="flex items-end gap-3">
			<div>
				<label class="label" for="sol-lang">정해 언어</label>
				<select class="input w-40" id="sol-lang" name="solution_language" bind:value={solutionLanguage}>
					{#each Object.entries(LANGUAGE_LABEL) as [id, name] (id)}<option value={id}>{name}</option>{/each}
				</select>
			</div>
			<p class="pb-2 text-xs text-faint">정해는 모든 테스트를 시간 제한의 절반 안에 통과해야 합니다.</p>
		</div>
		<CodeEditor bind:value={solutionCode} language={solutionLanguage} name="solution_code" minHeight="14rem" />
	</div>
</fieldset>
