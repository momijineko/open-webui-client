<script lang="ts">
	export let currentStep: number;
	export let mode: 'local' | 'remote' | null;
	export let onComplete: () => void;

	const handleComplete = () => {
		onComplete();
	};
</script>

{#if currentStep === 7}
	<div class="completion-screen">
		<div class="completion-content">
			<div class="success-icon">✓</div>

			<h1>设置完成！</h1>

			{#if mode === 'local'}
				<p class="description">
					OpenWebUI 已成功安装到您的计算机。您现在可以开始使用本地 AI
					助手，所有数据都保存在您的设备上。
				</p>

				<div class="info-box">
					<h3>🎉 本地模式已启用</h3>
					<ul>
						<li>✓ 后端服务已启动</li>
						<li>✓ 数据目录已配置</li>
						<li>✓ 所有功能可用</li>
					</ul>
				</div>
			{:else}
				<p class="description">
					您已选择远程模式。接下来需要配置您要连接的 OpenWebUI 服务器地址。
				</p>

				<div class="info-box">
					<h3>☁️ 远程模式已启用</h3>
					<ul>
						<li>• 连接到现有服务器</li>
						<li>• 跨设备同步对话</li>
						<li>• 使用服务器端的模型和配置</li>
					</ul>
				</div>
			{/if}

			<div class="actions">
				<button class="btn-primary" on:click={handleComplete}>
					{mode === 'local' ? '开始使用' : '前往设置'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.completion-screen {
		background: white;
		border-radius: 1rem;
		padding: 3rem 2rem;
		box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
		text-align: center;
	}

	.completion-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1.5rem;
	}

	.success-icon {
		width: 5rem;
		height: 5rem;
		background: linear-gradient(135deg, #10b981 0%, #059669 100%);
		color: white;
		font-size: 2.5rem;
		font-weight: bold;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 10px 30px rgba(16, 185, 129, 0.3);
	}

	.completion-content h1 {
		font-size: 2rem;
		font-weight: 700;
		color: #1f2937;
		margin: 0;
	}

	.description {
		color: #6b7280;
		font-size: 1rem;
		max-width: 400px;
		line-height: 1.6;
		margin: 0;
	}

	.info-box {
		background: #f9fafb;
		border-radius: 0.75rem;
		padding: 1.5rem;
		width: 100%;
		max-width: 400px;
		text-align: left;
	}

	.info-box h3 {
		font-size: 1.1rem;
		font-weight: 600;
		color: #1f2937;
		margin: 0 0 1rem 0;
	}

	.info-box ul {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.info-box li {
		color: #4b5563;
		font-size: 0.9rem;
		padding: 0.5rem 0;
		border-bottom: 1px solid #e5e7eb;
	}

	.info-box li:last-child {
		border-bottom: none;
	}

	.actions {
		margin-top: 1rem;
	}

	.btn-primary {
		padding: 1rem 2.5rem;
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
		border: none;
		border-radius: 0.5rem;
		font-size: 1rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn-primary:hover {
		transform: translateY(-2px);
		box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
	}
</style>
