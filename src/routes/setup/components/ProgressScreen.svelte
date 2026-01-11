<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { isTauriAvailable, listenDownloadStatus, listenDownloadComplete } from '$lib/utils/tauri';

	export let currentStep: number;
	export let onComplete: () => void;

	let installStatus = '正在准备安装...';
	let displayStatus = '正在准备安装...';
	let installLogs: string[] = [];
	let isComplete = false;

	// Store unlisten functions
	let unlistenStatus: (() => void) | null = null;
	let unlistenComplete: (() => void) | null = null;

	// 解析状态文本，提取简短的包名显示
	const parseStatus = (status: string): string => {
		if (status.includes('正在收集:')) {
			const pkgName = status.replace('正在收集:', '').trim().split(' ')[0];
			return `正在收集: ${pkgName}`;
		}
		if (status.includes('正在下载:')) {
			const pkgName = status.replace('正在下载:', '').trim().split(' ')[0];
			return `正在下载: ${pkgName}`;
		}
		if (status.includes('已安装:')) {
			// 从 "已安装: Requirement already satisfied: xxx in ..." 中提取包名
			const match = status.match(/Requirement already satisfied: ([\w-]+)/);
			if (match) {
				return `已安装: ${match[1]}`;
			}
			// 如果没有匹配到，尝试其他格式
			const pkgName = status.replace('已安装:', '').trim().split(' ')[0];
			return `已安装: ${pkgName}`;
		}
		if (status.includes('正在处理')) {
			return '正在安装依赖包...';
		}
		if (status.includes('安装完成！')) {
			return '安装完成！';
		}
		return status;
	};

	onMount(async () => {
		// Setup event listeners for installation progress
		if (isTauriAvailable()) {
			// 监听下载状态事件
			listenDownloadStatus((status) => {
				console.log('Installation status:', status);
				installStatus = status.status;
				displayStatus = parseStatus(status.status);

				// 添加到日志
				const logEntry = `[${new Date().toLocaleTimeString()}] ${status.status}`;
				addLog(logEntry);

				// 自动滚动到底部
				tick().then(() => {
					scrollToBottom();
				});
			}).then((unlisten) => {
				unlistenStatus = unlisten;
			}).catch((err) => {
				console.error('Failed to register download-status listener:', err);
			});

			// 监听安装完成事件
			listenDownloadComplete((result) => {
				console.log('Installation complete:', result);
				isComplete = true;
				installStatus = '安装完成！';
				displayStatus = '安装完成！';
				addLog('✅ 安装成功完成！');
			}).then((unlisten) => {
				unlistenComplete = unlisten;
			});
		}
	});

	onDestroy(() => {
		// Cleanup event listeners
		unlistenStatus?.();
		unlistenComplete?.();
	});

	const addLog = (log: string) => {
		installLogs = [...installLogs, log];
	};

	const scrollToBottom = () => {
		const logContainer = document.querySelector('.log-container');
		if (logContainer) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	};
</script>

{#if currentStep === 6}
	<div class="progress-screen">
		<div class="progress-content">
			<div class="header">
				<h2>正在安装 OpenWebUI 后端</h2>
				<p class="status">{displayStatus}</p>
			</div>

			<div class="log-section">
				<h3>安装日志</h3>
				<div class="log-container">
					{#if installLogs.length > 0}
						{#each installLogs as log}
							<div class="log-entry">{log}</div>
						{/each}
					{:else}
						<div class="log-entry empty">等待安装开始...</div>
					{/if}
				</div>
			</div>

			{#if isComplete}
				<div class="success-message">
					<div class="success-icon">✓</div>
					<p>安装完成！</p>
					<button class="btn-next" on:click={onComplete}>下一页</button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.progress-screen {
		background: white;
		border-radius: 1rem;
		padding: 2rem;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
		max-height: 80vh;
		overflow: hidden;
	}

	.progress-content {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		height: 100%;
	}

	.header {
		text-align: center;
	}

	.header h2 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #1f2937;
		margin: 0 0 0.5rem 0;
	}

	.status {
		color: #6b7280;
		font-size: 1rem;
		margin: 0;
	}

	.log-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		flex: 1;
		min-height: 0;
	}

	.log-section h3 {
		font-size: 1rem;
		font-weight: 600;
		color: #374151;
		margin: 0;
	}

	.log-container {
		flex: 1;
		background: #1f2937;
		border-radius: 0.5rem;
		padding: 1rem;
		overflow-y: auto;
		max-height: 300px;
		font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
		font-size: 0.875rem;
		line-height: 1.5;
	}

	.log-entry {
		color: #e5e7eb;
		margin-bottom: 0.25rem;
		word-break: break-word;
	}

	.log-entry.empty {
		color: #9ca3af;
		font-style: italic;
	}

	.success-message {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.75rem;
		padding: 1rem;
		background: #d1fae5;
		border-radius: 0.5rem;
		color: #065f46;
	}

	.success-icon {
		width: 2rem;
		height: 2rem;
		background: #10b981;
		color: white;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.25rem;
		font-weight: bold;
	}

	.success-message p {
		margin: 0;
		font-size: 1rem;
		font-weight: 500;
	}

	.btn-next {
		margin-left: auto;
		padding: 0.75rem 2rem;
		background: linear-gradient(135deg, #6366f1, #8b5cf6);
		color: white;
		border: none;
		border-radius: 0.5rem;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn-next:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
	}

	/* 滚动条样式 */
	.log-container::-webkit-scrollbar {
		width: 8px;
	}

	.log-container::-webkit-scrollbar-track {
		background: #374151;
		border-radius: 4px;
	}

	.log-container::-webkit-scrollbar-thumb {
		background: #6b7280;
		border-radius: 4px;
	}

	.log-container::-webkit-scrollbar-thumb:hover {
		background: #9ca3af;
	}
</style>
