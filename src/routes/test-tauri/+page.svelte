<script lang="ts">
	import { onMount } from 'svelte';
	import { PLATFORM } from '$lib/constants';
	import { tauriCommands, isTauriAvailable, type BackendStatus } from '$lib/utils/tauri';

	let backendStatus: BackendStatus | null = null;
	let statusMessage = '点击按钮检查状态';
	let isLoading = false;

	const testTauriAvailability = () => {
		if (isTauriAvailable()) {
			statusMessage = '✅ Tauri 可用';
		} else {
			statusMessage = '❌ Tauri 不可用';
		}
	};

	const checkPlatform = () => {
		statusMessage = `平台信息: ${JSON.stringify(PLATFORM, null, 2)}`;
	};

	const testBackendStatus = async () => {
		isLoading = true;
		statusMessage = '正在检查后端状态...';

		try {
			backendStatus = await tauriCommands.backend.checkStatus();
			statusMessage = `后端状态: ${JSON.stringify(backendStatus, null, 2)}`;
		} catch (error) {
			statusMessage = `错误: ${(error as Error).message}`;
			console.error('Backend status check failed:', error);
		} finally {
			isLoading = false;
		}
	};

	const testBackendStart = async () => {
		isLoading = true;
		statusMessage = '正在启动后端...';

		try {
			const result = await tauriCommands.backend.start();
			statusMessage = `启动结果: ${result}`;
		} catch (error) {
			statusMessage = `错误: ${(error as Error).message}`;
			console.error('Backend start failed:', error);
		} finally {
			isLoading = false;
		}
	};

	const testBackendStop = async () => {
		isLoading = true;
		statusMessage = '正在停止后端...';

		try {
			const result = await tauriCommands.backend.stop();
			statusMessage = `停止结果: ${result}`;
		} catch (error) {
			statusMessage = `错误: ${(error as Error).message}`;
			console.error('Backend stop failed:', error);
		} finally {
			isLoading = false;
		}
	};

	onMount(() => {
		// 自动检测平台
		testTauriAvailability();
	});
</script>

<div class="test-container">
	<h1>Tauri 集成测试页面</h1>

	<div class="info-panel">
		<h2>环境信息</h2>
		<div class="info-item">
			<span class="label">isDesktop:</span>
			<span class="value">{String(PLATFORM.isDesktop)}</span>
		</div>
		<div class="info-item">
			<span class="label">isMobile:</span>
			<span class="value">{String(PLATFORM.isMobile)}</span>
		</div>
		<div class="info-item">
			<span class="label">isWeb:</span>
			<span class="value">{String(PLATFORM.isWeb)}</span>
		</div>
		<div class="info-item">
			<span class="label">Tauri Available:</span>
			<span class="value">{String(isTauriAvailable())}</span>
		</div>
	</div>

	<div class="status-panel">
		<h2>状态消息</h2>
		<div class="status-message">
			{#if isLoading}
				<span class="loading">⏳ 加载中...</span>
			{:else}
				<pre>{statusMessage}</pre>
			{/if}
		</div>
	</div>

	<div class="controls-panel">
		<h2>测试控制</h2>
		<div class="button-grid">
			<button on:click={testTauriAvailability} disabled={isLoading}>
				检测 Tauri 可用性
			</button>
			<button on:click={checkPlatform} disabled={isLoading}>
				显示平台信息
			</button>
			<button on:click={testBackendStatus} disabled={isLoading}>
				检查后端状态
			</button>
			<button on:click={testBackendStart} disabled={isLoading}>
				启动后端
			</button>
			<button on:click={testBackendStop} disabled={isLoading}>
				停止后端
			</button>
		</div>
	</div>

	{#if backendStatus}
		<div class="backend-status-panel">
			<h2>后端详细信息</h2>
			<div class="status-item">
				<span class="label">运行中:</span>
				<span class="value {backendStatus.is_running ? 'running' : 'stopped'}">
					{String(backendStatus.is_running)}
				</span>
			</div>
			<div class="status-item">
				<span class="label">端口:</span>
				<span class="value">{backendStatus.port || 'N/A'}</span>
			</div>
			<div class="status-item">
				<span class="label">进程 ID:</span>
				<span class="value">{backendStatus.pid || 'N/A'}</span>
			</div>
		</div>
	{/if}
</div>

<style>
	.test-container {
		max-width: 800px;
		margin: 2rem auto;
		padding: 2rem;
		background: #f9fafb;
		border-radius: 1rem;
		font-family: system-ui, -apple-system, sans-serif;
	}

	h1 {
		color: #1f2937;
		margin-bottom: 1.5rem;
	}

	h2 {
		color: #374151;
		font-size: 1.25rem;
		margin-bottom: 1rem;
	}

	.info-panel,
	.status-panel,
	.controls-panel,
	.backend-status-panel {
		background: white;
		padding: 1.5rem;
		border-radius: 0.5rem;
		margin-bottom: 1.5rem;
		border: 1px solid #e5e7eb;
	}

	.info-item,
	.status-item {
		display: flex;
		justify-content: space-between;
		padding: 0.5rem 0;
		border-bottom: 1px solid #f3f4f6;
	}

	.info-item:last-child,
	.status-item:last-child {
		border-bottom: none;
	}

	.label {
		font-weight: 500;
		color: #6b7280;
	}

	.value {
		color: #1f2937;
		font-family: monospace;
	}

	.value.running {
		color: #10b981;
		font-weight: 600;
	}

	.value.stopped {
		color: #ef4444;
		font-weight: 600;
	}

	.status-message {
		background: #f9fafb;
		padding: 1rem;
		border-radius: 0.375rem;
		min-height: 60px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.status-message pre {
		margin: 0;
		font-size: 0.875rem;
		color: #374151;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.loading {
		color: #6366f1;
		font-weight: 500;
	}

	.button-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 0.75rem;
	}

	button {
		padding: 0.75rem 1rem;
		background: #6366f1;
		color: white;
		border: none;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	button:hover:not(:disabled) {
		background: #4f46e5;
		transform: translateY(-1px);
	}

	button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		transform: none;
	}
</style>
