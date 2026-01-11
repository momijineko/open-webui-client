/**
 * OpenWebUI 实例类型定义
 */

export interface OpenWebUIInstance {
	id: string;
	name: string;
	type: 'local' | 'remote';
	url: string;
	port?: number;
	status: 'online' | 'offline' | 'unknown';
	lastUsed: number;
	autoConnect: boolean;
	isLocalBackend: boolean;
}

export interface InstanceConnection {
	instanceId: string;
	connected: boolean;
	lastError?: string;
	latency?: number;
}

export interface InstanceConfig {
	instances: OpenWebUIInstance[];
	currentInstance: string | null;
	settings: {
		checkInterval: number;
		autoReconnect: boolean;
		maxRetries: number;
	};
}

export interface DownloadComponent {
	id: string;
	name: string;
	size: number;
	sizeDisplay: string;
	url: string;
	required: boolean;
	checked?: boolean;
	progress?: number;
	downloaded?: boolean;
}

export interface MirrorSource {
	id: string;
	name: string;
	url: string;
	priority: number;
	regions: string[];
}

export interface DownloadConfig {
	mirrorSource: 'auto' | 'custom' | string;
	customMirrors: string[];
	enableProxy: boolean;
	proxyUrl: string;
	downloadOptional: boolean;
}
