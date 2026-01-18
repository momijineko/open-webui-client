/**
 * Platform detection and configuration utilities
 */

export const PLATFORM = {
	/**
	 * Whether running on Tauri desktop
	 */
	isDesktop: typeof window !== 'undefined' && '__TAURI__' in window,

	/**
	 * Whether running on Capacitor mobile
	 */
	isMobile: typeof window !== 'undefined' && 'Capacitor' in window,

	/**
	 * Whether running in pure Web environment
	 */
	isWeb: typeof window !== 'undefined' && !('__TAURI__' in window) && !('Capacitor' in window)
};

/**
 * Get current platform identifier
 */
export function getPlatform(): 'desktop' | 'mobile' | 'web' {
	if (PLATFORM.isDesktop) return 'desktop';
	if (PLATFORM.isMobile) return 'mobile';
	return 'web';
}

/**
 * Get API base URL
 * Desktop: Connect to local backend
 * Mobile: Read from config or use current connected server
 * Web: Use current domain
 */
export async function getApiBaseUrl(): Promise<string> {
	const platform = getPlatform();

	switch (platform) {
		case 'desktop':
			// Desktop: Connect to local backend
			// TODO: Get port from Tauri config
			return 'http://127.0.0.1:8080/api/v1';

		case 'mobile':
			// Mobile: Read from config
			// TODO: Read from Capacitor Preferences
			return '/api/v1';

		case 'web':
		default:
			// Web: Use current domain
			return window.location.origin + '/api/v1';
	}
}

/**
 * Get WebSocket URL
 */
export async function getWebSocketUrl(): Promise<string> {
	const platform = getPlatform();

	switch (platform) {
		case 'desktop':
			// Desktop: Local backend
			return 'ws://127.0.0.1:8080';

		case 'mobile':
			// Mobile: Read from config
			// TODO: Read from Capacitor Preferences and convert protocol
			return window.location.origin.replace(/^http/, 'ws');

		case 'web':
		default:
			// Web: Use current domain
			const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
			return `${protocol}//${window.location.host}`;
	}
}

/**
 * Whether local backend is supported
 */
export function supportsLocalBackend(): boolean {
	return PLATFORM.isDesktop;
}

/**
 * Whether component download is needed
 */
export function needsComponentDownload(): boolean {
	return PLATFORM.isDesktop;
}
