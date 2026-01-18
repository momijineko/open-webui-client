<script lang="ts">
	export let currentStep: number;
	export let selectedMode: 'local' | 'remote' | null = null;
	export let onConfirm: (config: { url: string; username?: string; password?: string }) => void;
	export let onBack: () => void;

	let serverUrl = '';
	let username = '';
	let password = '';
	let needsAuth = false; // Whether authentication is needed
	let isValidUrl = true;
	let urlError = '';
	let authError = ''; // Authentication error
	let isTesting = false;
	let hasTested = false; // Whether test has been executed
	let testResult: { success: boolean; message: string } | null = null;

	const exampleServers = [
		{ name: 'Example: http://localhost:8080', url: 'http://localhost:8080' },
		{ name: 'Example: http://192.168.1.100:3000', url: 'http://192.168.1.100:3000' }
	];

	const validateUrl = (url: string): boolean => {
		if (!url.trim()) {
			urlError = 'Please enter server address';
			return false;
		}

		try {
			const parsed = new URL(url);
			if (!['http:', 'https:'].includes(parsed.protocol)) {
				urlError = 'Please use http:// or https://';
				return false;
			}
			urlError = '';
			return true;
		} catch {
			urlError = 'Please enter a valid URL';
			return false;
		}
	};

	const validateAuth = (): boolean => {
		if (!needsAuth) {
			authError = '';
			return true;
		}

		if (!username.trim()) {
			authError = 'Please enter username';
			return false;
		}

		if (!password) {
			authError = 'Please enter password';
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

		// If authentication is needed, verify credentials first
		if (needsAuth && !validateAuth()) {
			return;
		}

		isTesting = true;
		testResult = null;

		try {
			// Test connection - try to access server health check endpoint
			const testUrl = serverUrl.replace(/\/$/, '') + '/api/config';

			// Build headers
			const headers: Record<string, string> = {
				'Content-Type': 'application/json'
			};

			// Add Basic Auth if authentication is required and credentials are provided
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
					message: 'Connection successful! Server is accessible'
				};
			} else if (response.status === 401) {
				testResult = {
					success: false,
					message: 'Authentication failed: Incorrect username or password'
				};
			} else {
				testResult = {
					success: false,
					message: `Server returned error: ${response.status} ${response.statusText}`
				};
			}
		} catch (error) {
			testResult = {
				success: false,
				message: `Connection failed: ${(error as Error).message}`
			};
		} finally {
			isTesting = false;
			hasTested = true; // Mark that test has been executed
		}
	};

	const handleConfirm = () => {
		if (!validateUrl(serverUrl)) {
			isValidUrl = false;
			return;
		}

		// If authentication is needed, verify credentials
		if (needsAuth && !validateAuth()) {
			return;
		}

		// Check if test has been executed and passed
		if (!hasTested) {
			alert('Please test connection first');
			return;
		}

		if (!testResult?.success) {
			alert('Connection test failed. Please check Server Address and Authentication Information');
			return;
		}

		// Remove trailing slash
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
		<h1>Configure Remote Server</h1>
		<p>Enter the OpenWebUI server address you want to connect to</p>

		<div class="config-form">
			<!-- Server Address Configuration -->
			<div class="config-section">
				<div class="section-header">
					<h2>Server Address</h2>
					<span class="section-description">
						Enter your self-deployed OpenWebUI Server Address
					</span>
				</div>

				<div class="form-group">
					<label>Server URL</label>
					<input
						type="text"
						placeholder="https://your-server.com"
						bind:value={serverUrl}
						on:input={handleUrlInput}
						class="text-input"
						class:invalid={!isValidUrl}
					/>
					<span class="input-hint">
						e.g.: http://localhost:8080 or https://your-server.com
					</span>
					{#if !isValidUrl}
						<span class="error-message">{urlError}</span>
					{/if}
				</div>

				<div class="form-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={needsAuth} />
						<span>Server requires authentication</span>
					</label>
				</div>

				<div class="form-group">
					<label>Quick Fill Examples</label>
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
						Testing...
					{:else}
						🔍 Test Connection
					{/if}
				</button>

				{#if testResult}
					<div class="test-result" class:success={testResult.success} class:error={!testResult.success}>
						{testResult.success ? '✓' : '✗'} {testResult.message}
					</div>
				{/if}
			</div>

			<!-- Authentication Configuration -->
			{#if needsAuth}
				<div class="config-section">
					<div class="section-header">
						<h2>Authentication Information</h2>
						<span class="section-description">
							Enter Basic Auth username and password
						</span>
					</div>

					<div class="form-group">
						<label>Username</label>
						<input
							type="text"
							placeholder="admin"
							bind:value={username}
							on:input={handleAuthInput}
							class="text-input"
						/>
					</div>

					<div class="form-group">
						<label>Password</label>
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

			<!-- Tips Information -->
			<div class="info-box">
				<h3>💡 Tips</h3>
				<ul>
					<li>Ensure your server is accessible from current device</li>
					<li>If using self-signed certificate, you may need to trust it in browser first</li>
					<li>Basic Auth credentials will be securely stored locally</li>
					<li>Can be modified in settings page after configuration</li>
				</ul>
			</div>
		</div>

		<div class="actions">
			<button class="btn-secondary" on:click={onBack}>Previous</button>
			<button
				class="btn-primary"
				on:click={handleConfirm}
				disabled={!isValidUrl || !serverUrl || !hasTested || !testResult?.success}
			>
				Next
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

	/* Custom scrollbar style */
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
