<script lang="ts">
	import { enhance } from '$app/forms';
	import FormMessage from '$lib/components/FormMessage.svelte';
	import { date, LANGUAGE_LABEL } from '$lib/format';

	let { data, form } = $props();
	const u = $derived(data.user!);
	const nextHandleChange = $derived(
		u.handle_changed_at ? new Date(new Date(u.handle_changed_at).getTime() + 90 * 86400_000) : null
	);
	const msg = (section: string) => (form?.section === section ? form : null);
</script>

<svelte:head><title>설정 · acc</title></svelte:head>

<h1 class="mb-6 text-2xl font-semibold">설정</h1>

<div class="grid max-w-2xl gap-6">
	<section class="card p-6">
		<h2 class="mb-1 font-semibold">계정</h2>
		<p class="text-sm text-muted">{u.email} · {u.email_verified ? '인증됨' : '인증 전'}</p>
	</section>

	<form method="POST" action="?/language" use:enhance={() => async ({ update }) => update({ reset: false })} class="card grid gap-3 p-6">
		<h2 class="font-semibold">기본 언어</h2>
		<FormMessage form={msg('language')} />
		<select class="input w-48" name="default_language" aria-label="기본 언어">
			<option value="">지정 안 함</option>
			{#each Object.entries(LANGUAGE_LABEL) as [id, name] (id)}<option value={id} selected={u.default_language === id}>{name}</option>{/each}
		</select>
		<div><button class="btn">저장</button></div>
	</form>

	<form method="POST" action="?/password" use:enhance class="card grid gap-3 p-6">
		<h2 class="font-semibold">비밀번호 바꾸기</h2>
		<FormMessage form={msg('password')} />
		<input class="input" type="password" name="current" placeholder="현재 비밀번호" autocomplete="current-password" required aria-label="현재 비밀번호" />
		<input class="input" type="password" name="new" placeholder="새 비밀번호 (8자 이상)" minlength="8" autocomplete="new-password" required aria-label="새 비밀번호" />
		<p class="text-xs text-faint">바꾸면 이 기기를 뺀 모든 기기에서 로그아웃됩니다.</p>
		<div><button class="btn">바꾸기</button></div>
	</form>

	<form method="POST" action="?/handle" use:enhance class="card grid gap-3 p-6">
		<h2 class="font-semibold">핸들 바꾸기</h2>
		<FormMessage form={msg('handle')} />
		<input class="input font-mono" name="handle" value={u.handle} pattern="[a-z0-9_]{'{'}3,20{'}'}" required aria-label="새 핸들" />
		<p class="text-xs text-faint">
			90일에 한 번 바꿀 수 있습니다{nextHandleChange && nextHandleChange > new Date() ? ` (다음 변경 가능일 ${date(nextHandleChange.toISOString())})` : ''}.
			예전 핸들은 180일 동안 잠기고 새 핸들로 연결됩니다.
		</p>
		<div><button class="btn">바꾸기</button></div>
	</form>

	<form method="POST" action="?/delete" use:enhance class="card grid gap-3 border-bad/40 p-6">
		<h2 class="font-semibold text-bad">탈퇴</h2>
		<FormMessage form={msg('delete')} />
		<p class="text-sm text-muted">
			이메일 등 개인정보는 바로 지웁니다. 제출과 글은 "탈퇴한 사용자"로 남고, 랭킹에서 빠지며, 핸들은 영구히 다시 쓸 수 없습니다.
		</p>
		<input class="input" type="password" name="password" placeholder="비밀번호" autocomplete="current-password" required aria-label="비밀번호" />
		<input class="input" name="confirm" placeholder='확인을 위해 "탈퇴" 입력' required aria-label="확인 문구" />
		<div><button class="btn btn-danger">탈퇴하기</button></div>
	</form>
</div>
