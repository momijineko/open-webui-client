<script lang="ts">
	import { onMount } from 'svelte';
	import { WEBUI_BASE_URL, PLATFORM } from '$lib/constants';
	import { runtimeBackendUrl } from '$lib/stores';
	import { proxyCommands } from '$lib/utils/tauri';
	import { isTauriAvailable } from '$lib/utils/tauri';

	export let className = 'size-8';
	export let src = PLATFORM.isDesktop ? '/static/favicon.png' : `${WEBUI_BASE_URL}/static/favicon.png`;

	let imageUrl = src;
	let mounted = false;

	// Use static path for desktop/Tauri environment
	const faviconPath = PLATFORM.isDesktop ? '/static/favicon.png' : `${WEBUI_BASE_URL}/static/favicon.png`;
	const userPath = PLATFORM.isDesktop ? '/static/user.png' : `${WEBUI_BASE_URL}/static/user.png`;

	// Get dynamic base URL for remote mode support
	$: backendBaseUrl = $runtimeBackendUrl;
	$: dynamicFaviconPath = PLATFORM.isDesktop ? '/static/favicon.png' : `${backendBaseUrl}/static/favicon.png`;
	$: dynamicUserPath = PLATFORM.isDesktop ? '/static/user.png' : `${backendBaseUrl}/static/user.png`;

	// Check if URL is from backend (needs auth)
	function isBackendUrl(url: string): boolean {
		return url.startsWith(backendBaseUrl) &&
			!url.startsWith('https://www.gravatar.com/avatar/') &&
			!url.startsWith('data:') &&
			!url.startsWith('/');
	}

	onMount(async () => {
		mounted = true;

		let targetUrl = src === ''
			? dynamicFaviconPath
			: src.startsWith(backendBaseUrl) ||
				  src.startsWith('https://www.gravatar.com/avatar/') ||
				  src.startsWith('data:') ||
				  src.startsWith('/')
				? src
				: dynamicUserPath;

		// In Tauri remote mode, fetch images through IPC proxy
		if (isTauriAvailable() && isBackendUrl(targetUrl) && (window as any).REMOTE_BACKEND_URL) {
			try {
				// Extract path from full URL
				const path = targetUrl.replace(backendBaseUrl, '');
				const result = await proxyCommands.fetchImage(path, (window as any).REMOTE_BACKEND_URL);
				imageUrl = `data:${result.mime_type};base64,${result.data}`;
			} catch (error) {
				console.error('[ProfileImage] Failed to fetch image:', error);
				imageUrl = targetUrl;
			}
		} else {
			imageUrl = targetUrl;
		}
	});
</script>

{#if mounted}
	<img
		aria-hidden="true"
		src={imageUrl}
		class="{className} object-cover rounded-full"
		alt="profile"
		draggable="false"
		on:error={(e) => {
			// Fallback to default image on error
			const target = e.currentTarget;
			if (!target.src.includes('favicon.png') && !target.src.includes('user.png')) {
				target.src = userPath;
			}
		}}
	/>
{:else}
	<div class={className} />
{/if}
