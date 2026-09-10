<script lang="ts">
  import { invalidateAll } from '$app/navigation';
  import { RefreshCw, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime } from '$lib/utils/format';
  import AlertsStrip from './AlertsStrip.svelte';
  import RoomCard from './RoomCard.svelte';

  let { data } = $props();
  let refreshing = $state(false);
  let query = $state('');
  let floorFilter = $state<'all' | number>('all');

  // Distinct floors present in the data, lowest first. Rooms without a floor are not listed.
  const floors = $derived(
    [...new Set(data.rooms.map((r) => r.room.floor).filter((f): f is number => f != null))].sort(
      (a, b) => a - b
    )
  );

  const visible = $derived.by(() => {
    const onFloor =
      floorFilter === 'all'
        ? data.rooms
        : data.rooms.filter(({ room }) => room.floor === floorFilter);

    const q = query.trim().toLowerCase();
    if (!q) return onFloor;

    const floorWord = m.floor().toLowerCase();
    const byFloor = q.match(new RegExp(`^${floorWord}\\s*(\\d+)$`));
    if (byFloor) {
      const wanted = Number(byFloor[1]);
      return onFloor.filter(({ room }) => room.floor === wanted);
    }

    return onFloor.filter(({ room }) => {
      const floor = room.floor === null || room.floor === undefined ? '' : String(room.floor);
      return (
        floor === q ||
        room.room_number.toLowerCase().includes(q) ||
        (room.name ?? '').toLowerCase().includes(q)
      );
    });
  });

  const filtering = $derived(query.trim() !== '' || floorFilter !== 'all');

  async function refresh() {
    refreshing = true;
    try {
      await invalidateAll();
    } finally {
      refreshing = false;
    }
  }

  // The dashboard is the panel that must notice a button press. Refresh every 15 s while visible.
  $effect(() => {
    const id = setInterval(() => {
      if (document.visibilityState === 'visible') refresh();
    }, 15_000);
    return () => clearInterval(id);
  });
</script>

<svelte:head>
  <title>{m.title_overview()}</title>
</svelte:head>

<div class="flex min-h-0 flex-1 flex-col">
  <div class="flex flex-wrap items-center justify-between gap-4">
    <h1 class="text-3xl font-semibold tracking-tight">{m.nav_overview()}</h1>
    <div class="flex items-center gap-2 text-sm text-subtext">
      <span class="num">{m.updated_at({ time: fmtDateTime(data.loadedAt) })}</span>
      <button
        aria-label={m.refresh()}
        class="rounded-md p-1.5 transition hover:bg-surface-0/60 hover:text-text disabled:opacity-60"
        disabled={refreshing}
        onclick={refresh}
        type="button"
      >
        <RefreshCw aria-hidden="true" class="size-4 {refreshing ? 'animate-spin' : ''}" />
      </button>
    </div>
  </div>

  <div class="mt-6 shrink-0">
    <AlertsStrip rooms={data.rooms} />
  </div>

  <section aria-labelledby="rooms-heading" class="mt-8 flex min-h-0 flex-1 flex-col">
    <div class="flex shrink-0 flex-wrap items-center gap-3">
      <h2 class="text-lg font-semibold" id="rooms-heading">{m.rooms()}</h2>
      <span class="h-px flex-1 bg-surface-0 sm:hidden" aria-hidden="true"></span>
      <span class="text-sm text-subtext num sm:hidden">
        {#if filtering}{visible.length} /
        {/if}{data.rooms.length}
      </span>

      <div class="flex w-full items-center gap-2 sm:w-auto">
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
            bind:value={floorFilter}
            class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
          >
            <option value="all">{m.all_floors()}</option>
            {#each floors as f (f)}
              <option value={f}>{m.floor()} {f}</option>
            {/each}
          </select>
        </label>
      </div>

      <span class="hidden h-px flex-1 bg-surface-0 sm:block" aria-hidden="true"></span>
      <span class="hidden text-sm text-subtext num sm:inline">
        {#if filtering}{visible.length} /
        {/if}{data.rooms.length}
      </span>
    </div>

    {#if data.rooms.length === 0}
      <p class="mt-3 text-sm text-subtext">{m.rooms_none()}</p>
    {:else if visible.length === 0}
      <p class="mt-3 text-sm text-subtext">{m.search_no_match()}</p>
    {:else}
      <div class="mt-3 min-h-0 flex-1 overflow-y-auto p-1">
        <div class="grid grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-4">
          {#each visible as r (r.room.id)}
            <RoomCard room={r} />
          {/each}
        </div>
      </div>
    {/if}
  </section>
</div>
