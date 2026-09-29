<script lang="ts">
	import { LANGUAGE_LABEL, STATUS_LABEL, STATUS_ORDER } from '$lib/format';
	let { query, problemField = true }: { query: Record<string, string | undefined>; problemField?: boolean } = $props();
</script>

<form method="GET" class="mb-4 flex flex-wrap items-end gap-2 text-sm">
	{#if problemField}
		<div>
			<label class="label" for="f-problem">문제</label>
			<input class="input w-28 font-mono" id="f-problem" name="problem_id" inputmode="numeric" value={query.problem_id ?? ''} />
		</div>
	{/if}
	<div>
		<label class="label" for="f-handle">아이디</label>
		<input class="input w-36" id="f-handle" name="handle" value={query.handle ?? ''} />
	</div>
	<div>
		<label class="label" for="f-status">결과</label>
		<select class="input w-36" id="f-status" name="status">
			<option value="">전체</option>
			{#each STATUS_ORDER as s (s)}<option value={s} selected={query.status === s}>{STATUS_LABEL[s]}</option>{/each}
		</select>
	</div>
	<div>
		<label class="label" for="f-lang">언어</label>
		<select class="input w-32" id="f-lang" name="language">
			<option value="">전체</option>
			{#each Object.entries(LANGUAGE_LABEL) as [id, name] (id)}<option value={id} selected={query.language === id}>{name}</option>{/each}
		</select>
	</div>
	<button class="btn">검색</button>
</form>
