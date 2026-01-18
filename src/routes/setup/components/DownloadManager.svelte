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
			name: 'Python Environment',
			description: 'Python 3.11 runtime environment (will be skipped if already installed)',
			size: '~25MB',
			required: true,
			installed: false,
			progress: 0,
			status: 'pending'
		},
		{
			id: 'backend',
			name: 'OpenWebUI Backend',
			description: 'Build and install Python backend service from local source',
			size: 'Local build',
			required: true,
			installed: false,
			progress: 0,
			status: 'pending'
		}
	];

	let selectedComponents = ['python', 'backend'];
	let isInstalling = false;
	let installStatus = 'Preparing installation...';
	let installingPython = false; // Whether installing Python

	// Store unlisten functions
	let unlistenProgress: (() => void) | null = null;
	let unlistenStatus: (() => void) | null = null;
	let unlistenComplete: (() => void) | null = null;

	onMount(async () => {
		// Setup event listeners for installation progress
		if (isTauriAvailable()) {
			// Add a global test listener (for debugging only)
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

			// Python installation progress listener
			listenDownloadProgress((progress) => {
				console.log('=== Download progress event received ===', progress);
				console.log('=== progress.percentage ===', progress.percentage);
				console.log('=== installingPython ===', installingPython);

				const compIndex = components.findIndex((c) => c.id === (installingPython ? 'python' : 'backend'));
				console.log('=== Found component index ===', compIndex);

				if (compIndex !== -1) {
					console.log('=== component.progress before ===', components[compIndex].progress);
					// Use Svelte's reactive update method: reassign the entire array
					components[compIndex] = {
						...components[compIndex],
						progress: progress.percentage
					};
					components = [...components]; // Trigger reactive update
					console.log('=== component.progress after ===', components[compIndex].progress);

					const statusText = installingPython
						? `Installing Python: ${Math.round(progress.percentage)}%`
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

			// Installation status listener
			listenDownloadStatus((status) => {
				console.log('=== Installation status event received ===', status);
				console.log('=== status.status ===', status.status);
				console.log('=== installStatus before ===', installStatus);

				// Update status text
				installStatus = status.status;

				console.log('=== installStatus after ===', installStatus);

				// If Python installation is detected
				if (status.file?.includes('Python')) {
					console.log('=== Installing Python ===');
					installingPython = true;
					const pythonComp = components.find((c) => c.id === 'python');
					if (pythonComp) {
						pythonComp.status = 'installing';
						pythonComp.progress = 50;
					}
				} else if (status.file?.includes('Backend')) {
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

				// Force trigger reactive update
				return Promise.resolve();
			}).then((unlisten) => {
				console.log('download-status listener registered');
				unlistenStatus = unlisten;
			}).catch((err) => {
				console.error('Failed to register download-status listener:', err);
			});

			// Installation complete listener
			listenDownloadComplete((result) => {
				console.log('Installation complete:', result);
				const backendComp = components.find((c) => c.id === 'backend');
				if (backendComp) {
					backendComp.installed = result.completed.includes(backendComp.name);
					backendComp.progress = 100;
				}
				isInstalling = false;
				installStatus = 'Installation Complete!';
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
		installStatus = 'Preparing installation...';
		console.log('=== isInstalling set to true ===');

		try {
			if (isTauriAvailable()) {
				console.log('=== Tauri is available, starting installation ===');
				// Get PyPI mirror and proxy settings from config
				const backendConfig: { pypiMirror?: string | null; proxyUrl?: string | null } | undefined =
					config?.pypiMirror?.url && config?.pypiMirror?.source !== 'auto'
						? {
								pypiMirror: config.pypiMirror.url || null,
								proxyUrl: config?.proxy?.enabled ? config?.proxy?.url || null : null
							}
						: undefined;

				console.log('=== Backend config:', backendConfig);

				// Navigate to next immediately (Installation Log page)
				console.log('=== Navigating to next step ===');
				onComplete();

				// Delay calling install command to give ProgressScreen time to register event listeners
				setTimeout(() => {
					console.log('=== Calling installLocalBackend... ===');
					tauriCommands.download.installLocalBackend(backendConfig).then((result) => {
						console.log('=== Installation command result:', result);
					}).catch((error) => {
						console.error('=== Installation failed:', error);
						alert('Installation failed:' + (error as Error).message);
						isInstalling = false;
						installStatus = 'Installation failed';
					});
				}, 500); // Delay 500ms to ensure ProgressScreen has completed onMount
			} else {
				// Non-Tauri environment simulation
				console.warn('Tauri not available, simulating installation');
				onComplete();
				simulateInstallation();
			}
		} catch (error) {
			console.error('=== Installation failed:', error);
			alert('Installation failed:' + (error as Error).message);
			isInstalling = false;
			installStatus = 'Installation failed';
		}
	};

	const simulateInstallation = () => {
		let progress = 0;
		const stages = [
			{ text: 'Checking Python Environment...', at: 10 },
			{ text: 'Installing dependency packages...', at: 30 },
			{ text: 'Configuring database...', at: 60 },
			{ text: 'Completing installation...', at: 90 }
		];

		let stageIndex = 0;
		const interval = setInterval(() => {
			progress += 2;

			// Update status text
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
				installStatus = 'Installation Complete!';
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
			<h1>Install Components</h1>
			<p>Preparing to build and install OpenWebUI Backend from local source</p>
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
									{component.required ? 'Required' : 'Optional'}
								</span>
							</div>
							<p class="item-desc">{component.description}</p>
							<div class="item-meta">
								<span class="size">🔧 {component.size}</span>
								{#if component.id === 'python'}
									<span class="note">💡 Will skip if Python 3.10+ is already installed</span>
								{:else}
									<span class="note">💡 Using local Python Environment</span>
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
				<h4>Local Installation Instructions</h4>
				<ul>
					<li>System will automatically detect Python Environment, download if not installed (~25MB)</li>
					<li>Python will be installed in app data directory, will not affect system environment</li>
					<li>Installation process requires network connection to download dependency packages</li>
					<li>If proxy is configured, installation will use it automatically</li>
					<li>Estimated installation time: 2-5 minutes (depends on network speed)</li>
				</ul>
			</div>
		</div>

		<div class="footer">
			<div class="actions">
				<button class="btn-back" on:click={onBack} disabled={isInstalling}>Previous</button>
				<button class="btn-primary" on:click={startInstallation} disabled={isInstalling}>
					{isInstalling ? 'Installing...' : 'Start Installation'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.download-manager {
		/* Remove white background and shadow to match other components */
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
