import { browser, dev } from '$app/environment';
// import { version } from '../../package.json';

// Tauri 2.x 平台检测 - 使用更可靠的方法
let isTauriDesktop = false;
if (typeof window !== 'undefined') {
	// 检查 __TAURI__ 或 __TAURI_INTERNALS__ 对象 (Tauri 2.x)
	isTauriDesktop = !!(window as any).__TAURI__ || '__TAURI_INTERNALS__' in window;
}

export const APP_NAME = 'Open WebUI';

// 平台检测
export const PLATFORM = {
	isDesktop: isTauriDesktop,
	isMobile: typeof window !== 'undefined' && 'Capacitor' in window,
	isWeb: typeof window !== 'undefined' && !isTauriDesktop && !('Capacitor' in window)
};

// 根据平台动态配置 URL
export const WEBUI_HOSTNAME = browser
	? (PLATFORM.isDesktop ? '127.0.0.1:8080' : (dev ? `${location.hostname}:8080` : ``))
	: '';

export const WEBUI_BASE_URL = browser
	? (PLATFORM.isDesktop ? `http://${WEBUI_HOSTNAME}` : (dev ? `http://${WEBUI_HOSTNAME}` : ``))
	: '';

/**
 * Get the backend base URL at runtime
 * Supports remote mode configuration via window.REMOTE_BACKEND_URL
 */
export function getBackendBaseUrl(): string {
	if (typeof window !== 'undefined' && (window as any).REMOTE_BACKEND_URL) {
		return (window as any).REMOTE_BACKEND_URL;
	}
	return WEBUI_BASE_URL;
}

/**
 * Get the backend API base URL at runtime
 */
export function getBackendApiBaseUrl(): string {
	const baseUrl = getBackendBaseUrl();
	return baseUrl ? `${baseUrl}/api/v1` : '';
}

/**
 * Get authenticated image URL by adding Basic Auth credentials as query parameters
 * This is needed for <img> tags which can't include custom headers
 */
export function getAuthenticatedImageUrl(imagePath: string): string {
	const baseUrl = getBackendBaseUrl();
	const auth = (window as any).REMOTE_BACKEND_AUTH as { username: string; password: string } | undefined;

	if (!baseUrl) return imagePath;

	let url = `${baseUrl}${imagePath}`;

	// Add Basic Auth as query parameters if remote mode is configured
	if (auth && auth.username && auth.password) {
		const separator = url.includes('?') ? '&' : '?';
		url += `${separator}username=${encodeURIComponent(auth.username)}&password=${encodeURIComponent(auth.password)}`;
	}

	return url;
}

export const WEBUI_API_BASE_URL = `${WEBUI_BASE_URL}/api/v1`;

export const OLLAMA_API_BASE_URL = `${WEBUI_BASE_URL}/ollama`;
export const OPENAI_API_BASE_URL = `${WEBUI_BASE_URL}/openai`;
export const AUDIO_API_BASE_URL = `${WEBUI_BASE_URL}/api/v1/audio`;
export const IMAGES_API_BASE_URL = `${WEBUI_BASE_URL}/api/v1/images`;
export const RETRIEVAL_API_BASE_URL = `${WEBUI_BASE_URL}/api/v1/retrieval`;

export const WEBUI_VERSION = APP_VERSION;
export const WEBUI_BUILD_HASH = APP_BUILD_HASH;
export const REQUIRED_OLLAMA_VERSION = '0.1.16';

export const SUPPORTED_FILE_TYPE = [
	'application/epub+zip',
	'application/pdf',
	'text/plain',
	'text/csv',
	'text/xml',
	'text/html',
	'text/x-python',
	'text/css',
	'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
	'application/octet-stream',
	'application/x-javascript',
	'text/markdown',
	'audio/mpeg',
	'audio/wav',
	'audio/ogg',
	'audio/x-m4a'
];

export const SUPPORTED_FILE_EXTENSIONS = [
	'md',
	'rst',
	'go',
	'py',
	'java',
	'sh',
	'bat',
	'ps1',
	'cmd',
	'js',
	'ts',
	'css',
	'cpp',
	'hpp',
	'h',
	'c',
	'cs',
	'htm',
	'html',
	'sql',
	'log',
	'ini',
	'pl',
	'pm',
	'r',
	'dart',
	'dockerfile',
	'env',
	'php',
	'hs',
	'hsc',
	'lua',
	'nginxconf',
	'conf',
	'm',
	'mm',
	'plsql',
	'perl',
	'rb',
	'rs',
	'db2',
	'scala',
	'bash',
	'swift',
	'vue',
	'svelte',
	'doc',
	'docx',
	'pdf',
	'csv',
	'txt',
	'xls',
	'xlsx',
	'pptx',
	'ppt',
	'msg'
];

export const PASTED_TEXT_CHARACTER_LIMIT = 1000;
