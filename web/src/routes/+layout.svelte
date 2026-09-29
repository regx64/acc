<script lang="ts">
	import '../app.css';
	import 'katex/dist/katex.min.css';
	import { page } from '$app/state';
	import Badge from '$lib/components/Badge.svelte';

	let { data, children } = $props();

	const nav = [
		{ href: '/problems', label: '문제' },
		{ href: '/status', label: '채점 현황' },
		{ href: '/ranking', label: '랭킹' },
		{ href: '/create', label: '출제' }
	];
	const active = (href: string) => page.url.pathname === href || page.url.pathname.startsWith(href + '/');
</script>

<svelte:head>
	<title>acc</title>
</svelte:head>

<div class="flex min-h-screen flex-col">
	{#if data.site.banner}
		<div class="bg-warn/15 px-4 py-2 text-center text-sm text-fg" role="status">{data.site.banner}</div>
	{/if}

	<header class="sticky top-0 z-20 border-b border-line bg-bg/85 backdrop-blur">
		<div class="mx-auto flex max-w-6xl items-center gap-4 px-4 py-2.5">
			<a href="/" class="font-mono text-lg font-semibold tracking-tight text-fg no-underline">acc<span class="text-accent">_</span></a>
			<nav class="flex min-w-0 flex-1 gap-1 overflow-x-auto" aria-label="주 메뉴">
				{#each nav as n (n.href)}
					<a
						href={n.href}
						class="rounded-md px-2.5 py-1 text-sm whitespace-nowrap no-underline {active(n.href) ? 'bg-surface-2 text-fg' : 'text-muted hover:text-fg'}"
						>{n.label}</a
					>
				{/each}
				{#if data.user?.role === 'ADMIN'}
					<a href="/admin" class="rounded-md px-2.5 py-1 text-sm whitespace-nowrap no-underline {active('/admin') ? 'bg-surface-2 text-fg' : 'text-muted hover:text-fg'}">관리</a>
				{/if}
			</nav>
			<div class="flex shrink-0 items-center gap-2 text-sm">
				{#if data.user}
					<a href="/u/{data.user.handle}" class="flex items-center gap-1.5 font-medium text-fg no-underline">
						<Badge level={data.user.tier} size="xs" />
						{data.user.handle}
					</a>
					<a href="/settings" class="hidden text-muted no-underline hover:text-fg sm:inline">설정</a>
					<form method="POST" action="/logout">
						<button class="text-muted hover:text-fg">로그아웃</button>
					</form>
				{:else}
					<a href="/login?next={encodeURIComponent(page.url.pathname + page.url.search)}" class="btn py-1">로그인</a>
					<a href="/signup" class="btn btn-primary py-1">가입</a>
				{/if}
			</div>
		</div>
	</header>

	{#if data.user && !data.user.email_verified}
		<div class="border-b border-line bg-accent/10 px-4 py-2 text-center text-sm">
			이메일 인증 후에 제출과 글쓰기를 할 수 있습니다. 받은 메일의 링크를 여세요.
			<form method="POST" action="/verify-email?/resend" class="inline">
				<button class="link ml-1 font-medium">인증 메일 다시 보내기</button>
			</form>
		</div>
	{/if}
	{#if data.user?.suspended}
		<div class="border-b border-line bg-bad/10 px-4 py-2 text-center text-sm text-bad">
			정지된 계정입니다. 제출과 글쓰기를 할 수 없습니다. 사유: {data.user.suspend_reason ?? '없음'}
		</div>
	{/if}

	<main class="mx-auto w-full max-w-6xl flex-1 px-4 py-8">
		{@render children()}
	</main>

	<footer class="border-t border-line">
		<div class="mx-auto flex max-w-6xl flex-wrap justify-between gap-3 px-4 py-6 text-xs text-faint">
			<span>acc · 코드 MIT · 문제 지문 CC BY-SA 4.0</span>
			<span class="flex gap-4">
				<a href="/terms" class="hover:text-fg">이용약관</a>
				<a href="/privacy" class="font-semibold hover:text-fg">개인정보처리방침</a>
				<a href="/authoring-terms" class="hover:text-fg">출제 약관</a>
				<a href="https://github.com/regx64/acc" class="hover:text-fg" rel="noopener">GitHub</a>
			</span>
		</div>
	</footer>
</div>
