<script lang="ts">
	export let currentStep: number;
	export let selectedMode: 'local' | 'remote' | null = null;
	export let onConfirm: (config: { url: string; username?: string; password?: string }) => void;
	export let onBack: () => void;

	let serverUrl = '';
	let username = '';
	let password = '';
	let needsAuth = false; // 是否需要认证
	let isValidUrl = true;
	let urlError = '';
	let authError = ''; // 认证信息错误
	let isTesting = false;
	let hasTested = false; // 是否已执行过测试
	let testResult: { success: boolean; message: string } | null = null;

	const exampleServers = [
		{ name: '示例: http://localhost:8080', url: 'http://localhost:8080' },
		{ name: '示例: http://192.168.1.100:3000', url: 'http://192.168.1.100:3000' }
	];

	const validateUrl = (url: string): boolean => {
		if (!url.trim()) {
			urlError = '请输入服务器地址';
			return false;
		}

		try {
			const parsed = new URL(url);
			if (!['http:', 'https:'].includes(parsed.protocol)) {
				urlError = '请使用 http:// 或 https:// 开头的地址';
				return false;
			}
			urlError = '';
			return true;
		} catch {
			urlError = '请输入有效的 URL 地址';
			return false;
		}
	};

	const validateAuth = (): boolean => {
		if (!needsAuth) {
			authError = '';
			return true;
		}

		if (!username.trim()) {
			authError = '请输入用户名';
			return false;
		}

		if (!password) {
			authError = '请输入密码';
			return false;
		}

		authError = '';
		return true;
	};

	const handleUrlInput = () => {
		isValidUrl = validateUrl(serverUrl);
		testResult = null;
		hasTested = false;
	};

	const handleAuthInput = () => {
		validateAuth();
		testResult = null;
		hasTested = false;
	};

	const fillExample = (url: string) => {
		serverUrl = url;
		isValidUrl = true;
		urlError = '';
		testResult = null;
		hasTested = false;
	};

	const handleTestConnection = async () => {
		if (!validateUrl(serverUrl)) {
			isValidUrl = false;
			return;
		}

		// 如果需要认证，先验证认证信息
		if (needsAuth && !validateAuth()) {
			return;
		}

		isTesting = true;
		testResult = null;

		try {
			// 测试连接 - 尝试访问服务器的健康检查端点
			const testUrl = serverUrl.replace(/\/$/, '') + '/api/config';

			// 构建 headers
			const headers: Record<string, string> = {
				'Content-Type': 'application/json'
			};

			// 如果需要认证且提供了用户名和密码，添加 Basic Auth
			if (needsAuth && username && password) {
				const credentials = btoa(`${username}:${password}`);
				headers['Authorization'] = `Basic ${credentials}`;
			}

			const response = await fetch(testUrl, {
				method: 'GET',
				headers
			});

			if (response.ok) {
				testResult = {
					success: true,
					message: '连接成功！服务器可以访问'
				};
			} else if (response.status === 401) {
				testResult = {
					success: false,
					message: '认证失败：用户名或密码错误'
				};
			} else {
				testResult = {
					success: false,
					message: `服务器返回错误: ${response.status} ${response.statusText}`
				};
			}
		} catch (error) {
			testResult = {
				success: false,
				message: `连接失败: ${(error as Error).message}`
			};
		} finally {
			isTesting = false;
			hasTested = true; // 标记已执行过测试
		}
	};

	const handleConfirm = () => {
		if (!validateUrl(serverUrl)) {
			isValidUrl = false;
			return;
		}

		// 如果需要认证，验证认证信息
		if (needsAuth && !validateAuth()) {
			return;
		}

		// 检查是否已执行测试且测试成功
		if (!hasTested) {
			alert('请先测试连接');
			return;
		}

		if (!testResult?.success) {
			alert('测试连接未通过，请检查服务器地址和认证信息');
			return;
		}

		// 移除末尾的斜杠
		const cleanUrl = serverUrl.replace(/\/$/, '');

		onConfirm({
			url: cleanUrl,
			username: needsAuth ? username.trim() || undefined : undefined,
			password: needsAuth ? password : undefined
		});
	};
</script>

{#if currentStep === 4 && selectedMode === 'remote'}
	<div class="remote-config">
		<h1>配置远程服务器</h1>
		<p>输入您要连接的 OpenWebUI 服务器地址</p>

		<div class="config-form">
			<!-- 服务器地址配置 -->
			<div class="config-section">
				<div class="section-header">
					<h2>服务器地址</h2>
					<span class="section-description">
						输入您自己部署的 OpenWebUI 服务器地址
					</span>
				</div>

				<div class="form-group">
					<label>服务器 URL</label>
					<input
						type="text"
						placeholder="https://your-server.com"
						bind:value={serverUrl}
						on:input={handleUrlInput}
						class="text-input"
						class:invalid={!isValidUrl}
					/>
					<span class="input-hint">
						例如: http://localhost:8080 或 https://your-server.com
					</span>
					{#if !isValidUrl}
						<span class="error-message">{urlError}</span>
					{/if}
				</div>

				<div class="form-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={needsAuth} />
						<span>服务器需要认证</span>
					</label>
				</div>

				<div class="form-group">
					<label>快速填入示例</label>
					<div class="example-buttons">
						{#each exampleServers as example}
							<button
								class="btn-example"
								on:click={() => fillExample(example.url)}
							>
								{example.name}
							</button>
						{/each}
					</div>
				</div>

				<button
					class="btn-test"
					on:click={handleTestConnection}
					disabled={!serverUrl || isTesting}
				>
					{#if isTesting}
						测试中...
					{:else}
						🔍 测试连接
					{/if}
				</button>

				{#if testResult}
					<div class="test-result" class:success={testResult.success} class:error={!testResult.success}>
						{testResult.success ? '✓' : '✗'} {testResult.message}
					</div>
				{/if}
			</div>

			<!-- 认证配置 -->
			{#if needsAuth}
				<div class="config-section">
					<div class="section-header">
						<h2>认证信息</h2>
						<span class="section-description">
							输入 Basic Auth 用户名和密码
						</span>
					</div>

					<div class="form-group">
						<label>用户名</label>
						<input
							type="text"
							placeholder="admin"
							bind:value={username}
							on:input={handleAuthInput}
							class="text-input"
						/>
					</div>

					<div class="form-group">
						<label>密码</label>
						<input
							type="password"
							placeholder="••••••••"
							bind:value={password}
							on:input={handleAuthInput}
							class="text-input"
						/>
					</div>

					{#if authError}
						<span class="error-message">{authError}</span>
					{/if}
				</div>
			{/if}

			<!-- 提示信息 -->
			<div class="info-box">
				<h3>💡 提示</h3>
				<ul>
					<li>确保您的服务器可以从当前设备访问</li>
					<li>如果使用自签名证书，可能需要在浏览器中先信任该证书</li>
					<li>Basic Auth 凭据将安全保存在本地</li>
					<li>配置后可以在设置页面随时修改</li>
				</ul>
			</div>
		</div>

		<div class="actions">
			<button class="btn-secondary" on:click={onBack}>上一步</button>
			<button
				class="btn-primary"
				on:click={handleConfirm}
				disabled={!isValidUrl || !serverUrl || !hasTested || !testResult?.success}
			>
				下一步
			</button>
		</div>
	</div>
{/if}

<style>
	.remote-config {
		background: white;
		border-radius: 1rem;
		padding: 2rem;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
		max-height: 80vh;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}

	/* 自定义滚动条样式 */
	.remote-config::-webkit-scrollbar {
		width: 8px;
	}

	.remote-config::-webkit-scrollbar-track {
		background: #f1f1f1;
		border-radius: 4px;
	}

	.remote-config::-webkit-scrollbar-thumb {
		background: #888;
		border-radius: 4px;
	}

	.remote-config::-webkit-scrollbar-thumb:hover {
		background: #555;
	}

	.remote-config h1 {
		font-size: 1.75rem;
		font-weight: 700;
		color: #1f2937;
		margin-bottom: 0.5rem;
		text-align: center;
		flex-shrink: 0;
	}

	.remote-config p {
		color: #6b7280;
		text-align: center;
		margin-bottom: 2rem;
		flex-shrink: 0;
	}

	.config-form {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		flex: 1;
		overflow-y: auto;
		padding-right: 0.5rem;
	}

	.config-section {
		padding: 1.25rem;
		background: #f9fafb;
		border-radius: 0.75rem;
		border: 1px solid #e5e7eb;
	}

	.section-header {
		margin-bottom: 1rem;
	}

	.section-header h2 {
		font-size: 1rem;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 0.25rem;
	}

	.section-description {
		font-size: 0.8rem;
		color: #6b7280;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 1rem;
	}

	.form-group:last-child {
		margin-bottom: 0;
	}

	.form-group label {
		font-weight: 500;
		color: #374151;
		font-size: 0.9rem;
	}

	.form-group select,
	.form-group input,
	.text-input {
		padding: 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.5rem;
		font-size: 0.9rem;
		font-family: inherit;
	}

	.text-input.invalid {
		border-color: #ef4444;
	}

	.form-group select:focus,
	.form-group input:focus,
	.text-input:focus {
		outline: none;
		border-color: #6366f1;
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
	}

	.input-hint {
		font-size: 0.75rem;
		color: #9ca3af;
	}

	.error-message {
		font-size: 0.75rem;
		color: #ef4444;
	}

	.btn-test {
		padding: 0.75rem 1.5rem;
		background: #f3f4f6;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.5rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
		width: 100%;
	}

	.btn-test:hover:not(:disabled) {
		background: #e5e7eb;
		border-color: #9ca3af;
	}

	.btn-test:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.example-buttons {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.btn-example {
		padding: 0.5rem 0.75rem;
		background: #e5e7eb;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.85rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn-example:hover {
		background: #d1d5db;
		border-color: #9ca3af;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
		user-select: none;
	}

	.checkbox-label input[type='checkbox'] {
		width: 1.1rem;
		height: 1.1rem;
		cursor: pointer;
	}

	.checkbox-label span {
		font-weight: 500;
		color: #374151;
		font-size: 0.9rem;
	}

	.auth-fields {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		margin-top: 1rem;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.5rem;
		border: 1px solid #e5e7eb;
	}

	.test-result {
		padding: 0.75rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
		margin-top: 0.5rem;
	}

	.test-result.success {
		background: #d1fae5;
		color: #065f46;
		border: 1px solid #6ee7b7;
	}

	.test-result.error {
		background: #fee2e2;
		color: #991b1b;
		border: 1px solid #fca5a5;
	}

	.info-box {
		background: #eff6ff;
		border: 1px solid #bfdbfe;
		border-radius: 0.75rem;
		padding: 1rem;
	}

	.info-box h3 {
		font-size: 0.9rem;
		font-weight: 600;
		color: #1e40af;
		margin: 0 0 0.75rem 0;
	}

	.info-box ul {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.info-box li {
		font-size: 0.85rem;
		color: #1e3a8a;
		padding: 0.35rem 0;
		padding-left: 1.5rem;
		position: relative;
	}

	.info-box li::before {
		content: '•';
		position: absolute;
		left: 0.5rem;
		color: #3b82f6;
		font-weight: bold;
	}

	.actions {
		display: flex;
		gap: 1rem;
		justify-content: flex-end;
		margin-top: 1rem;
		flex-shrink: 0;
	}

	.btn-primary,
	.btn-secondary {
		padding: 0.75rem 1.5rem;
		border-radius: 0.5rem;
		font-weight: 500;
		cursor: pointer;
		border: none;
		transition: all 0.2s;
	}

	.btn-primary {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		transform: none;
	}

	.btn-primary:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-secondary:hover {
		background: #e5e7eb;
	}
</style>
