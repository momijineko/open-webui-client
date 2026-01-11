import { proxyCommands, isTauriAvailable } from '$lib/utils/tauri';

/**
 * Svelte action that loads images through IPC proxy in remote mode
 * Automatically handles authentication for backend images
 *
 * Usage: <img use:authImage src={url} alt="..." />
 */
export function authImage(node: HTMLImageElement) {
	async function loadImage() {
		const url = node.src;
		const isRemoteMode = isTauriAvailable() && (window as any).REMOTE_BACKEND_URL;
		const isBackendImage = url.includes('/profile/image') || url.includes('/models/model/profile/image') || url.includes('/users/');

		if (isRemoteMode && isBackendImage) {
			try {
				const remoteBackendUrl = (window as any).REMOTE_BACKEND_URL as string;
				// Extract path from URL
				const path = url.replace(remoteBackendUrl, '').replace('http://127.0.0.1:8080', '');
				const result = await proxyCommands.fetchImage(path, remoteBackendUrl);
				node.src = `data:${result.mime_type};base64,${result.data}`;
			} catch (error) {
				console.error('[authImage] Failed to load image:', error);
			}
		}
	}

	// Load image when element is mounted
	loadImage();

	return {
		update() {
			// Reload if src changes
			loadImage();
		}
	};
}
