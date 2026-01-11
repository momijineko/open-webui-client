/**
 * 平台检测和配置工具
 */

export const PLATFORM = {
	/**
	 * 是否为 Tauri 桌面端
	 */
	isDesktop: typeof window !== 'undefined' && '__TAURI__' in window,

	/**
	 * 是否为 Capacitor 移动端
	 */
	isMobile: typeof window !== 'undefined' && 'Capacitor' in window,

	/**
	 * 是否为纯 Web 环境
	 */
	isWeb: typeof window !== 'undefined' && !('__TAURI__' in window) && !('Capacitor' in window)
};

/**
 * 获取当前平台标识
 */
export function getPlatform(): 'desktop' | 'mobile' | 'web' {
	if (PLATFORM.isDesktop) return 'desktop';
	if (PLATFORM.isMobile) return 'mobile';
	return 'web';
}

/**
 * 获取 API 基础 URL
 * 桌面端：连接本地后端
 * 移动端：从配置读取或使用当前连接的服务器
 * Web 端：使用当前域名
 */
export async function getApiBaseUrl(): Promise<string> {
	const platform = getPlatform();

	switch (platform) {
		case 'desktop':
			// 桌面端：连接本地后端
			// TODO: 从 Tauri 配置获取端口
			return 'http://127.0.0.1:8080/api/v1';

		case 'mobile':
			// 移动端：从配置读取
			// TODO: 从 Capacitor Preferences 读取
			return '/api/v1';

		case 'web':
		default:
			// Web 端：使用当前域名
			return window.location.origin + '/api/v1';
	}
}

/**
 * 获取 WebSocket URL
 */
export async function getWebSocketUrl(): Promise<string> {
	const platform = getPlatform();

	switch (platform) {
		case 'desktop':
			// 桌面端：本地后端
			return 'ws://127.0.0.1:8080';

		case 'mobile':
			// 移动端：从配置读取
			// TODO: 从 Capacitor Preferences 读取并转换协议
			return window.location.origin.replace(/^http/, 'ws');

		case 'web':
		default:
			// Web 端：使用当前域名
			const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
			return `${protocol}//${window.location.host}`;
	}
}

/**
 * 是否支持本地后端
 */
export function supportsLocalBackend(): boolean {
	return PLATFORM.isDesktop;
}

/**
 * 是否需要下载组件
 */
export function needsComponentDownload(): boolean {
	return PLATFORM.isDesktop;
}
