<script lang="ts">
  import { Search } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import type { Room } from '$lib/api/types';

  let { rooms, name = 'room_id' }: { rooms: Room[]; name?: string } = $props();

  let query = $state('');
  let chosen = $state('');

  const label = (r: Room) => {
    const floor = r.floor == null ? '' : `${m.floor()} ${r.floor}, `;
    return `${floor}${r.room_number}${r.name ? ` - ${r.name}` : ''}`;
  };

  const matches = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return rooms;
    return rooms.filter(
      (r) =>
        r.id.toLowerCase().startsWith(q) ||
        r.room_number.toLowerCase().includes(q) ||
        (r.name ?? '').toLowerCase().includes(q) ||
        (r.floor != null && String(r.floor) === q)
    );
  });
</script>

<div>
  <label class="relative block">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_room_picker()}</span>
    <input
      type="search"
      bind:value={query}
      placeholder={m.search_room_picker()}
      class="w-full rounded-md border bg-canvas py-1.5 pr-3 pl-8 text-sm placeholder:text-overlay focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    />
  </label>

  <input type="hidden" {name} value={chosen} />

  {#if matches.length === 0}
    <p class="mt-2 text-sm text-subtext">{m.search_no_match()}</p>
  {:else}
    <ul class="mt-2 max-h-48 divide-y overflow-y-auto rounded-md border bg-canvas" role="listbox">
      {#each matches as r (r.id)}
        <li>
          <button
            type="button"
            role="option"
            aria-selected={chosen === r.id}
            onclick={() => (chosen = r.id)}
            class="flex w-full items-center justify-between px-3 py-2 text-left text-sm hover:bg-surface-0/40 {chosen ===
            r.id
              ? 'bg-accent/10 text-accent'
              : ''}"
          >
            <span class="num">{label(r)}</span>
            <span class="text-xs text-overlay num">{r.id.slice(0, 8)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
