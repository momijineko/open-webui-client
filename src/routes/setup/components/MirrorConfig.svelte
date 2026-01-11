<script lang="ts">
	export let currentStep: number;
	export let selectedMode: 'local' | 'remote' | null = null;
	export let onConfirm: (config: any) => void;
	export let onBack: () => void;

	// 组件镜像源配置
	let componentMirrorSource = 'auto';
	let customComponentMirrors = '';

	// Python PyPI 镜像源配置
	let pypiMirrorSource = 'auto';
	let customPyPIUrl = '';

	// 代理配置
	let enableProxy = false;
	let proxyUrl = '';

	// Hugging Face 镜像配置
	let hfMirrorSource = 'auto';
	let customHFUrl = '';

	// 离线模式
	let offlineMode = false;

	const predefinedComponentMirrors = [
		{ id: 'auto', name: '自动选择（推荐）', url: '' },
		{ id: 'aliyun', name: '阿里云 CDN (中国)', url: 'https://openwebui.oss-cn-hangzhou.aliyuncs.com' },
		{ id: 'ghproxy', name: 'GHProxy (中国加速)', url: 'https://ghproxy.com/https://github.com' },
		{ id: 'github', name: 'GitHub Release', url: 'https://github.com/open-webui/open-webui/releases' },
		{ id: 'custom', name: '自定义镜像源', url: '' }
	];

	const predefinedPyPIMirrors = [
		{ id: 'auto', name: '自动选择（推荐）', url: '' },
		{ id: 'aliyun', name: '阿里云 PyPI (中国)', url: 'https://mirrors.aliyun.com/pypi/simple/' },
		{ id: 'tsinghua', name: '清华大学 PyPI (中国)', url: 'https://pypi.tuna.tsinghua.edu.cn/simple/' },
		{ id: 'official', name: 'PyPI 官方源', url: 'https://pypi.org/simple' },
		{ id: 'custom', name: '自定义 PyPI 源', url: '' }
	];

	const predefinedHFMirrors = [
		{ id: 'auto', name: '自动选择（推荐）', url: '' },
		{ id: 'hf-mirror', name: 'HF-Mirror 镜像 (中国加速)', url: 'https://hf-mirror.com' },
		{ id: 'official', name: 'Hugging Face 官方', url: 'https://huggingface.co' },
		{ id: 'custom', name: '自定义镜像源', url: '' }
	];

	const handleConfirm = () => {
		const componentMirrors = customComponentMirrors.split('\n').filter((m) => m.trim());

		// 获取选中的镜像 URL
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
		<h1>配置下载源</h1>
		<p>选择用于下载组件和 Python 包的镜像源</p>

		<div class="config-form">
			<!-- 组件镜像源配置 -->
			<div class="config-section">
				<div class="section-header">
					<h2>组件下载源</h2>
					<span class="section-description">用于下载后端程序、模型等大型文件</span>
				</div>

				<div class="form-group">
					<label>镜像源</label>
					<select bind:value={componentMirrorSource}>
						{#each predefinedComponentMirrors as mirror}
							<option value={mirror.id}>{mirror.name}</option>
						{/each}
					</select>
				</div>

				{#if componentMirrorSource === 'custom'}
					<div class="form-group">
						<label>自定义镜像源 URL（每行一个）</label>
						<textarea
							bind:value={customComponentMirrors}
							placeholder="https://mirror1.example.com&#10;https://mirror2.example.com"
							rows="3"
						></textarea>
					</div>
				{/if}
			</div>

			<!-- Python PyPI 镜像源配置 -->
			<div class="config-section">
				<div class="section-header">
					<h2>Python 包下载源</h2>
					<span class="section-description">用于通过 pip 下载 Python 依赖包</span>
				</div>

				<div class="form-group">
					<label>PyPI 镜像源</label>
					<select bind:value={pypiMirrorSource}>
						{#each predefinedPyPIMirrors as mirror}
							<option value={mirror.id}>{mirror.name}</option>
						{/each}
					</select>
				</div>

				{#if pypiMirrorSource === 'custom'}
					<div class="form-group">
						<label>自定义 PyPI 源 URL</label>
						<input
							type="text"
							placeholder="https://pypi.example.com/simple/"
							bind:value={customPyPIUrl}
							class="text-input"
						/>
						<span class="input-hint">请输入完整的 PyPI simple index URL</span>
					</div>
				{/if}
			</div>

			<!-- Hugging Face 镜像配置 -->
			<div class="config-section">
				<div class="section-header">
					<h2>Hugging Face 镜像</h2>
					<span class="section-description">用于下载 AI 模型和数据集</span>
				</div>

				<div class="form-group">
					<label>镜像源</label>
					<select bind:value={hfMirrorSource}>
						{#each predefinedHFMirrors as mirror}
							<option value={mirror.id}>{mirror.name}</option>
						{/each}
					</select>
				</div>

				{#if hfMirrorSource === 'custom'}
					<div class="form-group">
						<label>自定义镜像源 URL</label>
						<input
							type="text"
							placeholder="https://hf-mirror.com"
							bind:value={customHFUrl}
							class="text-input"
						/>
						<span class="input-hint">Hugging Face 镜像站点的完整 URL</span>
					</div>
				{/if}
			</div>

			<!-- 离线模式配置 -->
			<div class="config-section">
				<div class="section-header">
					<h2>离线模式</h2>
					<span class="section-description">禁用所有在线功能，适合完全离线环境</span>
				</div>

				<div class="form-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={offlineMode} />
						<span>启用离线模式</span>
					</label>
					<span class="input-hint">
						启用后将禁用版本检查、RAG 功能等需要联网的功能
					</span>
				</div>
			</div>

			<!-- 代理配置 -->
			<div class="config-section">
				<div class="section-header">
					<h2>代理设置</h2>
					<span class="section-description">如果网络环境需要代理，请在此配置</span>
				</div>

				<div class="form-group">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={enableProxy} />
						<span>启用代理</span>
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
			<button class="btn-secondary" on:click={onBack}>上一步</button>
			<button class="btn-primary" on:click={handleConfirm}>开始下载</button>
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

	/* 自定义滚动条样式 */
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
