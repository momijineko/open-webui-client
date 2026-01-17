<script lang="ts">
	export let currentStep: number;
	export let onDetected: (status: any) => void;
	export let onSkip: () => void;
	export let onUseExisting: () => void;
	export let onBack: () => void;

	import { onMount } from 'svelte';
	import { tauriCommands, isTauriAvailable, type BackendInstallationStatus } from '$lib/utils/tauri';

	let detectionStatus: 'detecting' | 'completed' | 'error' = 'detecting';
	let installStatus: BackendInstallationStatus | null = null;
	let errorMessage = '';

	// 当 currentStep 变为 3 时重置检测状态
	$: if (currentStep === 3) {
		detectionStatus = 'detecting';
		installStatus = null;
		errorMessage = '';
	}

	onMount(async () => {
		// 只在当前步骤是环境检测步骤时才执行检测
		if (currentStep !== 3) {
			return;
		}

		if (!isTauriAvailable()) {
			detectionStatus = 'error';
			errorMessage = '未在 Tauri 环境中运行';
			return;
		}

		try {
			installStatus = await tauriCommands.backend.checkInstallation();
			detectionStatus = 'completed';
			// 不再自动跳转，让用户选择下一步操作
		} catch (error) {
			console.error('环境检测失败:', error);
			detectionStatus = 'error';
			errorMessage = (error as Error).message;
		}
	});

	// 当 currentStep 变为 3 时触发检测
	$: if (currentStep === 3 && detectionStatus === 'detecting') {
		(async () => {
			if (!isTauriAvailable()) {
				detectionStatus = 'error';
				errorMessage = '未在 Tauri 环境中运行';
				return;
			}

			try {
				installStatus = await tauriCommands.backend.checkInstallation();
				detectionStatus = 'completed';
				// 不再自动跳转，让用户选择下一步操作
			} catch (error) {
				console.error('环境检测失败:', error);
				detectionStatus = 'error';
				errorMessage = (error as Error).message;
			}
		})();
	}

	const formatStatusText = () => {
		if (!installStatus) return '';

		if (installStatus.is_installed) {
			if (installStatus.backend_executable) {
				return `已安装后端可执行文件：${installStatus.backend_executable}`;
			} else if (installStatus.open_webui_installed) {
				return `已通过 Python 环境安装 OpenWebUI (${installStatus.python_version || '未知版本'})`;
			} else if (installStatus.installation_path && !installStatus.open_webui_installed) {
				// 有安装路径但没有 open-webui 包，说明是源码开发模式
				return `检测到源码开发目录：${installStatus.installation_path}`;
			}
		}

		return '未检测到已安装的后端';
	};

	const getStatusIcon = () => {
		if (!installStatus) return '⏳';

		if (installStatus.is_installed) {
			return '✅';
		}

		return '⚠️';
	};
</script>

{#if currentStep === 3}
	<div class="environment-check">
		<h1>环境检测</h1>
		<p>正在检测您的系统环境...</p>

		{#if detectionStatus === 'detecting'}
			<div class="detection-loading">
				<div class="spinner"></div>
				<p>正在检测 Python 环境...</p>
			</div>
		{:else if detectionStatus === 'completed' && installStatus}
			<div class="detection-result">
				<div class="status-header">
					<span class="status-icon">{getStatusIcon()}</span>
					<h2>{installStatus.is_installed ? '检测到已安装的后端' : '需要下载后端组件'}</h2>
				</div>

				<div class="status-details">
					<div class="detail-item">
						<span class="label">后端状态：</span>
						<span class="value">{installStatus.is_installed ? '已安装' : '未安装'}</span>
					</div>

					<div class="detail-item">
						<span class="label">Python 环境：</span>
						<span class="value">
							{installStatus.python_available
								? `${installStatus.python_version || '已安装'}`
								: '未安装'}
						</span>
					</div>

					{#if installStatus.installation_path}
						<div class="detail-item">
							<span class="label">安装路径：</span>
							<span class="value">{installStatus.installation_path}</span>
						</div>
					{/if}
				</div>

				<div class="status-message">
					<p>{formatStatusText()}</p>
				</div>

				<div class="actions">
					<button class="btn-secondary btn-back" on:click={onBack}>上一步</button>
					{#if installStatus.is_installed}
						<button class="btn-primary" on:click={onUseExisting}>
							使用现有后端
						</button>
						<button class="btn-secondary" on:click={() => onDetected(installStatus)}>
							重新下载
						</button>
					{:else}
						<button class="btn-primary" on:click={() => onDetected(installStatus)}>
							继续安装
						</button>
					{/if}
				</div>
			</div>
		{:else if detectionStatus === 'error'}
			<div class="detection-error">
				<div class="error-icon">❌</div>
				<h2>环境检测失败</h2>
				<p class="error-message">{errorMessage}</p>

				<div class="actions">
					<button class="btn-secondary btn-back" on:click={onBack}>上一步</button>
					<button class="btn-primary" on:click={onSkip}>跳过检测</button>
					<button class="btn-secondary" on:click={() => window.location.reload()}>重新检测</button>
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	.environment-check {
		background: white;
		border-radius: 1rem;
		padding: 2rem;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
	}

	.environment-check h1 {
		font-size: 1.75rem;
		font-weight: 700;
		color: #1f2937;
		margin-bottom: 0.5rem;
		text-align: center;
	}

	.environment-check > p {
		color: #6b7280;
		text-align: center;
		margin-bottom: 2rem;
	}

	.detection-loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1.5rem;
		padding: 3rem 0;
	}

	.spinner {
		width: 3rem;
		height: 3rem;
		border: 4px solid #e5e7eb;
		border-top-color: #6366f1;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.detection-result {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.status-header {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding-bottom: 1rem;
		border-bottom: 2px solid #e5e7eb;
	}

	.status-icon {
		font-size: 2.5rem;
	}

	.status-header h2 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1f2937;
		margin: 0;
	}

	.status-details {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		background: #f9fafb;
		padding: 1rem;
		border-radius: 0.5rem;
	}

	.detail-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.detail-item .label {
		font-weight: 500;
		color: #4b5563;
	}

	.detail-item .value {
		color: #1f2937;
		font-family: monospace;
	}

	.status-message {
		background: #fef3c7;
		border-left: 4px solid #f59e0b;
		padding: 1rem;
		border-radius: 0.5rem;
	}

	.status-message p {
		margin: 0;
		color: #92400e;
	}

	.detection-error {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
		padding: 2rem 0;
	}

	.error-icon {
		font-size: 4rem;
	}

	.detection-error h2 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #dc2626;
		margin: 0;
	}

	.error-message {
		color: #6b7280;
		text-align: center;
	}

	.actions {
		display: flex;
		gap: 1rem;
		justify-content: center;
		margin-top: 1rem;
	}

	.btn-primary,
	.btn-secondary {
		padding: 0.75rem 1.5rem;
		border-radius: 0.5rem;
		font-weight: 500;
		cursor: pointer;
		border: none;
		transition: all 0.2s;
	}

	.btn-primary {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
	}

	.btn-primary:hover {
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-secondary:hover {
		background: #e5e7eb;
	}
</style>
