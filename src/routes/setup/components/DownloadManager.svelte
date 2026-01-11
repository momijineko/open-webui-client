<script lang="ts">
	export let currentStep: number;
	export let config: any;
	export let onComplete: () => void;
	export let onBack: () => void;

	import { onMount, onDestroy } from 'svelte';
	import {
		tauriCommands,
		isTauriAvailable,
		listenDownloadProgress,
		listenDownloadStatus,
		listenDownloadComplete
	} from '$lib/utils/tauri';

	let components = [
		{
			id: 'python',
			name: 'Python 环境',
			description: 'Python 3.11 运行时环境（如系统已有则跳过）',
			size: '~25MB',
			required: true,
			installed: false,
			progress: 0,
			status: 'pending'
		},
		{
			id: 'backend',
			name: 'OpenWebUI 后端',
			description: '从本地源码构建并安装 Python 后端服务',
			size: '本地构建',
			required: true,
			installed: false,
			progress: 0,
			status: 'pending'
		}
	];

	let selectedComponents = ['python', 'backend'];
	let isInstalling = false;
	let installStatus = '准备安装...';
	let installingPython = false; // 是否正在安装 Python

	// Store unlisten functions
	let unlistenProgress: (() => void) | null = null;
	let unlistenStatus: (() => void) | null = null;
	let unlistenComplete: (() => void) | null = null;

	onMount(async () => {
		// Setup event listeners for installation progress
		if (isTauriAvailable()) {
			// 添加一个全局测试监听器（仅用于调试）
			try {
				const { listen } = await import('@tauri-apps/api/event');
				await listen('download-status', (event) => {
					console.log('=== GLOBAL download-status event ===', event);
					console.log('=== Payload ===', event.payload);
				});
				console.log('Global test listener registered');
			} catch (e) {
				console.error('Failed to register test listener:', e);
			}

			// Python 安装进度监听
			listenDownloadProgress((progress) => {
				console.log('=== Download progress event received ===', progress);
				console.log('=== progress.percentage ===', progress.percentage);
				console.log('=== installingPython ===', installingPython);

				const compIndex = components.findIndex((c) => c.id === (installingPython ? 'python' : 'backend'));
				console.log('=== Found component index ===', compIndex);

				if (compIndex !== -1) {
					console.log('=== component.progress before ===', components[compIndex].progress);
					// 使用 Svelte 的响应式更新方式：重新赋值整个数组
					components[compIndex] = {
						...components[compIndex],
						progress: progress.percentage
					};
					components = [...components]; // 触发响应式更新
					console.log('=== component.progress after ===', components[compIndex].progress);

					const statusText = installingPython
						? `安装 Python: ${Math.round(progress.percentage)}%`
						: `${progress.speed} ${Math.round(progress.percentage)}%`;
					installStatus = statusText;
					console.log('=== Updated installStatus ===', installStatus);
				}
			}).then((unlisten) => {
				console.log('download-progress listener registered');
				unlistenProgress = unlisten;
			}).catch((err) => {
				console.error('Failed to register download-progress listener:', err);
			});

			// 安装状态监听
			listenDownloadStatus((status) => {
				console.log('=== Installation status event received ===', status);
				console.log('=== status.status ===', status.status);
				console.log('=== installStatus before ===', installStatus);

				// 更新状态文本
				installStatus = status.status;

				console.log('=== installStatus after ===', installStatus);

				// 如果检测到正在安装 Python
				if (status.file?.includes('Python')) {
					console.log('=== Installing Python ===');
					installingPython = true;
					const pythonComp = components.find((c) => c.id === 'python');
					if (pythonComp) {
						pythonComp.status = 'installing';
						pythonComp.progress = 50;
					}
				} else if (status.file?.includes('后端')) {
					console.log('=== Installing Backend ===');
					installingPython = false;
					const pythonComp = components.find((c) => c.id === 'python');
					if (pythonComp) {
						pythonComp.installed = true;
						pythonComp.progress = 100;
					}
					const backendComp = components.find((c) => c.id === 'backend');
					if (backendComp) {
						backendComp.status = 'installing';
					}
				}

				// 强制触发响应式更新
				return Promise.resolve();
			}).then((unlisten) => {
				console.log('download-status listener registered');
				unlistenStatus = unlisten;
			}).catch((err) => {
				console.error('Failed to register download-status listener:', err);
			});

			// 安装完成监听
			listenDownloadComplete((result) => {
				console.log('Installation complete:', result);
				const backendComp = components.find((c) => c.id === 'backend');
				if (backendComp) {
					backendComp.installed = result.completed.includes(backendComp.name);
					backendComp.progress = 100;
				}
				isInstalling = false;
				installStatus = '安装完成！';
				onComplete();
			}).then((unlisten) => {
				unlistenComplete = unlisten;
			});
		}
	});

	onDestroy(() => {
		// Cleanup event listeners
		unlistenProgress?.();
		unlistenStatus?.();
		unlistenComplete?.();
	});

	const startInstallation = async () => {
		console.log('=== startInstallation called ===');
		isInstalling = true;
		installStatus = '正在准备安装...';
		console.log('=== isInstalling set to true ===');

		try {
			if (isTauriAvailable()) {
				console.log('=== Tauri is available, starting installation ===');
				// 从配置中获取 PyPI 镜像和代理设置
				const backendConfig: { pypiMirror?: string | null; proxyUrl?: string | null } | undefined =
					config?.pypiMirror?.url && config?.pypiMirror?.source !== 'auto'
						? {
								pypiMirror: config.pypiMirror.url || null,
								proxyUrl: config?.proxy?.enabled ? config?.proxy?.url || null : null
							}
						: undefined;

				console.log('=== Backend config:', backendConfig);

				// 立即跳转到下一页（安装日志页面）
				console.log('=== Navigating to next step ===');
				onComplete();

				// 延迟调用安装命令，让 ProgressScreen 有时间注册事件监听器
				setTimeout(() => {
					console.log('=== Calling installLocalBackend... ===');
					tauriCommands.download.installLocalBackend(backendConfig).then((result) => {
						console.log('=== Installation command result:', result);
					}).catch((error) => {
						console.error('=== Installation failed:', error);
						alert('安装失败：' + (error as Error).message);
						isInstalling = false;
						installStatus = '安装失败';
					});
				}, 500); // 延迟 500ms 确保 ProgressScreen 已完成 onMount
			} else {
				// 非 Tauri 环境的模拟
				console.warn('Tauri not available, simulating installation');
				onComplete();
				simulateInstallation();
			}
		} catch (error) {
			console.error('=== Installation failed:', error);
			alert('安装失败：' + (error as Error).message);
			isInstalling = false;
			installStatus = '安装失败';
		}
	};

	const simulateInstallation = () => {
		let progress = 0;
		const stages = [
			{ text: '检查 Python 环境...', at: 10 },
			{ text: '安装依赖包...', at: 30 },
			{ text: '配置数据库...', at: 60 },
			{ text: '完成安装...', at: 90 }
		];

		let stageIndex = 0;
		const interval = setInterval(() => {
			progress += 2;

			// 更新状态文本
			while (stageIndex < stages.length && progress >= stages[stageIndex].at) {
				installStatus = stages[stageIndex].text;
				stageIndex++;
			}

			const comp = components.find((c) => c.id === 'backend');
			if (comp) {
				comp.progress = Math.min(progress, 100);
			}

			if (progress >= 100) {
				clearInterval(interval);
				const comp = components.find((c) => c.id === 'backend');
				if (comp) {
					comp.installed = true;
					comp.progress = 100;
				}
				isInstalling = false;
				installStatus = '安装完成！';
				setTimeout(() => {
					onComplete();
				}, 500);
			}
		}, 100);
	};
</script>

{#if currentStep === 5}
	<div class="download-manager">
		<div class="header">
			<h1>安装组件</h1>
			<p>准备从本地源码构建并安装 OpenWebUI 后端</p>
		</div>

		<div class="component-list">
			{#each components as component}
				<div
					class="component-item"
					class:selected={selectedComponents.includes(component.id)}
					class:disabled={isInstalling}
				>
					<div class="item-main">
						<label class="checkbox-wrapper">
							<input
								type="checkbox"
								bind:group={selectedComponents}
								value={component.id}
								disabled={component.required || isInstalling}
							/>
							<span class="checkbox"></span>
						</label>

						<div class="item-content">
							<div class="item-title-row">
								<h3>{component.name}</h3>
								<span class="badge {component.required ? 'required' : 'optional'}">
									{component.required ? '必需' : '可选'}
								</span>
							</div>
							<p class="item-desc">{component.description}</p>
							<div class="item-meta">
								<span class="size">🔧 {component.size}</span>
								{#if component.id === 'python'}
									<span class="note">💡 如系统已有 Python 3.10+ 将自动跳过</span>
								{:else}
									<span class="note">💡 使用本地 Python 环境</span>
								{/if}
							</div>
						</div>
					</div>
				</div>
			{/each}
		</div>

		<div class="info-box">
			<div class="info-icon">ℹ️</div>
			<div class="info-content">
				<h4>本地安装说明</h4>
				<ul>
					<li>系统将自动检测 Python 环境，如未安装将自动下载（~25MB）</li>
					<li>Python 将安装在应用数据目录，不影响系统环境</li>
					<li>安装过程需要网络连接以下载依赖包</li>
					<li>如果配置了代理，安装过程将自动使用代理</li>
					<li>预计安装时间：2-5 分钟（取决于网络速度）</li>
				</ul>
			</div>
		</div>

		<div class="footer">
			<div class="actions">
				<button class="btn-back" on:click={onBack} disabled={isInstalling}>上一步</button>
				<button class="btn-primary" on:click={startInstallation} disabled={isInstalling}>
					{isInstalling ? '安装中...' : '开始安装'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.download-manager {
		/* 移除白色背景和阴影，与其他组件保持一致 */
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.header {
		text-align: center;
	}

	.header h1 {
		font-size: 2rem;
		font-weight: 700;
		color: white;
		margin-bottom: 0.5rem;
	}

	.header p {
		color: rgba(255, 255, 255, 0.8);
		font-size: 0.9375rem;
		margin: 0;
	}

	.component-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.component-item {
		background: rgba(255, 255, 255, 0.95);
		border: 1px solid rgba(255, 255, 255, 0.3);
		border-radius: 1rem;
		overflow: hidden;
		transition: all 0.2s ease;
	}

	.component-item:hover:not(.disabled) {
		background: rgba(255, 255, 255, 1);
		border-color: rgba(255, 255, 255, 0.5);
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
	}

	.component-item.selected {
		border-color: #6366f1;
		background: rgba(255, 255, 255, 1);
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.15);
	}

	.component-item.disabled {
		opacity: 0.5;
		pointer-events: none;
	}

	.item-main {
		display: flex;
		align-items: flex-start;
		gap: 1rem;
		padding: 1.25rem;
	}

	.checkbox-wrapper {
		position: relative;
		cursor: pointer;
		flex-shrink: 0;
	}

	.checkbox-wrapper input {
		position: absolute;
		opacity: 0;
		width: 0;
		height: 0;
	}

	.checkbox {
		width: 1.25rem;
		height: 1.25rem;
		border: 2px solid #d1d5db;
		border-radius: 0.375rem;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s ease;
		background: white;
	}

	.checkbox::after {
		content: '';
		width: 0.625rem;
		height: 0.375rem;
		border: solid white;
		border-width: 0 0 2px 2px;
		transform: rotate(-45deg) scale(0);
		transition: transform 0.2s ease;
		margin-top: -0.125rem;
	}

	.checkbox-wrapper input:checked ~ .checkbox {
		background: #6366f1;
		border-color: #6366f1;
	}

	.checkbox-wrapper input:checked ~ .checkbox::after {
		transform: rotate(-45deg) scale(1);
	}

	.item-content {
		flex: 1;
		min-width: 0;
	}

	.item-title-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
		margin-bottom: 0.375rem;
	}

	.item-title-row h3 {
		font-size: 1rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.badge {
		font-size: 0.75rem;
		padding: 0.25rem 0.625rem;
		border-radius: 0.375rem;
		font-weight: 500;
		flex-shrink: 0;
	}

	.badge.required {
		background: linear-gradient(135deg, #6366f1, #8b5cf6);
		color: white;
	}

	.badge.optional {
		background: #f3f4f6;
		color: #6b7280;
	}

	.item-desc {
		color: #6b7280;
		font-size: 0.875rem;
		margin: 0 0 0.75rem 0;
		line-height: 1.5;
	}

	.item-meta {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.size,
	.note {
		font-size: 0.8125rem;
		color: #9ca3af;
		display: flex;
		align-items: center;
		gap: 0.375rem;
	}

	.info-box {
		display: flex;
		gap: 1rem;
		padding: 1rem;
		background: rgba(255, 255, 255, 0.1);
		border: 1px solid rgba(255, 255, 255, 0.2);
		border-radius: 0.5rem;
	}

	.info-icon {
		font-size: 1.5rem;
		flex-shrink: 0;
	}

	.info-content h4 {
		font-size: 1rem;
		font-weight: 600;
		color: white;
		margin: 0 0 0.75rem 0;
	}

	.info-content ul {
		margin: 0;
		padding-left: 1.25rem;
		color: rgba(255, 255, 255, 0.9);
		font-size: 0.875rem;
		line-height: 1.75;
	}

	.info-content li {
		margin-bottom: 0.25rem;
	}

	.footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.summary {
		display: flex;
		align-items: center;
		gap: 1.5rem;
	}

	.summary-item {
		font-size: 0.875rem;
		color: rgba(255, 255, 255, 0.8);
		font-weight: 500;
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.actions {
		display: flex;
		gap: 0.75rem;
		justify-content: flex-end;
	}

	.btn-back,
	.btn-primary {
		padding: 0.75rem 1.5rem;
		border-radius: 0.5rem;
		font-weight: 500;
		font-size: 0.9375rem;
		cursor: pointer;
		transition: all 0.2s;
		border: none;
	}

	.btn-back {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-back:hover {
		background: #e5e7eb;
	}

	.btn-primary {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
	}

	.btn-primary:active:not(:disabled) {
		transform: translateY(0);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
