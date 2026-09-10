<script lang="ts">
  import '@fontsource-variable/noto-sans';
  import './layout.css';
  import favicon from '$lib/assets/favicon.svg';

  let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex h-dvh flex-col">
  {@render children()}
</div>
