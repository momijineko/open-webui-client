<script lang="ts">
	import { PLATFORM } from '$lib/constants';
	import { onMount } from 'svelte';
	import { backendCommands } from '$lib/utils/tauri';

	export let showBackendControls = false;

	let backendStatus: {
		is_running: boolean;
		port: number | null;
		pid: number | null;
	} | null = null;

	let isLoading = false;

	const checkStatus = async () => {
		if (PLATFORM.isDesktop) {
			isLoading = true;
			try {
				backendStatus = await backendCommands.checkStatus();
			} catch (error) {
				console.error('Failed to check backend status:', error);
			} finally {
				isLoading = false;
			}
		}
	};

	const toggleBackend = async () => {
		if (!backendStatus) return;

		try {
			if (backendStatus.is_running) {
				await backendCommands.stop();
			} else {
				await backendCommands.start();
			}
			// 延迟一下再检查状态，让后端有时间启动/停止
			setTimeout(() => checkStatus(), 1000);
		} catch (error) {
			console.error('Failed to toggle backend:', error);
			alert('操作失败：' + (error as Error).message);
		}
	};

	onMount(() => {
		if (PLATFORM.isDesktop) {
			checkStatus();
			// 每 5 秒检查一次状态
			const interval = setInterval(checkStatus, 5000);
			return () => clearInterval(interval);
		}
	});
</script>

{#if PLATFORM.isDesktop && showBackendControls}
	<div class="desktop-statusbar">
		<div class="statusbar-content">
			<div class="backend-status">
				<span class="status-label">后端状态：</span>
				{#if isLoading}
					<span class="status-indicator loading">检查中...</span>
				{:else if backendStatus?.is_running}
					<span class="status-indicator running">
						<span class="dot"></span>
						运行中
						{#if backendStatus.port}
							(端口: {backendStatus.port})
						{/if}
					</span>
				{:else}
					<span class="status-indicator stopped">
						<span class="dot"></span>
						已停止
					</span>
				{/if}
			</div>

			<button
				class="toggle-btn"
				on:click={toggleBackend}
				disabled={isLoading || !backendStatus}
			>
				{backendStatus?.is_running ? '停止后端' : '启动后端'}
			</button>
		</div>
	</div>
{/if}

<style>
	.desktop-statusbar {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		background: #1f2937;
		color: white;
		padding: 0.5rem 1rem;
		z-index: 1000;
		border-top: 1px solid #374151;
	}

	.statusbar-content {
		display: flex;
		align-items: center;
		justify-content: space-between;
		max-width: 1200px;
		margin: 0 auto;
	}

	.backend-status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.status-label {
		font-size: 0.875rem;
		color: #9ca3af;
	}

	.status-indicator {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.status-indicator .dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		animation: pulse 2s ease-in-out infinite;
	}

	.status-indicator.running {
		color: #10b981;
	}

	.status-indicator.running .dot {
		background: #10b981;
	}

	.status-indicator.stopped {
		color: #ef4444;
	}

	.status-indicator.stopped .dot {
		background: #ef4444;
		animation: none;
	}

	.status-indicator.loading {
		color: #f59e0b;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	.toggle-btn {
		padding: 0.5rem 1rem;
		background: #374151;
		color: white;
		border: 1px solid #4b5563;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.toggle-btn:hover:not(:disabled) {
		background: #4b5563;
		border-color: #6b7280;
	}

	.toggle-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
