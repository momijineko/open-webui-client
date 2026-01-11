/**
 * API 调用工具 - 适配不同平台
 */

import { getApiBaseUrl, getWebSocketUrl, getPlatform } from '../constants/platforms';

/**
 * 平台感知的 API 调用类
 */
export class PlatformAPI {
	private baseUrl: string = '';
	private wsUrl: string = '';
	private token: string | null = null;

	async init() {
		this.baseUrl = await getApiBaseUrl();
		this.wsUrl = await getWebSocketUrl();
		this.token = localStorage.getItem('token');
	}

	/**
	 * 发送 API 请求
	 */
	async request(endpoint: string, options?: RequestInit): Promise<Response> {
		if (!this.baseUrl) {
			await this.init();
		}

		const url = `${this.baseUrl}${endpoint}`;
		const token = localStorage.getItem('token');

		const response = await fetch(url, {
			...options,
			headers: {
				'Content-Type': 'application/json',
				...(token && { Authorization: `Bearer ${token}` }),
				...options?.headers,
			},
		});

		return response;
	}

	/**
	 * GET 请求
	 */
	async get<T = any>(endpoint: string): Promise<T> {
		const response = await this.request(endpoint, { method: 'GET' });
		if (!response.ok) {
			throw new Error(`API Error: ${response.status}`);
		}
		return response.json();
	}

	/**
	 * POST 请求
	 */
	async post<T = any>(endpoint: string, data?: any): Promise<T> {
		const response = await this.request(endpoint, {
			method: 'POST',
			body: JSON.stringify(data),
		});
		if (!response.ok) {
			throw new Error(`API Error: ${response.status}`);
		}
		return response.json();
	}

	/**
	 * PUT 请求
	 */
	async put<T = any>(endpoint: string, data?: any): Promise<T> {
		const response = await this.request(endpoint, {
			method: 'PUT',
			body: JSON.stringify(data),
		});
		if (!response.ok) {
			throw new Error(`API Error: ${response.status}`);
		}
		return response.json();
	}

	/**
	 * DELETE 请求
	 */
	async delete<T = any>(endpoint: string): Promise<T> {
		const response = await this.request(endpoint, { method: 'DELETE' });
		if (!response.ok) {
			throw new Error(`API Error: ${response.status}`);
		}
		return response.json();
	}

	/**
	 * 获取 WebSocket URL
	 */
	async getWebSocketUrl(): Promise<string> {
		if (!this.wsUrl) {
			await this.init();
		}
		return this.wsUrl;
	}

	/**
	 * 获取基础 URL
	 */
	getBaseUrl(): string {
		return this.baseUrl;
	}

	/**
	 * 获取当前平台
	 */
	getPlatform(): 'desktop' | 'mobile' | 'web' {
		return getPlatform();
	}

	/**
	 * 是否为桌面端
	 */
	isDesktop(): boolean {
		return getPlatform() === 'desktop';
	}

	/**
	 * 是否为移动端
	 */
	isMobile(): boolean {
		return getPlatform() === 'mobile';
	}

	/**
	 * 是否为 Web 端
	 */
	isWeb(): boolean {
		return getPlatform() === 'web';
	}
}

// 创建全局实例
export const platformAPI = new PlatformAPI();

/**
 * 初始化 API（在应用启动时调用）
 */
export async function initAPI() {
	await platformAPI.init();
}
