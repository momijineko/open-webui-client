<script lang="ts">
	import Tooltip from '$lib/components/common/Tooltip.svelte';
	import { PLATFORM } from '$lib/constants';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { tauriCommands, isTauriAvailable } from '$lib/utils/tauri';
	import { onMount } from 'svelte';

	let selected = '';

	// Set selected state based on current route
	$: if ($page.url.pathname === '/') {
		selected = 'chat';
	} else if ($page.url.pathname === '/setup') {
		selected = 'setup';
	} else if ($page.url.pathname.startsWith('/test-tauri')) {
		selected = 'test';
	}

	const navigateTo = (path: string) => {
		selected = path;
		goto(path);
	};

	let backendStatus = {
		is_running: false,
		port: null,
		pid: null
	};

	const checkBackendStatus = async () => {
		if (isTauriAvailable()) {
			try {
				backendStatus = await tauriCommands.backend.checkStatus();
			} catch (error) {
				console.error('Failed to check backend status:', error);
			}
		}
	};

	onMount(() => {
		checkBackendStatus();
		// Check backend status every 5 seconds
		const interval = setInterval(checkBackendStatus, 5000);
		return () => clearInterval(interval);
	});
</script>

{#if PLATFORM.isDesktop}
	<div class="tauri-sidebar min-w-[4.5rem] bg-gray-50 dark:bg-gray-950 flex gap-2.5 flex-col pt-8">
		<!-- Logo / Home -->
		<div class="flex justify-center relative">
			{#if selected === 'home' || selected === 'chat'}
				<div class="absolute top-0 left-0 flex h-full">
					<div class="my-auto rounded-r-lg w-1 h-8 bg-black dark:bg-white"></div>
				</div>
			{/if}

			<Tooltip content="Chat" placement="right">
				<button
					class="cursor-pointer {selected === 'home' || selected === 'chat' ? 'rounded-2xl' : 'rounded-full'}"
					on:click={() => navigateTo('/')}
				>
					<img
						src="/static/splash.png"
						class="size-11 dark:invert p-0.5"
						alt="logo"
						draggable="false"
					/>
				</button>
			</Tooltip>
		</div>

		<div class="-mt-1 border-[1.5px] border-gray-100 dark:border-gray-900 mx-4"></div>

		<!-- Backend Status -->
		<div class="flex justify-center relative group">
			<Tooltip content={backendStatus.is_running ? 'Backend Running' : 'Backend Stopped'} placement="right">
				<button
					class="cursor-pointer p-2 rounded-full hover:bg-gray-200 dark:hover:bg-gray-800 transition-colors"
					on:click={checkBackendStatus}
				>
					<div class="relative">
						<div
							class="size-4 rounded-full {backendStatus.is_running
								? 'bg-green-500 animate-pulse'
								: 'bg-gray-400 dark:bg-gray-600'}"
						></div>
					</div>
				</button>
			</Tooltip>
		</div>

		<!-- Setup Wizard -->
		<div class="flex justify-center relative group">
			{#if selected === 'setup'}
				<div class="absolute top-0 left-0 flex h-full">
					<div class="my-auto rounded-r-lg w-1 h-8 bg-black dark:bg-white"></div>
				</div>
			{/if}

			<Tooltip content="Setup Wizard" placement="right">
				<button
					class="cursor-pointer p-2 {selected === 'setup'
						? 'rounded-2xl bg-gray-200 dark:bg-gray-800'
						: 'rounded-full hover:bg-gray-200 dark:hover:bg-gray-800'}"
					on:click={() => navigateTo('/setup')}
				>
					<svg
						class="size-5 text-gray-700 dark:text-gray-300"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
						/>
					</svg>
				</button>
			</Tooltip>
		</div>

		<!-- Test Page (Dev Mode Only) -->
		{#if import.meta.env.DEV}
			<div class="flex justify-center relative group">
				{#if selected === 'test'}
					<div class="absolute top-0 left-0 flex h-full">
						<div class="my-auto rounded-r-lg w-1 h-8 bg-black dark:bg-white"></div>
					</div>
				{/if}

				<Tooltip content="Test" placement="right">
					<button
						class="cursor-pointer p-2 {selected === 'test'
							? 'rounded-2xl bg-gray-200 dark:bg-gray-800'
							: 'rounded-full hover:bg-gray-200 dark:hover:bg-gray-800'}"
						on:click={() => navigateTo('/test-tauri')}
					>
						<svg
							class="size-5 text-gray-700 dark:text-gray-300"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
							/>
						</svg>
					</button>
				</Tooltip>
			</div>
		{/if}
	</div>
{/if}
