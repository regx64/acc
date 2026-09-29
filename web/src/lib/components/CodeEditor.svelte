<script lang="ts">
	import { onMount } from 'svelte';
	import type { EditorView } from '@codemirror/view';
	import type { Compartment } from '@codemirror/state';

	let {
		value = $bindable(''),
		language = 'cpp17',
		readonly = false,
		name = undefined,
		minHeight = '22rem'
	}: { value?: string; language?: string; readonly?: boolean; name?: string; minHeight?: string } = $props();

	let host: HTMLDivElement;
	let view: EditorView | undefined;
	let langSlot: Compartment | undefined;
	let loadLang: ((id: string) => Promise<unknown>) | undefined;
	let ready = $state(false);

	onMount(() => {
		let destroyed = false;
		(async () => {
			const [{ basicSetup }, viewMod, stateMod, commands] = await Promise.all([
				import('codemirror'),
				import('@codemirror/view'),
				import('@codemirror/state'),
				import('@codemirror/commands')
			]);
			if (destroyed) return;
			loadLang = async (id: string) => {
				if (id === 'python3') return (await import('@codemirror/lang-python')).python();
				if (id === 'java') return (await import('@codemirror/lang-java')).java();
				if (id === 'rust') return (await import('@codemirror/lang-rust')).rust();
				return (await import('@codemirror/lang-cpp')).cpp();
			};
			langSlot = new stateMod.Compartment();
			const theme = viewMod.EditorView.theme({
				'&': { backgroundColor: 'var(--surface)', color: 'var(--fg)', fontSize: '13.5px', minHeight },
				'.cm-scroller': { fontFamily: 'var(--font-mono)', minHeight },
				'.cm-gutters': { backgroundColor: 'var(--surface-2)', color: 'var(--faint)', borderRight: '1px solid var(--line)' },
				'.cm-activeLine, .cm-activeLineGutter': { backgroundColor: 'color-mix(in srgb, var(--accent) 7%, transparent)' },
				'&.cm-focused': { outline: 'none' },
				'.cm-cursor': { borderLeftColor: 'var(--fg)' }
			});
			view = new viewMod.EditorView({
				parent: host,
				state: stateMod.EditorState.create({
					doc: value,
					extensions: [
						basicSetup,
						viewMod.keymap.of([commands.indentWithTab]),
						theme,
						langSlot.of((await loadLang(language)) as never),
						stateMod.EditorState.readOnly.of(readonly),
						viewMod.EditorView.updateListener.of((u) => {
							if (u.docChanged) value = u.state.doc.toString();
						})
					]
				})
			});
			ready = true;
		})();
		return () => {
			destroyed = true;
			view?.destroy();
		};
	});

	$effect(() => {
		const id = language;
		if (!view || !langSlot || !loadLang) return;
		loadLang(id).then((ext) => view?.dispatch({ effects: langSlot!.reconfigure(ext as never) }));
	});

	$effect(() => {
		// Outside changes (e.g. loading a template) flow into the editor.
		if (view && value !== view.state.doc.toString()) {
			view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } });
		}
	});
</script>

<div class="overflow-hidden rounded-lg border border-line-strong" class:hidden={!ready} bind:this={host}></div>
{#if !ready}
	<textarea class="input" style="min-height: {minHeight}" bind:value {readonly} spellcheck="false" aria-label="코드"></textarea>
{/if}
{#if name}<textarea {name} hidden readonly {value}></textarea>{/if}
