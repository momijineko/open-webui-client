<script lang="ts">
	import { onMount } from 'svelte';
	import { PLATFORM } from '$lib/constants';
	import { tauriCommands, isTauriAvailable, type BackendStatus } from '$lib/utils/tauri';

	let backendStatus: BackendStatus | null = null;
	let statusMessage = '点击按钮检查状态';
	let isLoading = false;

	// Auto test state
	let autoTestRunning = false;
	let autoTestResults: Array<{ name: string; status: 'pass' | 'fail' | 'skip'; message: string; duration?: number }> = [];
	let autoTestProgress = 0;

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

	// Auto test runner
	const runAutoTest = async () => {
		autoTestRunning = true;
		autoTestResults = [];
		autoTestProgress = 0;

		const tests = [
			{
				name: 'Tauri 可用性检测',
				run: async () => {
					const available = isTauriAvailable();
					return { pass: available, message: available ? 'Tauri 可用' : 'Tauri 不可用（非桌面环境）' };
				}
			},
			{
				name: '平台检测',
				run: async () => {
					return { pass: true, message: `Desktop: ${PLATFORM.isDesktop}, Mobile: ${PLATFORM.isMobile}, Web: ${PLATFORM.isWeb}` };
				}
			},
			{
				name: '后端状态检查',
				run: async () => {
					try {
						const status = await tauriCommands.backend.checkStatus();
						backendStatus = status;
						return { pass: true, message: `运行中: ${status.is_running}, 端口: ${status.port || 'N/A'}` };
					} catch (error) {
						return { pass: false, message: `错误: ${(error as Error).message}` };
					}
				}
			},
			{
				name: '获取应用配置',
				run: async () => {
					try {
						const config = await tauriCommands.config.get();
						return { pass: true, message: `安装完成: ${config.setupCompleted || false}` };
					} catch (error) {
						return { pass: false, message: `错误: ${(error as Error).message}` };
					}
				}
			},
			{
				name: '获取下载目录',
				run: async () => {
					try {
						const dir = await tauriCommands.download.getDir();
						return { pass: true, message: `目录: ${dir}` };
					} catch (error) {
						return { pass: false, message: `错误: ${(error as Error).message}` };
					}
				}
			},
			{
				name: '检查后端安装',
				run: async () => {
					try {
						const result = await tauriCommands.backend.checkInstallation();
						return { pass: true, message: `已安装: ${result.installed}, 路径: ${result.path || 'N/A'}` };
					} catch (error) {
						return { pass: false, message: `错误: ${(error as Error).message}` };
					}
				}
			},
			{
				name: '启动后端（测试）',
				run: async () => {
					try {
						// 如果后端已在运行，跳过
						const currentStatus = await tauriCommands.backend.checkStatus();
						if (currentStatus.is_running) {
							return { pass: true, message: '后端已在运行，跳过启动测试', skip: true };
						}
						const result = await tauriCommands.backend.start();
						// Wait a bit for backend to start
						await new Promise(r => setTimeout(r, 2000));
						const newStatus = await tauriCommands.backend.checkStatus();
						return { pass: newStatus.is_running, message: `启动成功: ${result}` };
					} catch (error) {
						return { pass: false, message: `错误: ${(error as Error).message}` };
					}
				}
			}
		];

		for (let i = 0; i < tests.length; i++) {
			const test = tests[i];
			autoTestProgress = ((i + 1) / tests.length) * 100;

			const startTime = performance.now();
			const result = await test.run();
			const duration = Math.round(performance.now() - startTime);

			autoTestResults.push({
				name: test.name,
				status: result.skip ? 'skip' : (result.pass ? 'pass' : 'fail'),
				message: result.message,
				duration
			});

			// Small delay between tests
			await new Promise(r => setTimeout(r, 500));
		}

		autoTestRunning = false;
		autoTestProgress = 100;

		// Update summary
		const passed = autoTestResults.filter(r => r.status === 'pass').length;
		const failed = autoTestResults.filter(r => r.status === 'fail').length;
		const skipped = autoTestResults.filter(r => r.status === 'skip').length;
		statusMessage = `测试完成: ${passed} 通过, ${failed} 失败, ${skipped} 跳过`;
	};

	const getAutoTestSummary = () => {
		const passed = autoTestResults.filter(r => r.status === 'pass').length;
		const failed = autoTestResults.filter(r => r.status === 'fail').length;
		const skipped = autoTestResults.filter(r => r.status === 'skip').length;
		const total = autoTestResults.length;
		return { total, passed, failed, skipped };
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
			<button on:click={testTauriAvailability} disabled={isLoading || autoTestRunning}>
				检测 Tauri 可用性
			</button>
			<button on:click={checkPlatform} disabled={isLoading || autoTestRunning}>
				显示平台信息
			</button>
			<button on:click={testBackendStatus} disabled={isLoading || autoTestRunning}>
				检查后端状态
			</button>
			<button on:click={testBackendStart} disabled={isLoading || autoTestRunning}>
				启动后端
			</button>
			<button on:click={testBackendStop} disabled={isLoading || autoTestRunning}>
				停止后端
			</button>
		</div>
	</div>

	<!-- Auto Test Panel -->
	<div class="auto-test-panel">
		<div class="auto-test-header">
			<h2>自动化测试</h2>
			<button
				class="run-auto-test-btn"
				on:click={runAutoTest}
				disabled={autoTestRunning || isLoading}
				class:running={autoTestRunning}
			>
				{#if autoTestRunning}
					⏳ 测试中... ({Math.round(autoTestProgress)}%)
				{:else}
					▶️ 运行自动化测试
				{/if}
			</button>
		</div>

		{#if autoTestResults.length > 0}
			<div class="test-results">
				<div class="test-summary">
					{#if autoTestRunning}
						<div class="progress-bar">
							<div class="progress-fill" style="width: {autoTestProgress}%"></div>
						</div>
					{/if}
					<div class="summary-stats">
						<span class="stat total">总计: {getAutoTestSummary().total}</span>
						<span class="stat pass">通过: {getAutoTestSummary().passed}</span>
						<span class="stat fail">失败: {getAutoTestSummary().failed}</span>
						<span class="stat skip">跳过: {getAutoTestSummary().skipped}</span>
					</div>
				</div>

				<div class="test-list">
					{#each autoTestResults as result (result.name)}
						<div class="test-item {result.status}">
							<span class="test-icon">
								{#if result.status === 'pass'}✅
								{:else if result.status === 'fail'}❌
								{:else}⏭️{/if}
							</span>
							<span class="test-name">{result.name}</span>
							<span class="test-message">{result.message}</span>
							{#if result.duration}
								<span class="test-duration">{result.duration}ms</span>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/if}
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
	html, body {
		height: auto;
		min-height: 100vh;
		overflow-y: auto;
	}

	.test-container {
		max-width: 800px;
		margin: 2rem auto;
		padding: 2rem;
		background: #f9fafb;
		border-radius: 1rem;
		font-family: system-ui, -apple-system, sans-serif;
		min-height: 100vh;
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

	/* Auto Test Panel Styles */
	.auto-test-panel {
		background: white;
		padding: 1.5rem;
		border-radius: 0.5rem;
		margin-bottom: 1.5rem;
		border: 1px solid #e5e7eb;
	}

	.auto-test-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
		flex-wrap: wrap;
		gap: 1rem;
	}

	.run-auto-test-btn {
		padding: 0.75rem 1.5rem;
		background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
		color: white;
		border: none;
		border-radius: 0.5rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.3s ease;
		box-shadow: 0 2px 8px rgba(99, 102, 241, 0.3);
	}

	.run-auto-test-btn:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
	}

	.run-auto-test-btn.running {
		background: linear-gradient(135deg, #10b981 0%, #059669 100%);
		animation: pulse 2s infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.8; }
	}

	.test-results {
		margin-top: 1rem;
	}

	.test-summary {
		margin-bottom: 1rem;
	}

	.progress-bar {
		height: 8px;
		background: #e5e7eb;
		border-radius: 4px;
		overflow: hidden;
		margin-bottom: 1rem;
	}

	.progress-fill {
		height: 100%;
		background: linear-gradient(90deg, #6366f1 0%, #8b5cf6 100%);
		transition: width 0.3s ease;
	}

	.summary-stats {
		display: flex;
		gap: 1.5rem;
		flex-wrap: wrap;
	}

	.stat {
		padding: 0.5rem 1rem;
		border-radius: 0.375rem;
		font-weight: 600;
		font-size: 0.875rem;
	}

	.stat.total {
		background: #e5e7eb;
		color: #374151;
	}

	.stat.pass {
		background: #d1fae5;
		color: #065f46;
	}

	.stat.fail {
		background: #fee2e2;
		color: #991b1b;
	}

	.stat.skip {
		background: #fef3c7;
		color: #92400e;
	}

	.test-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.test-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 1rem;
		border-radius: 0.375rem;
		background: #f9fafb;
		border-left: 3px solid transparent;
		transition: all 0.2s;
	}

	.test-item:hover {
		background: #f3f4f6;
	}

	.test-item.pass {
		border-left-color: #10b981;
	}

	.test-item.fail {
		border-left-color: #ef4444;
		background: #fef2f2;
	}

	.test-item.skip {
		border-left-color: #f59e0b;
	}

	.test-icon {
		font-size: 1.25rem;
		flex-shrink: 0;
	}

	.test-name {
		font-weight: 600;
		color: #1f2937;
		flex-shrink: 0;
		min-width: 150px;
	}

	.test-message {
		flex: 1;
		color: #6b7280;
		font-size: 0.875rem;
		font-family: monospace;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.test-duration {
		color: #9ca3af;
		font-size: 0.75rem;
		font-family: monospace;
		flex-shrink: 0;
	}
</style>
