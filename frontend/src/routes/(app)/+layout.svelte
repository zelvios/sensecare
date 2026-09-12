<script lang="ts">
  import Header from '$lib/components/Header.svelte';
  let { data, children } = $props();
</script>

<Header user={data.user} />

<main class="mx-auto flex w-full max-w-6xl flex-1 flex-col px-4 py-6">
  {@render children()}
</main>
