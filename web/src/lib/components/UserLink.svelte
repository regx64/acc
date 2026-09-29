<script lang="ts">
	let { handle }: { handle: string | null | undefined } = $props();
</script>

{#if handle}
	<a class="link font-medium" href="/u/{handle}">{handle}</a>
{:else}
	<span class="text-faint">탈퇴한 사용자</span>
{/if}
