<script lang="ts">
  import Header from '$lib/components/Header.svelte';
  let { data, children } = $props();
</script>

<Header user={data.user} />

<main class="mx-auto max-w-6xl px-4 py-6">
  {@render children()}
</main>
