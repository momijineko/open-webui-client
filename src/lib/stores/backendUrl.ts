import { writable, derived, get } from 'svelte/store';
import { isTauriAvailable } from '$lib/utils/tauri';

export type BackendMode = 'local' | 'remote';

interface BackendUrlState {
	mode: BackendMode;
	url: string;
	username?: string;
	password?: string;
}

const createBackendUrlStore = () => {
	const { subscribe, set, update } = writable<BackendUrlState>({
		mode: 'local',
		url: 'http://127.0.0.1:8080',
		username: undefined,
		password: undefined
	});

	return {
		subscribe,
		set,
		update,
		init: async () => {
			if (!isTauriAvailable()) {
				return;
			}

			try {
				const { invoke } = await import('@tauri-apps/api/core');
				const config = await invoke('get_app_config') as {
					setup_mode: string | null;
					remote_url: string | null;
					remote_username: string | null;
					remote_password: string | null;
				};

				if (config.setup_mode === 'remote' && config.remote_url) {
					set({
						mode: 'remote',
						url: config.remote_url,
						username: config.remote_username || undefined,
						password: config.remote_password || undefined
					});
				}
			} catch (error) {
				console.error('[BackendUrl] Failed to load config:', error);
			}
		},
		getBaseUrl: () => {
			const state = get(backendUrlStore);
			return state.url;
		},
		getAuthHeaders: () => {
			const state = get(backendUrlStore);
			if (state.mode === 'remote' && state.username && state.password) {
				const credentials = btoa(`${state.username}:${state.password}`);
				return {
					Authorization: `Basic ${credentials}`
				};
			}
			return {};
		}
	};
};

export const backendUrlStore = createBackendUrlStore();

// 导出一个 derived store 来获取当前的基础 URL
export const backendBaseUrl = derived(backendUrlStore, ($state) => $state.url);
