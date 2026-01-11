<script lang="ts">
	import { PLATFORM } from '$lib/constants';

	/**
	 * 仅在桌面端显示内容的包装组件
	 * @slot - 默认插槽，仅在桌面端显示
	 * @slot fallback - 可选的降级插槽，在非桌面端显示
	 */
	let slotType: 'default' | 'fallback' = PLATFORM.isDesktop ? 'default' : 'fallback';
</script>

{#if PLATFORM.isDesktop}
	<slot />
{:else}
	<slot name="fallback" />
{/if}
