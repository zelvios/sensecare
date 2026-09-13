<script lang="ts">
  import { Search } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import type { User } from '$lib/api/types.ts';

  let {
    clients,
    occupiedIn,
    name = 'user_id'
  }: { clients: User[]; occupiedIn: Record<string, string>; name?: string } = $props();

  let query = $state('');
  let chosen = $state('');

  const matches = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const list = q
      ? clients.filter(
          (c) => c.username.toLowerCase().includes(q) || c.display_name.toLowerCase().includes(q)
        )
      : clients;
    return [...list].sort((x, y) => Number(x.id in occupiedIn) - Number(y.id in occupiedIn));
  });
</script>

<div>
  <label class="relative block">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_users()}</span>
    <input
      type="search"
      bind:value={query}
      placeholder={m.search_users()}
      class="w-full rounded-md border bg-canvas py-1.5 pr-3 pl-8 text-sm placeholder:text-overlay focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    />
  </label>

  <input type="hidden" {name} value={chosen} />

  {#if matches.length === 0}
    <p class="mt-2 text-sm text-subtext">{m.search_no_match_users()}</p>
  {:else}
    <ul class="mt-2 max-h-48 divide-y overflow-y-auto rounded-md border bg-canvas" role="listbox">
      {#each matches as c (c.id)}
        {@const room = occupiedIn[c.id]}
        <li>
          <button
            type="button"
            role="option"
            aria-selected={chosen === c.id}
            disabled={room !== undefined}
            onclick={() => (chosen = c.id)}
            class="flex w-full items-center justify-between gap-3 px-3 py-2 text-left text-sm
              {chosen === c.id ? 'bg-accent/10 text-accent' : ''}
              {room !== undefined ? 'cursor-not-allowed text-subtext' : 'hover:bg-surface-0/40'}"
          >
            <span class="min-w-0 truncate">
              {c.display_name}
              <span class="ml-1 text-xs text-subtext">{c.username}</span>
            </span>
            {#if room !== undefined}
              <span class="shrink-0 text-xs num">{m.checked_in_elsewhere()} {room}</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
