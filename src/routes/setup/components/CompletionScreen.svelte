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

			<h1>Setup Complete!</h1>

			{#if mode === 'local'}
				<p class="description">
					OpenWebUI has been successfully installed on your computer. You can now start using the local AI
					assistant with all data stored on your device.
				</p>

				<div class="info-box">
					<h3>🎉 Local Mode Enabled</h3>
					<ul>
						<li>✓ Backend service started</li>
						<li>✓ Data directory configured</li>
						<li>✓ All features available</li>
					</ul>
				</div>
			{:else}
				<p class="description">
					You have selected remote mode. Next, you need to configure the OpenWebUI server address you want to connect to.
				</p>

				<div class="info-box">
					<h3>Remote Mode Enabled</h3>
					<ul>
						<li>• Connect to existing server</li>
						<li>• Sync conversations across devices</li>
						<li>• Use server-side models and configuration</li>
					</ul>
				</div>
			{/if}

			<div class="actions">
				<button class="btn-primary" on:click={handleComplete}>
					{mode === 'local' ? 'Get Started' : 'Go to Settings'}
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
