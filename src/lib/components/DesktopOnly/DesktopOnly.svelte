<script lang="ts">
	import { PLATFORM } from '$lib/constants';

	/**
	 * Wrapper component to display content only on desktop
	 * @slot - Default slot, displayed only on desktop
	 * @slot fallback - Optional fallback slot, displayed on non-desktop platforms
	 */
	let slotType: 'default' | 'fallback' = PLATFORM.isDesktop ? 'default' : 'fallback';
</script>

{#if PLATFORM.isDesktop}
	<slot />
{:else}
	<slot name="fallback" />
{/if}
