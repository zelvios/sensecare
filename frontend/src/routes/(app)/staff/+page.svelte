<script lang="ts">
  import { invalidateAll } from '$app/navigation';
  import { RefreshCw } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTimeSec } from '$lib/utils/format';
  import RoomFilterBar from '$lib/components/rooms/RoomFilterBar.svelte';
  import { filterRooms, type FloorFilter, floorsOf } from '$lib/utils/rooms';
  import AlertsStrip from './AlertsStrip.svelte';
  import RoomCard from './RoomCard.svelte';

  let { data } = $props();
  let refreshing = $state(false);
  let query = $state('');
  let floor = $state<FloorFilter>('all');

  const floors = $derived(floorsOf(data.rooms.map((r) => r.room)));

  const visible = $derived(
    filterRooms(
      data.rooms.map((r) => ({ ...r.room, overview: r })),
      query,
      floor
    ).map((x) => x.overview)
  );

  const filtering = $derived(query.trim() !== '' || floor !== 'all');

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

<div class="flex h-[calc(100dvh-6.5rem)] min-h-0 flex-col">
  <div class="flex flex-wrap items-center justify-between gap-4">
    <h1 class="text-3xl font-semibold tracking-tight">{m.nav_overview()}</h1>
    <div class="flex items-center gap-2 text-sm text-subtext">
      <span class="num">{m.updated_at()} {fmtDateTimeSec(data.loadedAt)}</span>
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
      <span aria-hidden="true" class="h-px flex-1 bg-surface-0 sm:hidden"></span>
      <span class="text-sm text-subtext num sm:hidden">
        {#if filtering}{visible.length} /
        {/if}{data.rooms.length}
      </span>

      <RoomFilterBar bind:floor bind:query class="w-full sm:w-auto" {floors} />

      <span aria-hidden="true" class="hidden h-px flex-1 bg-surface-0 sm:block"></span>
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
