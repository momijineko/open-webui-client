/**
 * Tauri API wrapper for desktop client integration
 * Only available when running in Tauri desktop environment
 */

import { PLATFORM } from '$lib/constants';

// Type definitions for Tauri commands
export interface BackendStatus {
	is_running: boolean;
	port: number | null;
	pid: number | null;
}

export interface BackendConfig {
	hf_endpoint?: string | null;
	offline_mode?: boolean | null;
	pypi_mirror?: string | null;
}

export interface BackendInstallationStatus {
	is_installed: boolean;
	installation_path: string | null;
	python_available: boolean;
	python_version: string | null;
	open_webui_installed: boolean;
	backend_executable: string | null;
}

export interface DownloadComponent {
	name: string;
	url: string;
	size: number;
	required: boolean;
}

export interface DownloadConfig {
	components: DownloadComponent[];
	mirror_url: string | null;
	proxy_url: string | null;
}

export interface DownloadProgress {
	current: number;
	total: number;
	file: string;
	speed: string;
	percentage: number;
}

export interface PythonInstallProgress {
	current: number;
	total: number;
	file: string;
	speed: string;
	percentage: number;
	stage: string; // "downloading", "extracting", "configuring"
}

export interface PythonInstallResult {
	success: boolean;
	python_path: string;
	version: string;
	message: string;
}

export interface Instance {
	id: string;
	name: string;
	url: string;
	mode: string;
}

export interface AppConfig {
	setup_completed: boolean;
	setup_mode: string | null;
	remote_url: string | null;
	remote_username: string | null;
	remote_password: string | null;
}

/**
 * Check if Tauri invoke API is available
 */
export function isTauriAvailable(): boolean {
	return PLATFORM.isDesktop && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Safely invoke Tauri commands with error handling
 */
async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauriAvailable()) {
		throw new Error(`Tauri is not available. Cannot invoke command: ${command}`);
	}

	try {
		const { invoke } = await import('@tauri-apps/api/core');
		return await invoke<T>(command, args);
	} catch (error) {
		console.error(`Tauri command ${command} failed:`, error);
		throw error;
	}
}

// Backend Management Commands
export const backendCommands = {
	/**
	 * Start the Python backend server
	 */
	start: async (config?: BackendConfig): Promise<string> => {
		return invoke('start_backend', { config });
	},

	/**
	 * Stop the Python backend server
	 */
	stop: async (): Promise<string> => {
		return invoke('stop_backend');
	},

	/**
	 * Check if backend is running
	 */
	checkStatus: async (): Promise<BackendStatus> => {
		return invoke('check_backend_status');
	},

	/**
	 * Check backend installation status
	 */
	checkInstallation: async (): Promise<BackendInstallationStatus> => {
		return invoke('check_backend_installation');
	},

	/**
	 * Get backend logs
	 */
	getLogs: async (): Promise<string> => {
		return invoke('get_backend_logs');
	}
};

// Download Management Commands
export const downloadCommands = {
	/**
	 * Install local backend from source
	 */
	installLocalBackend: async (config?: {
		pypiMirror?: string | null;
		proxyUrl?: string | null;
	}): Promise<string> => {
		return invoke('install_local_backend', { config });
	},

	/**
	 * Install Python (auto-downloads if not present)
	 */
	installPython: async (): Promise<PythonInstallResult> => {
		return invoke('install_python');
	},

	/**
	 * Get installed Python path (if any)
	 */
	getInstalledPython: async (): Promise<string | null> => {
		return invoke('get_installed_python');
	},

	/**
	 * Check if Python needs to be installed
	 */
	checkPythonNeeded: async (): Promise<boolean> => {
		return invoke('check_python_needed');
	},

	/**
	 * Start downloading components
	 */
	start: async (config: DownloadConfig): Promise<string> => {
		return invoke('start_download', { config });
	},

	/**
	 * Cancel ongoing download
	 */
	cancel: async (): Promise<string> => {
		return invoke('cancel_download');
	},

	/**
	 * Get download directory path
	 */
	getDir: async (): Promise<string> => {
		return invoke('get_download_dir_path');
	}
};

/**
 * Listen to download progress events
 */
export async function listenDownloadProgress(
	callback: (progress: DownloadProgress) => void
): Promise<() => void> {
	if (!isTauriAvailable()) {
		console.warn('Tauri is not available, download progress listener will not be registered');
		return () => {};
	}

	try {
		const { listen } = await import('@tauri-apps/api/event');
		const unlisten = await listen('download-progress', (event) => {
			// 兼容两种类型: DownloadProgress 和 PythonInstallProgress
			const payload = event.payload as any;
			const progress: DownloadProgress = {
				current: payload.current || 0,
				total: payload.total || 0,
				file: payload.file || '',
				speed: payload.speed || '',
				percentage: payload.percentage || 0
			};
			callback(progress);
		});
		return unlisten;
	} catch (error) {
		console.error('Failed to listen to download-progress:', error);
		return () => {};
	}
}

/**
 * Listen to download status events
 */
export async function listenDownloadStatus(
	callback: (status: { status: string; file: string; url: string }) => void
): Promise<() => void> {
	if (!isTauriAvailable()) {
		return () => {};
	}

	try {
		const { listen } = await import('@tauri-apps/api/event');
		const unlisten = await listen('download-status', (event) => {
			callback(event.payload as { status: string; file: string; url: string });
		});
		return unlisten;
	} catch (error) {
		console.error('Failed to listen to download-status:', error);
		return () => {};
	}
}

/**
 * Listen to download complete events
 */
export async function listenDownloadComplete(
	callback: (result: { completed: string[]; failed: string[] }) => void
): Promise<() => void> {
	if (!isTauriAvailable()) {
		return () => {};
	}

	try {
		const { listen } = await import('@tauri-apps/api/event');
		const unlisten = await listen('download-complete', (event) => {
			callback(event.payload as { completed: string[]; failed: string[] });
		});
		return unlisten;
	} catch (error) {
		console.error('Failed to listen to download-complete:', error);
		return () => {};
	}
}

// Instance Management Commands
export const instanceCommands = {
	/**
	 * Add a new instance
	 */
	add: async (instance: Instance): Promise<string> => {
		return invoke('add_instance', { instance });
	},

	/**
	 * Remove an instance
	 */
	remove: async (id: string): Promise<string> => {
		return invoke('remove_instance', { id });
	},

	/**
	 * Get all instances
	 */
	getAll: async (): Promise<Instance[]> => {
		return invoke('get_instances');
	},

	/**
	 * Connect to an instance
	 */
	connect: async (id: string): Promise<string> => {
		return invoke('connect_to_instance', { id });
	},

	/**
	 * Check instance status
	 */
	checkStatus: async (id: string): Promise<string> => {
		return invoke('check_instance_status', { id });
	}
};

// Config Commands - for app configuration management
export const configCommands = {
	/**
	 * Get app configuration
	 */
	get: async (): Promise<AppConfig> => {
		return invoke('get_app_config');
	},

	/**
	 * Update app configuration
	 */
	update: async (config: AppConfig): Promise<void> => {
		return invoke('update_app_config', { config });
	},

	/**
	 * Mark setup as completed
	 */
	setSetupCompleted: async (
		mode: 'local' | 'remote',
		remoteUrl?: string,
		remoteUsername?: string,
		remotePassword?: string
	): Promise<void> => {
		return invoke('set_setup_completed', {
			mode,
			remote_url: remoteUrl,
			remote_username: remoteUsername,
			remote_password: remotePassword
		});
	}
};

// Proxy Commands - for bypassing CORS by using Tauri IPC
interface ProxyRequest {
	method: string;
	path: string;
	headers?: [string, string][];
	body?: string;
}

interface ProxyResponse {
	status: number;
	status_text: string;
	headers: [string, string][];
	body: string;
}

export const proxyCommands = {
	/**
	 * Proxy an HTTP request through Tauri IPC to the Python backend
	 * This bypasses browser CORS restrictions
	 */
	request: async (request: ProxyRequest): Promise<ProxyResponse> => {
		return invoke('proxy_request', { request });
	},

	/**
	 * Fetch an image through Tauri IPC (bypasses CORS, handles cookies)
	 * Returns base64 encoded image data
	 */
	fetchImage: async (path: string, remoteUrl?: string): Promise<{ mime_type: string; data: string }> => {
		return invoke('fetch_image', { path, remoteUrl });
	}
};

/**
 * Export all commands as a unified API
 */
export const tauriCommands = {
	backend: backendCommands,
	download: downloadCommands,
	instances: instanceCommands,
	config: configCommands,
	proxy: proxyCommands
};

/**
 * Custom fetch implementation that uses Tauri IPC proxy
 * This replaces the browser's fetch for API calls in Tauri desktop app
 */
export function createTauriFetch(originalFetch: typeof fetch) {
	return async (url: string, options: RequestInit = {}): Promise<Response> => {
		const backendUrl = 'http://127.0.0.1:8080';
		const apiBase = `${backendUrl}/api`;

		// Check if it's an image request in remote mode
		const isRemoteMode = !!(window as any).REMOTE_BACKEND_URL;
		if (isRemoteMode && (url.includes('/profile/image') || url.includes('/models/model/profile/image'))) {
			try {
				const remoteBackendUrl = (window as any).REMOTE_BACKEND_URL as string;
				const path = url.replace(remoteBackendUrl, '').replace(backendUrl, '');
				const result = await proxyCommands.fetchImage(path, remoteBackendUrl);
				const dataUrl = `data:${result.mime_type};base64,${result.data}`;
				// Convert base64 to blob
				const fetchResponse = await originalFetch(dataUrl);
				return fetchResponse;
			} catch (error) {
				console.error('[TauriFetch] Failed to fetch image:', error);
				// Fall through to regular fetch
			}
		}

		// If it's not a backend API request, use regular fetch
		if (!url.startsWith(apiBase) && !url.startsWith(backendUrl)) {
			return originalFetch(url, options);
		}

		// For remote mode, we still use IPC proxy but need to add special headers
		const remoteAuth = (window as any).REMOTE_BACKEND_AUTH as { username: string; password: string } | undefined;

		// Convert URL to path (remove the base URL)
		const path = url.replace(backendUrl, '');

		// Extract headers
		const headers: [string, string][] = [];

		// Add remote server URL header for IPC proxy to know where to forward
		if (isRemoteMode) {
			headers.push(['X-Remote-Backend-URL', (window as any).REMOTE_BACKEND_URL]);
		}

		// Add Basic Auth header if configured
		if (isRemoteMode && remoteAuth) {
			const credentials = btoa(`${remoteAuth.username}:${remoteAuth.password}`);
			headers.push(['X-Remote-Auth', `Basic ${credentials}`]);
		}

		// Add original request headers
		if (options.headers) {
			const headersInit = options.headers as HeadersInit;
			if (headersInit instanceof Headers) {
				headersInit.forEach((value, key) => {
					headers.push([key, value]);
				});
			} else if (Array.isArray(headersInit)) {
				for (const [key, value] of headersInit) {
					headers.push([key, value]);
				}
			} else {
				for (const [key, value] of Object.entries(headersInit)) {
					headers.push([key, value as string]);
				}
			}
		}

		// Get body as string
		let body: string | undefined;
		if (options.body) {
			body = typeof options.body === 'string'
				? options.body
				: JSON.stringify(options.body);
		}

		// Make the proxied request
		const proxyResponse = await proxyCommands.request({
			method: options.method || 'GET',
			path,
			headers: headers.length > 0 ? headers : undefined,
			body
		});

		// Convert proxy response to a Response-like object
		return new Response(proxyResponse.body, {
			status: proxyResponse.status,
			statusText: proxyResponse.status_text,
			headers: new Headers(proxyResponse.headers)
		});
	};
}

/**
 * Get the appropriate fetch function based on platform
 * In Tauri desktop, use the proxied fetch to avoid CORS issues
 */
export function getFetchFunction(originalFetch: typeof fetch): typeof fetch {
	if (isTauriAvailable()) {
		return createTauriFetch(originalFetch) as unknown as typeof fetch;
	}
	return originalFetch;
}
