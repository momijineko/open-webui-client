<script lang="ts">
	export let currentStep: number;
	export let selectedMode: 'local' | 'remote' | null = null;
	export let onConfirm: (config: any) => void;
	export let onBack: () => void;

	// Component mirror configuration
	let componentMirrorSource = 'auto';
	let customComponentMirrors = '';

	// Python PyPI mirror configuration
	let pypiMirrorSource = 'auto';
	let customPyPIUrl = '';

	// Proxy configuration
	let enableProxy = false;
	let proxyUrl = '';

	// Hugging Face mirror configuration
	let hfMirrorSource = 'auto';
	let customHFUrl = '';

	// Offline mode
	let offlineMode = false;

	const predefinedComponentMirrors = [
		{ id: 'auto', name: 'Auto (Recommended)', url: '' },
		{ id: 'aliyun', name: 'Aliyun CDN (China)', url: 'https://openwebui.oss-cn-hangzhou.aliyuncs.com' },
		{ id: 'ghproxy', name: 'GHProxy (China Acceleration)', url: 'https://ghproxy.com/https://github.com' },
		{ id: 'github', name: 'GitHub Release', url: 'https://github.com/open-webui/open-webui/releases' },
		{ id: 'custom', name: 'Custom Mirror Source', url: '' }
	];

	const predefinedPyPIMirrors = [
		{ id: 'auto', name: 'Auto (Recommended)', url: '' },
		{ id: 'aliyun', name: 'Aliyun PyPI (China)', url: 'https://mirrors.aliyun.com/pypi/simple/' },
		{ id: 'tsinghua', name: 'Tsinghua PyPI (China)', url: 'https://pypi.tuna.tsinghua.edu.cn/simple/' },
		{ id: 'official', name: 'PyPI Official', url: 'https://pypi.org/simple' },
		{ id: 'custom', name: 'Custom PyPI Source', url: '' }
	];

	const predefinedHFMirrors = [
		{ id: 'auto', name: 'Auto (Recommended)', url: '' },
		{ id: 'hf-mirror', name: 'HF-Mirror (China Acceleration)', url: 'https://hf-mirror.com' },
		{ id: 'official', name: 'Hugging Face Official', url: 'https://huggingface.co' },
		{ id: 'custom', name: 'Custom Mirror Source', url: '' }
	];

	const handleConfirm = () => {
		const componentMirrors = customComponentMirrors.split('\n').filter((m) => m.trim());

		// Get selected mirror URL
		const componentMirrorUrl = componentMirrorSource === 'custom'
			? null
			: predefinedComponentMirrors.find(m => m.id === componentMirrorSource)?.url || null;

		const pypiMirrorUrl = pypiMirrorSource === 'custom'
			? customPyPIUrl || null
			: predefinedPyPIMirrors.find(m => m.id === pypiMirrorSource)?.url || null;

		const hfMirrorUrl = hfMirrorSource === 'custom'
			? customHFUrl || null
			: predefinedHFMirrors.find(m => m.id === hfMirrorSource)?.url || null;

		onConfirm({
			componentMirror: {
				source: componentMirrorSource,
				customMirrors: componentMirrors,
				url: componentMirrorUrl
			},
			pypiMirror: {
				source: pypiMirrorSource,
				url: pypiMirrorUrl
			},
			hfMirror: {
				source: hfMirrorSource,
				url: hfMirrorUrl
			},
			proxy: {
				enabled: enableProxy,
				url: proxyUrl
			},
			offlineMode: offlineMode
		});
	};
</script>

{#if currentStep === 4 && selectedMode === 'local'}
	<div class="mirror-config">
		<h1>Configure Download Sources</h1>
		<p>Select mirror sources for downloading components and Python packages</p>

		<div class="config-form">
			<!-- Component mirror configuration -->
			<div class="config-section">
				<div class="section-header">
					<h2>Component Download Source</h2>
					<span class="section-description">For downloading backend, models and other large files</span>
				</div>

				<div class="form-group">
					<label>Mirror Source</label>
					<select bind:value={componentMirrorSource}>
						{#each predefinedComponentMirrors as mirror}
							<option value={mirror.id}>{mirror.name}</option>
						{/each}
					</select>
				</div>

				{#if componentMirrorSource === 'custom'}
					<div class="form-group">
						<label>Custom Mirror Source URL (one per line)</label>
						<textarea
							bind:value={customComponentMirrors}
							placeholder="https://mirror1.example.com&#10;https://mirror2.example.com"
							rows="3"
						></textarea>
					</div>
				{/if}
			</div>

			<!-- Python PyPI mirror configuration -->
			<div class="config-section">
				<div class="section-header">
					<h2>Python Package Download Source</h2>
					<span class="section-description">For downloading Python dependency packages via pip</span>
				</div>

				<div class="form-group">
					<label>PyPI Mirror Source</label>
					<select bind:value={pypiMirrorSource}>
						{#each predefinedPyPIMirrors as mirror}
							<option value={mirror.id}>{mirror.name}</option>
						{/each}
					</select>
				</div>

				{#if pypiMirrorSource === 'custom'}
					<div class="form-group">
						<label>Custom PyPI Source URL</label>
						<input
							type="text"
							placeholder="https://pypi.example.com/simple/"
							bind:value={customPyPIUrl}
							class="text-input"
						/>
						<span class="input-hint">Please enter full PyPI simple index URL</span>
					</div>
				{/if}
			</div>

			<!-- Hugging Face mirror configuration -->
			<div class="config-section">
				<div class="section-header">
					<h2>Hugging Face Mirror</h2>
					<span class="section-description">For downloading AI models and datasets</span>
				</div>

				<div class="form-group">
					<label>Mirror Source</label>
					<select bind:value={hfMirrorSource}>
						{#each predefinedHFMirrors as mirror}
							<option value={mirror.id}>{mirror.name}</option>
						{/each}
					</select>
				</div>

				{#if hfMirrorSource === 'custom'}
					<div class="form-group">
						<label>Custom Mirror Source URL</label>
						<input
							type="text"
							placeholder="https://hf-mirror.com"
							bind:value={customHFUrl}
							class="text-input"
						/>
						<span class="input-hint">Full URL of the Hugging Face Mirror site</span>
					</div>
				{/if}
			</div>

			<!-- Offline mode configuration -->
			<div class="config-section">
				<div class="section-header">
					<h2>Offline mode</h2>
					<span class="section-description">Disable all online features, suitable for completely offline environments</span>
				</div>

				<div class="form-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={offlineMode} />
						<span>Enable Offline Mode</span>
					</label>
					<span class="input-hint">
						When enabled, version checking, RAG and other features requiring internet will be disabled
					</span>
				</div>
			</div>

			<!-- Proxy configuration -->
			<div class="config-section">
				<div class="section-header">
					<h2>Proxy Settings</h2>
					<span class="section-description">Configure proxy if your network requires it</span>
				</div>

				<div class="form-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={enableProxy} />
						<span>Enable Proxy</span>
					</label>
					{#if enableProxy}
						<input
							type="text"
							placeholder="http://proxy.example.com:8080"
							bind:value={proxyUrl}
							class="text-input"
						/>
					{/if}
				</div>
			</div>
		</div>

		<div class="actions">
			<button class="btn-secondary" on:click={onBack}>Previous</button>
			<button class="btn-primary" on:click={handleConfirm}>Start Download</button>
		</div>
	</div>
{/if}

<style>
	.mirror-config {
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
	.mirror-config::-webkit-scrollbar {
		width: 8px;
	}

	.mirror-config::-webkit-scrollbar-track {
		background: #f1f1f1;
		border-radius: 4px;
	}

	.mirror-config::-webkit-scrollbar-thumb {
		background: #888;
		border-radius: 4px;
	}

	.mirror-config::-webkit-scrollbar-thumb:hover {
		background: #555;
	}

	.mirror-config h1 {
		font-size: 1.75rem;
		font-weight: 700;
		color: #1f2937;
		margin-bottom: 0.5rem;
		text-align: center;
		flex-shrink: 0;
	}

	.mirror-config p {
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
	}

	.form-group label {
		font-weight: 500;
		color: #374151;
		font-size: 0.9rem;
	}

	.form-group select,
	.form-group textarea,
	.text-input {
		padding: 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.5rem;
		font-size: 0.9rem;
		font-family: inherit;
	}

	.form-group select:focus,
	.form-group textarea:focus,
	.text-input:focus {
		outline: none;
		border-color: #6366f1;
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
	}

	.input-hint {
		font-size: 0.75rem;
		color: #9ca3af;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
	}

	.checkbox-label input[type='checkbox'] {
		width: 1.1rem;
		height: 1.1rem;
		cursor: pointer;
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

	.btn-primary:hover {
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
