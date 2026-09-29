<script lang="ts">
	import { page } from '$app/state';
	let { children } = $props();
	const tabs = [
		{ href: '/admin', label: '현황' },
		{ href: '/admin/problems', label: '문제·검수' },
		{ href: '/admin/users', label: '사용자' },
		{ href: '/admin/reports', label: '신고' },
		{ href: '/admin/typos', label: '오타 제보' }
	];
</script>

<nav class="mb-6 flex gap-1 overflow-x-auto border-b border-line" aria-label="관리 메뉴">
	{#each tabs as t (t.href)}
		<a
			href={t.href}
			class="-mb-px border-b-2 px-3 py-2 text-sm whitespace-nowrap no-underline {page.url.pathname === t.href ? 'border-accent text-fg' : 'border-transparent text-muted hover:text-fg'}"
			>{t.label}</a
		>
	{/each}
</nav>
{@render children()}
