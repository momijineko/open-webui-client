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

	// Reset detection status when currentStep becomes 3
	$: if (currentStep === 3) {
		detectionStatus = 'detecting';
		installStatus = null;
		errorMessage = '';
	}

	onMount(async () => {
		// Only execute detection when current step is Environment Check step
		if (currentStep !== 3) {
			return;
		}

		if (!isTauriAvailable()) {
			detectionStatus = 'error';
			errorMessage = 'Not running in Tauri environment';
			return;
		}

		try {
			installStatus = await tauriCommands.backend.checkInstallation();
			detectionStatus = 'completed';
			// No longer auto-redirect, let user choose Next action
		} catch (error) {
			console.error('Environment Check failed:', error);
			detectionStatus = 'error';
			errorMessage = (error as Error).message;
		}
	});

	// Trigger detection when currentStep becomes 3
	$: if (currentStep === 3 && detectionStatus === 'detecting') {
		(async () => {
			if (!isTauriAvailable()) {
				detectionStatus = 'error';
				errorMessage = 'Not running in Tauri environment';
				return;
			}

			try {
				installStatus = await tauriCommands.backend.checkInstallation();
				detectionStatus = 'completed';
				// No longer auto-redirect, let user choose Next action
			} catch (error) {
				console.error('Environment Check failed:', error);
				detectionStatus = 'error';
				errorMessage = (error as Error).message;
			}
		})();
	}

	const formatStatusText = () => {
		if (!installStatus) return '';

		if (installStatus.is_installed) {
			if (installStatus.backend_executable) {
				return `Installed backend executable: ${installStatus.backend_executable}`;
			} else if (installStatus.open_webui_installed) {
				return `OpenWebUI installed via Python Environment (${installStatus.python_version || 'Unknown version'})`;
			} else if (installStatus.installation_path && !installStatus.open_webui_installed) {
				// Has installation path but no open-webui package, indicates source development mode
				return `Source development directory detected:${installStatus.installation_path}`;
			}
		}

		return 'No backend installation detected';
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
		<h1>Environment Check</h1>
		<p>Checking your system environment...</p>

		{#if detectionStatus === 'detecting'}
			<div class="detection-loading">
				<div class="spinner"></div>
				<p>Detecting Python Environment...</p>
			</div>
		{:else if detectionStatus === 'completed' && installStatus}
			<div class="detection-result">
				<div class="status-header">
					<span class="status-icon">{getStatusIcon()}</span>
					<h2>{installStatus.is_installed ? 'Backend installation detected' : 'Backend components need to be downloaded'}</h2>
				</div>

				<div class="status-details">
					<div class="detail-item">
						<span class="label">Backend Status:</span>
						<span class="value">{installStatus.is_installed ? 'Installed' : 'Not Installed'}</span>
					</div>

					<div class="detail-item">
						<span class="label">Python Environment:</span>
						<span class="value">
							{installStatus.python_available
								? `${installStatus.python_version || 'Installed'}`
								: 'Not Installed'}
						</span>
					</div>

					{#if installStatus.installation_path}
						<div class="detail-item">
							<span class="label">Installation Path:</span>
							<span class="value">{installStatus.installation_path}</span>
						</div>
					{/if}
				</div>

				<div class="status-message">
					<p>{formatStatusText()}</p>
				</div>

				<div class="actions">
					<button class="btn-secondary btn-back" on:click={onBack}>Previous</button>
					{#if installStatus.is_installed}
						<button class="btn-primary" on:click={onUseExisting}>
							Use Existing Backend
						</button>
						<button class="btn-secondary" on:click={() => onDetected(installStatus)}>
							Re-download
						</button>
					{:else}
						<button class="btn-primary" on:click={() => onDetected(installStatus)}>
							Continue Installation
						</button>
					{/if}
				</div>
			</div>
		{:else if detectionStatus === 'error'}
			<div class="detection-error">
				<div class="error-icon">❌</div>
				<h2>Environment Check Failed</h2>
				<p class="error-message">{errorMessage}</p>

				<div class="actions">
					<button class="btn-secondary btn-back" on:click={onBack}>Previous</button>
					<button class="btn-primary" on:click={onSkip}>Skip Detection</button>
					<button class="btn-secondary" on:click={() => window.location.reload()}>Re-check</button>
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
