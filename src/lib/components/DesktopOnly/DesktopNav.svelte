<script lang="ts">
	import { PLATFORM } from '$lib/constants';
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher();

	export let currentView = 'chat';

	const views = [
		{ id: 'chat', label: 'Chat', icon: '💬' },
		{ id: 'instances', label: 'Instances', icon: '🔧', desktopOnly: true },
		{ id: 'settings', label: 'Settings', icon: '⚙️' }
	];

	const handleViewChange = (viewId: string) => {
		currentView = viewId;
		dispatch('viewChange', { view: viewId });
	};
</script>

{#if PLATFORM.isDesktop}
	<nav class="desktop-nav">
		<div class="nav-content">
			<div class="nav-brand">
				<span class="brand-icon">🤖</span>
				<span class="brand-text">OpenWebUI</span>
				<span class="brand-badge">Desktop</span>
			</div>

			<div class="nav-items">
				{#each views as view}
					{#if !view.desktopOnly || PLATFORM.isDesktop}
						<button
							class="nav-item"
							class:active={currentView === view.id}
							on:click={() => handleViewChange(view.id)}
						>
							<span class="item-icon">{view.icon}</span>
							<span class="item-label">{view.label}</span>
						</button>
					{/if}
				{/each}
			</div>
		</div>
	</nav>
{/if}

<style>
	.desktop-nav {
		background: #111827;
		color: white;
		border-bottom: 1px solid #1f2937;
	}

	.nav-content {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	.nav-brand {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.brand-icon {
		font-size: 1.5rem;
	}

	.brand-text {
		font-size: 1.125rem;
		font-weight: 600;
	}

	.brand-badge {
		background: #6366f1;
		color: white;
		font-size: 0.625rem;
		padding: 0.125rem 0.375rem;
		border-radius: 9999px;
		font-weight: 500;
		text-transform: uppercase;
	}

	.nav-items {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: transparent;
		border: none;
		color: #9ca3af;
		border-radius: 0.375rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.nav-item:hover {
		background: #374151;
		color: white;
	}

	.nav-item.active {
		background: #6366f1;
		color: white;
	}

	.item-icon {
		font-size: 1.125rem;
	}

	.item-label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
