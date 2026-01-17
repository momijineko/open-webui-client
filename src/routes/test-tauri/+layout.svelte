<!-- 这个布局用于绕过根布局的后端检查 -->
<svelte:head>
	<style>
		/* Override parent h-screen for test page */
		:global(body) {
			height: auto !important;
			min-height: 100vh;
		}

		/* Allow scrolling in test page */
		:global(.test-page-wrapper) > div {
			height: auto !important;
			min-height: 100vh;
		}
	</style>
</svelte:head>

<div class="test-page-wrapper">
	<slot />
</div>
