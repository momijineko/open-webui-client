<script lang="ts">
	export let currentStep: number;
	export let onSelect: (mode: 'local' | 'remote') => void;

	const modes = [
		{
			id: 'local',
			title: 'Local Mode',
			description: 'Download and run OpenWebUI locally',
			icon: '🖥️',
			features: [
				'Complete privacy - All data stays on your device',
				'No internet connection required after initial download',
				'Full access to all features including local models',
				'Requires ~500MB download'
			]
		},
		{
			id: 'remote',
			title: 'Remote Mode',
			description: 'Connect to existing OpenWebUI server',
			icon: '☁️',
			features: [
				'No download required',
				'Connect to your own server or public instance',
				'Share conversations across devices',
				'Requires network connection'
			]
		}
	];
</script>

{#if currentStep === 2}
	<div class="mode-selection">
		<h1>Welcome to OpenWebUI</h1>
		<p>Choose how you want to use OpenWebUI</p>

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
