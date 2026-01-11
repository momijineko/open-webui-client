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
		REMOTE_CONFIG: 4, // 远程模式配置（与本地模式复用步骤号）
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
	let installationStatus = '初始化中...';

	const handleModeSelect = (mode: 'local' | 'remote') => {
		selectedMode = mode;
		if (mode === 'remote') {
			currentStep = SetupStep.REMOTE_CONFIG;
		} else {
			currentStep = SetupStep.ENVIRONMENT_CHECK;
		}
	};

	const handleEnvironmentDetected = (status: any) => {
		console.log('环境检测结果:', status);

		if (status.is_installed) {
			// 已安装，直接跳到安装步骤启动后端
			currentStep = SetupStep.INSTALLATION;
			handleDownloadComplete();
		} else {
			// 未安装，继续下载流程
			currentStep = SetupStep.MIRROR_CONFIG;
		}
	};

	const handleUseExistingBackend = () => {
		// 使用现有后端，直接跳到安装步骤
		currentStep = SetupStep.INSTALLATION;
		handleDownloadComplete();
	};

	const handleSkipDetection = () => {
		// 跳过检测，直接进入下载流程
		currentStep = SetupStep.MIRROR_CONFIG;
	};

	const handleBackFromEnvironment = () => {
		// 返回到模式选择页面
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
		installationStatus = '正在启动后端服务...';

		try {
			if (isTauriAvailable()) {
				// 检查后端状态
				const status = await tauriCommands.backend.checkStatus();

				if (status.is_running) {
					installationStatus = '后端已在运行中';
				} else {
					// 启动后端，传入配置
					installationStatus = '正在启动后端服务...';

					// 构建后端配置
					const backendConfig = {
						hf_endpoint: config.hfMirror?.url || undefined,
						offline_mode: config.offlineMode || undefined,
						pypi_mirror: config.pypiMirror?.url || undefined
					};

					console.log('Starting backend with config:', backendConfig);
					const result = await tauriCommands.backend.start(backendConfig);
					console.log('Backend started:', result);

					// 等待一下让后端完全启动
					await new Promise((resolve) => setTimeout(resolve, 2000));

					// 再次检查状态
					const newStatus = await tauriCommands.backend.checkStatus();
					if (newStatus.is_running) {
						installationStatus = '后端启动成功！';
					} else {
						installationStatus = '后端启动失败，请检查日志';
					}
				}

				// 不再自动跳转到完成页面
			} else {
				// 非 Tauri 环境的模拟流程
				console.warn('Tauri 不可用，跳过后端启动');
				installationStatus = '检测到 Web 环境，跳过后端启动';
			}
		} catch (error) {
			console.error('后端启动失败:', error);
			installationStatus = '后端启动失败：' + (error as Error).message;
			// 不再自动跳转，让用户留在安装页面查看状态
		}
	};

	// 安装完成后直接跳转到完成页面（后端已在 handleDownloadComplete 中启动）
	const handleInstallationComplete = () => {
		currentStep = SetupStep.COMPLETION;
	};

	const handleComplete = async () => {
		try {
			// 如果是远程模式，传递远程服务器配置
			if (selectedMode === 'remote' && remoteServerConfig.url) {
				const params: Record<string, string | null> = {
					mode: 'remote',
					remote_url: remoteServerConfig.url || '',
					remote_username: remoteServerConfig.username || null,
					remote_password: remoteServerConfig.password || null
				};

				// 直接使用 invoke 调用后端
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

		// 跳转到主应用
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
