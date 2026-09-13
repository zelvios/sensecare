<script lang="ts">
  import { page } from '$app/state';
  import { resolve } from '$app/paths';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';

  const status = $derived(page.status);

  const title = $derived.by(() => {
    if (status === 404) return m.error_404_title();
    if (status === 403) return m.error_403_title();
    if (status === 401) return m.error_401_title();
    return m.error_500_title();
  });

  const text = $derived.by(() => {
    if (status === 404) return m.error_404_text();
    if (status === 403) return m.error_403_text();
    if (status === 401) return m.error_401_text();
    return m.error_500_text();
  });

  const home = $derived(
    page.data.user?.role === 'client'
      ? resolve('/me')
      : page.data.user
        ? resolve('/staff')
        : resolve('/login')
  );
</script>

<svelte:head>
  <title>{status} - SenseCare</title>
</svelte:head>

<div
  class="mx-auto flex max-w-md flex-1 flex-col items-center justify-center px-4 py-16 text-center"
>
  <p class="text-7xl font-semibold tracking-tight text-surface-2 num">{status}</p>
  <h1 class="mt-4 text-2xl font-semibold">{title}</h1>
  <p class="mt-2 text-subtext">{text}</p>
  {#if status >= 500 && page.error?.message}
    <p class="mt-4 rounded-lg bg-surface-0/60 px-3 py-2 text-xs text-subtext num">
      {page.error.message}
    </p>
  {/if}
  <div class="mt-8 flex gap-3">
    <Button onclick={() => history.back()} variant="secondary">{m.go_back()}</Button>
    <Button onclick={() => (location.href = home)}>{m.go_home()}</Button>
  </div>
</div>
