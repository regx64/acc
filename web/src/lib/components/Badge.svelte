<script lang="ts">
	import { groupIndex, step, longLabel, label } from '$lib/levels';

	let { level, size = 'sm', title = true }: { level: number; size?: 'xs' | 'sm' | 'md' | 'lg'; title?: boolean } = $props();

	const g = $derived(groupIndex(level));
	const s = $derived(step(level));
	const symbol = $derived(['?', 'ℕ', 'ℤ', 'ℚ', 'ℝ', 'ℂ', 'ℍ'][g]);
</script>

<span
	class="badge g{g} {size}"
	title={title ? `${longLabel(level)} (${label(level)})` : undefined}
	aria-label={longLabel(level)}
>
	<span class="face">{symbol}</span>
	{#if size !== 'xs'}
		<span class="pips" aria-hidden="true">
			{#each [1, 2, 3, 4, 5] as i (i)}<i class:on={i <= s}></i>{/each}
		</span>
	{/if}
</span>

<style>
	.badge {
		--c: var(--t-u);
		display: inline-grid;
		justify-items: center;
		gap: 2px;
		vertical-align: middle;
		line-height: 1;
	}
	.face {
		--s: 22px;
		width: var(--s);
		height: var(--s);
		display: grid;
		place-items: center;
		border-radius: calc(var(--s) * 0.24);
		font-family: var(--font-math);
		font-size: calc(var(--s) * 0.56);
		color: var(--c);
		border: 1.5px solid var(--c);
		background: var(--surface);
	}
	.xs .face { --s: 18px; }
	.md .face { --s: 36px; }
	.lg .face { --s: 72px; border-radius: 16px; }
	.pips {
		display: grid;
		grid-template-columns: repeat(5, 3px);
		gap: 1px;
	}
	.md .pips { grid-template-columns: repeat(5, 5px); gap: 2px; }
	.lg .pips { grid-template-columns: repeat(5, 11px); gap: 3px; }
	.pips i {
		height: 2px;
		border-radius: 1px;
		background: var(--line-strong);
	}
	.md .pips i { height: 3px; }
	.lg .pips i { height: 5px; }
	.pips i.on { background: var(--c); }

	/* Unrated: dashed, ? */
	.g0 .face { border-style: dashed; font-family: var(--font-sans); font-weight: 600; }
	/* ℕ thin border */
	.g1 { --c: var(--t-n); }
	/* ℤ thick border */
	.g2 { --c: var(--t-z); }
	.g2 .face { border-width: 2.5px; }
	.md.g2 .face, .lg.g2 .face { border-width: 4px; }
	/* ℚ double border */
	.g3 { --c: var(--t-q); }
	.g3 .face { border: 3.5px double var(--c); }
	.md.g3 .face { border-width: 5px; }
	.lg.g3 .face { border-width: 8px; }
	/* ℝ diagonal hatching */
	.g4 { --c: var(--t-r); }
	.g4 .face {
		border-width: 2px;
		background:
			repeating-linear-gradient(135deg, color-mix(in srgb, var(--c) 30%, transparent) 0 1.5px, transparent 1.5px 5px),
			var(--surface);
	}
	/* ℂ filled, white symbol */
	.g5 { --c: var(--t-c); }
	.g5 .face { background: var(--c); color: #fff; }
	/* ℍ filled + inner border (inverted in dark mode via --t-h) */
	.g6 { --c: var(--t-h); }
	.g6 .face { background: var(--c); color: var(--bg); box-shadow: inset 0 0 0 2px var(--c), inset 0 0 0 3px var(--bg); }
	.lg.g6 .face { box-shadow: inset 0 0 0 5px var(--c), inset 0 0 0 7.5px var(--bg); }
</style>
