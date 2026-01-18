/**
 * API utility for different platforms
 */

import { getApiBaseUrl, getWebSocketUrl, getPlatform } from '../constants/platforms';

/**
 * Platform-aware API class
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
 * Send API request
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
 * GET request
	 */
	async get<T = any>(endpoint: string): Promise<T> {
		const response = await this.request(endpoint, { method: 'GET' });
		if (!response.ok) {
			throw new Error(`API Error: ${response.status}`);
		}
		return response.json();
	}

	/**
 * POST request
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
 * PUT request
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
 * DELETE request
	 */
	async delete<T = any>(endpoint: string): Promise<T> {
		const response = await this.request(endpoint, { method: 'DELETE' });
		if (!response.ok) {
			throw new Error(`API Error: ${response.status}`);
		}
		return response.json();
	}

	/**
 * Get WebSocket URL
	 */
	async getWebSocketUrl(): Promise<string> {
		if (!this.wsUrl) {
			await this.init();
		}
		return this.wsUrl;
	}

	/**
 * Get base URL
	 */
	getBaseUrl(): string {
		return this.baseUrl;
	}

	/**
 * Get current platform
	 */
	getPlatform(): 'desktop' | 'mobile' | 'web' {
		return getPlatform();
	}

	/**
 * Is desktop
	 */
	isDesktop(): boolean {
		return getPlatform() === 'desktop';
	}

	/**
 * Is mobile
	 */
	isMobile(): boolean {
		return getPlatform() === 'mobile';
	}

	/**
 * Is web
	 */
	isWeb(): boolean {
		return getPlatform() === 'web';
	}
}

// Create global instance
export const platformAPI = new PlatformAPI();

/**
 * Initialize API (called on app startup)
 */
export async function initAPI() {
	await platformAPI.init();
}
