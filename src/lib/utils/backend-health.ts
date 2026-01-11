/**
 * Backend Health Check Utility
 *
 * Provides utilities to check backend health with retry logic,
 * particularly useful for Tauri desktop where the backend starts asynchronously.
 */

import { isTauriAvailable } from './tauri';

/**
 * 获取后端配置 URL 和认证信息
 * 支持本地和远程模式
 */
async function getBackendConfig() {
	const config = {
		url: 'http://127.0.0.1:8080' as string,
		auth: undefined as { username: string; password: string } | undefined
	};

	// 检查是否配置了远程服务器
	if (typeof window !== 'undefined' && (window as any).REMOTE_BACKEND_URL) {
		config.url = (window as any).REMOTE_BACKEND_URL;
		config.auth = (window as any).REMOTE_BACKEND_AUTH;
	} else if (isTauriAvailable()) {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const appConfig = await invoke('get_app_config') as {
				setup_mode: string | null;
				remote_url: string | null;
				remote_username: string | null;
				remote_password: string | null;
			};

			if (appConfig.setup_mode === 'remote' && appConfig.remote_url) {
				config.url = appConfig.remote_url;
				if (appConfig.remote_username && appConfig.remote_password) {
					config.auth = {
						username: appConfig.remote_username,
						password: appConfig.remote_password
					};
				}
			}
		} catch {
			// 忽略错误，使用默认配置
		}
	}

	return config;
}

export interface HealthCheckOptions {
	/** Maximum number of retry attempts */
	maxRetries?: number;
	/** Delay between retries in milliseconds */
	retryDelay?: number;
	/** Whether to show progress in console */
	verbose?: boolean;
}

export interface HealthCheckResult {
	/** Whether the backend is healthy */
	healthy: boolean;
	/** Backend config if healthy */
	config?: any;
	/** Error message if not healthy */
	error?: string;
	/** Number of attempts made */
	attempts: number;
}

/**
 * Check if the backend is healthy by fetching the config endpoint
 * Supports retry logic for cases where backend is still starting up
 */
export async function checkBackendHealth(
	options: HealthCheckOptions = {}
): Promise<HealthCheckResult> {
	const {
		maxRetries = isTauriAvailable() ? 30 : 3, // More retries in Tauri
		retryDelay = 1000,
		verbose = true
	} = options;

	let lastError: string = '';

	for (let attempt = 1; attempt <= maxRetries; attempt++) {
		try {
			if (verbose && attempt > 1) {
				console.log(`[Health Check] Attempt ${attempt}/${maxRetries}...`);
			}

			// 获取后端配置（支持远程模式）
			const backendConfig = await getBackendConfig();

			// 构建请求头
			const headers: Record<string, string> = {
				'Content-Type': 'application/json'
			};

			// 添加 Basic Auth（如果有）
			if (backendConfig.auth) {
				const credentials = btoa(`${backendConfig.auth.username}:${backendConfig.auth.password}`);
				headers['Authorization'] = `Basic ${credentials}`;
			}

			if (verbose) {
				console.log(`[Health Check] Connecting to: ${backendConfig.url}`);
			}

			// Try to fetch backend config
			const configUrl = backendConfig.url.replace(/\/$/, '') + '/api/config';
			const response = await fetch(configUrl, {
				method: 'GET',
				headers
			});

			if (response.ok) {
				const config = await response.json();
				if (verbose) {
					console.log(`[Health Check] Backend is healthy! (attempt ${attempt}/${maxRetries})`);
					console.log(`[Health Check] Backend URL: ${backendConfig.url}`);
				}
				return {
					healthy: true,
					config,
					attempts: attempt
				};
			}

			lastError = `HTTP ${response.status}: ${response.statusText}`;
		} catch (error) {
			lastError = error instanceof Error ? error.message : String(error);
			if (verbose) {
				console.log(`[Health Check] Attempt ${attempt}/${maxRetries} failed: ${lastError}`);
			}
		}

		// Wait before retrying (unless it's the last attempt)
		if (attempt < maxRetries) {
			await new Promise(resolve => setTimeout(resolve, retryDelay));
		}
	}

	if (verbose) {
		console.error(`[Health Check] Backend unhealthy after ${maxRetries} attempts`);
	}

	return {
		healthy: false,
		error: lastError,
		attempts: maxRetries
	};
}

/**
 * Start a background health check that will trigger a callback when the backend becomes healthy
 */
export function startBackgroundHealthCheck(
	onHealthy: (config: any) => void,
	onUnhealthy?: (error: string) => void,
	options: HealthCheckOptions = {}
): () => void {
	const {
		maxRetries = 60, // Keep checking for a while
		retryDelay = 2000,
		verbose = false
	} = options;

	let attempts = 0;
	let stopped = false;

	const check = async () => {
		while (!stopped && attempts < maxRetries) {
			attempts++;

			try {
				const response = await fetch(`${window.location.protocol}//127.0.0.1:8080/api/config`, {
					method: 'GET',
					headers: {
						'Content-Type': 'application/json'
					}
				});

				if (response.ok) {
					const config = await response.json();
					if (verbose) {
						console.log(`[Background Health Check] Backend is healthy!`);
					}
					onHealthy(config);
					return;
				}
			} catch (error) {
				// Silently ignore errors during background check
				if (verbose) {
					console.log(`[Background Health Check] Attempt ${attempts}/${maxRetries} failed`);
				}
			}

			// Wait before next check
			await new Promise(resolve => setTimeout(resolve, retryDelay));
		}

		if (!stopped && onUnhealthy) {
			onUnhealthy('Backend not available after maximum retries');
		}
	};

	// Start checking in background
	check();

	// Return stop function
	return () => {
		stopped = true;
	};
}
