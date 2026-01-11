<script lang="ts">
	export let currentStep: number;
	export let onSelect: (mode: 'local' | 'remote') => void;

	const modes = [
		{
			id: 'local',
			title: '本地模式',
			description: '在本地下载并运行 OpenWebUI',
			icon: '🖥️',
			features: [
				'完全隐私 - 所有数据保留在您的设备上',
				'初始下载后无需互联网连接',
				'完整访问所有功能包括本地模型',
				'需要约 500MB 下载'
			]
		},
		{
			id: 'remote',
			title: '远程模式',
			description: '连接到现有的 OpenWebUI 服务器',
			icon: '☁️',
			features: [
				'无需下载',
				'连接到您自己的服务器或公共实例',
				'跨设备共享对话',
				'需要网络连接'
			]
		}
	];
</script>

{#if currentStep === 2}
	<div class="mode-selection">
		<h1>欢迎使用 OpenWebUI</h1>
		<p>选择您想如何使用 OpenWebUI</p>

		<div class="modes">
			{#each modes as mode}
				<button class="mode-card" on:click={() => onSelect(mode.id)}>
					<div class="icon">{mode.icon}</div>
					<h2>{mode.title}</h2>
					<p>{mode.description}</p>
					<ul>
						{#each mode.features as feature}
							<li>{feature}</li>
						{/each}
					</ul>
				</button>
			{/each}
		</div>
	</div>
{/if}

<style>
	.mode-selection {
		text-align: center;
	}

	.mode-selection h1 {
		font-size: 2rem;
		font-weight: 700;
		color: white;
		margin-bottom: 0.5rem;
	}

	.mode-selection p {
		color: rgba(255, 255, 255, 0.8);
		font-size: 1.1rem;
		margin-bottom: 2rem;
	}

	.modes {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
	}

	.mode-card {
		background: white;
		border-radius: 1rem;
		padding: 2rem;
		cursor: pointer;
		transition: all 0.2s;
		border: none;
		text-align: left;
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.mode-card:hover {
		transform: translateY(-4px);
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
	}

	.icon {
		font-size: 3rem;
		margin-bottom: 1rem;
	}

	.mode-card h2 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #1f2937;
		margin-bottom: 0.5rem;
	}

	.mode-card p {
		color: #6b7280;
		font-size: 0.9rem;
		margin-bottom: 1rem;
	}

	.mode-card ul {
		list-style: none;
		padding: 0;
		margin: 0;
		flex: 1;
	}

	.mode-card li {
		color: #4b5563;
		font-size: 0.85rem;
		padding-left: 1.5rem;
		position: relative;
		margin-bottom: 0.5rem;
	}

	.mode-card li::before {
		content: '✓';
		position: absolute;
		left: 0;
		color: #10b981;
		font-weight: bold;
	}
</style>
