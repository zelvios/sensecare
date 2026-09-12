<script lang="ts">
  import { Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import type { FloorFilter } from '$lib/utils/rooms';

  let {
    query = $bindable(''),
    floor = $bindable<FloorFilter>('all'),
    floors,
    class: extra = ''
  }: { query?: string; floor?: FloorFilter; floors: number[]; class?: string } = $props();
</script>

<div class="flex items-center gap-2 {extra}">
  <label class="relative min-w-0 flex-1 sm:w-72 sm:flex-none">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_rooms()}</span>
    <input
      type="search"
      bind:value={query}
      placeholder={m.search_rooms()}
      class="w-full rounded-md border bg-canvas py-1 pr-7 pl-8 text-sm placeholder:text-overlay focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    />
    {#if query}
      <button
        type="button"
        onclick={() => (query = '')}
        aria-label={m.clear()}
        class="absolute top-1/2 right-1 -translate-y-1/2 rounded p-1 text-overlay hover:text-text"
      >
        <X class="size-3.5" aria-hidden="true" />
      </button>
    {/if}
  </label>

  <label class="shrink-0">
    <span class="sr-only">{m.floor()}</span>
    <select
      bind:value={floor}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="all">{m.all_floors()}</option>
      {#each floors as f (f)}
        <option value={f}>{m.floor()} {f}</option>
      {/each}
    </select>
  </label>
</div>
