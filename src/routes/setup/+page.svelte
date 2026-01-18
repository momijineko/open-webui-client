<script lang="ts">
	import ModeSelector from './components/ModeSelector.svelte';
	import MirrorConfig from './components/MirrorConfig.svelte';
	import EnvironmentCheck from './components/EnvironmentCheck.svelte';
	import DownloadManager from './components/DownloadManager.svelte';
	import ProgressScreen from './components/ProgressScreen.svelte';
	import CompletionScreen from './components/CompletionScreen.svelte';
	import RemoteServerConfig from './components/RemoteServerConfig.svelte';
	import { tauriCommands, isTauriAvailable } from '$lib/utils/tauri';

	const SetupStep = {
		WELCOME: 1,
		MODE_SELECTION: 2,
		ENVIRONMENT_CHECK: 3,
		MIRROR_CONFIG: 4,
		REMOTE_CONFIG: 4, // Remote mode configuration (shares step number with local mode)
		DOWNLOAD_COMPONENTS: 5,
		INSTALLATION: 6,
		COMPLETION: 7
	};

	let currentStep = SetupStep.MODE_SELECTION;
	let selectedMode: 'local' | 'remote' | null = null;
	let config = {
		mirrorSource: 'auto',
		customMirrors: [],
		enableProxy: false,
		proxyUrl: '',
		downloadOptional: false,
		componentMirror: { source: '', url: null as string | null },
		pypiMirror: { source: '', url: null as string | null },
		hfMirror: { source: '', url: null as string | null },
		offlineMode: false
	};
	let remoteServerConfig: {
		url: string;
		username?: string;
		password?: string;
	} = {
		url: '',
		username: undefined,
		password: undefined
	};
	let installationStatus = 'Initializing...';

	const handleModeSelect = (mode: 'local' | 'remote') => {
		selectedMode = mode;
		if (mode === 'remote') {
			currentStep = SetupStep.REMOTE_CONFIG;
		} else {
			currentStep = SetupStep.ENVIRONMENT_CHECK;
		}
	};

	const handleEnvironmentDetected = (status: any) => {
		console.log('Environment detection result:', status);

		if (status.is_installed) {
			// Already installed, jump to installation step to start backend
			currentStep = SetupStep.INSTALLATION;
			handleDownloadComplete();
		} else {
			// Not installed, continue download process
			currentStep = SetupStep.MIRROR_CONFIG;
		}
	};

	const handleUseExistingBackend = () => {
		// Use existing backend, jump directly to installation step
		currentStep = SetupStep.INSTALLATION;
		handleDownloadComplete();
	};

	const handleSkipDetection = () => {
		// Skip detection, enter download process directly
		currentStep = SetupStep.MIRROR_CONFIG;
	};

	const handleBackFromEnvironment = () => {
		// Return to mode selection page
		currentStep = SetupStep.MODE_SELECTION;
	};

	const handleMirrorConfig = (cfg: typeof config) => {
		config = cfg;
		currentStep = SetupStep.DOWNLOAD_COMPONENTS;
	};

	const handleBackFromDownload = () => {
		currentStep = SetupStep.MIRROR_CONFIG;
	};

	const handleBackFromMirror = () => {
		currentStep = SetupStep.ENVIRONMENT_CHECK;
	};

	const handleRemoteConfig = (serverConfig: { url: string; username?: string; password?: string }) => {
		remoteServerConfig = {
			url: serverConfig.url,
			username: serverConfig.username || undefined,
			password: serverConfig.password || undefined
		};
		currentStep = SetupStep.COMPLETION;
	};

	const handleBackFromRemote = () => {
		currentStep = SetupStep.MODE_SELECTION;
	};

	const handleDownloadComplete = async () => {
		currentStep = SetupStep.INSTALLATION;
		installationStatus = 'Starting backend service...';

		try {
			if (isTauriAvailable()) {
				// Check backend status
				const status = await tauriCommands.backend.checkStatus();

				if (status.is_running) {
					installationStatus = 'Backend is already running';
				} else {
					// Start backend with configuration
					installationStatus = 'Starting backend service...';

					// Build backend configuration
					const backendConfig = {
						hf_endpoint: config.hfMirror?.url || undefined,
						offline_mode: config.offlineMode || undefined,
						pypi_mirror: config.pypiMirror?.url || undefined
					};

					console.log('Starting backend with config:', backendConfig);
					const result = await tauriCommands.backend.start(backendConfig);
					console.log('Backend started:', result);

					// Wait a moment for backend to fully start
					await new Promise((resolve) => setTimeout(resolve, 2000));

					// Check status again
					const newStatus = await tauriCommands.backend.checkStatus();
					if (newStatus.is_running) {
						installationStatus = 'Backend started successfully!';
					} else {
						installationStatus = 'Backend startup failed, please check logs';
					}
				}

				// No longer auto-redirect to completion page
			} else {
				// Simulated process for non-Tauri environment
				console.warn('Tauri not available, skipping backend startup');
				installationStatus = 'Web environment detected, skipping backend startup';
			}
		} catch (error) {
			console.error('Backend startup failed:', error);
			installationStatus = 'Backend startup failed: ' + (error as Error).message;
			// No longer auto-redirect, let user stay on installation page to view status
		}
	};

	// Jump to completion page after installation (backend already started in handleDownloadComplete)
	const handleInstallationComplete = () => {
		currentStep = SetupStep.COMPLETION;
	};

	const handleComplete = async () => {
		try {
			// If remote mode, pass remote server configuration
			if (selectedMode === 'remote' && remoteServerConfig.url) {
				const params: Record<string, string | null> = {
					mode: 'remote',
					remote_url: remoteServerConfig.url || '',
					remote_username: remoteServerConfig.username || null,
					remote_password: remoteServerConfig.password || null
				};

				// Use invoke to call backend directly
				if (isTauriAvailable()) {
					const { invoke } = await import('@tauri-apps/api/core');
					await invoke('set_setup_completed', params);
				}
			} else {
				await tauriCommands.config.setSetupCompleted('local');
			}
		} catch (error) {
			console.error('[Setup] Failed to save configuration:', error);
		}

		// Redirect to main application
		window.location.href = '/';
	};
</script>

<div class="setup-container">
	<div class="setup-content">
		<ModeSelector
			{currentStep}
			onSelect={handleModeSelect}
		/>

		<EnvironmentCheck
			{currentStep}
			onDetected={handleEnvironmentDetected}
			onSkip={handleSkipDetection}
			onUseExisting={handleUseExistingBackend}
			onBack={handleBackFromEnvironment}
		/>

		<MirrorConfig
			{currentStep}
			{selectedMode}
			onConfirm={handleMirrorConfig}
			onBack={handleBackFromMirror}
		/>

		<RemoteServerConfig
			{currentStep}
			{selectedMode}
			onConfirm={handleRemoteConfig}
			onBack={handleBackFromRemote}
		/>

		<DownloadManager
			{currentStep}
			{config}
			onComplete={handleDownloadComplete}
			onBack={handleBackFromDownload}
		/>

		<ProgressScreen
			{currentStep}
			onComplete={handleInstallationComplete}
		/>

		<CompletionScreen
			{currentStep}
			mode={selectedMode}
			onComplete={handleComplete}
		/>
	</div>
</div>

<style>
	.setup-container {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 100vh;
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		padding: 2rem;
	}

	.setup-content {
		width: 100%;
		max-width: 600px;
	}
</style>
