<script lang="ts">
  import { enhance } from '$app/forms';
  import { resolve } from '$app/paths';
  import { ArrowLeft, ChevronLeft, ChevronRight, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { fmtDateTime, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import ThresholdForm from './ThresholdForm.svelte';
  import RoomPicker from './RoomPicker.svelte';
  import type { Room, Threshold } from '$lib/api/types';

  let { data } = $props();
  const a = actionState();

  const roomById = $derived(new Map(data.rooms.map((r) => [r.id, r])));
  const roomLabel = (r: Room | undefined) => {
    if (!r) return '';
    const floor = r.floor == null ? '' : `${m.floor()} ${r.floor}, `;
    return `${floor}${r.room_number}${r.name ? ` - ${r.name}` : ''}`;
  };

  type Row = Threshold & { room?: Room };
  const rows = $derived<Row[]>(
    data.overrides.map((t) => ({ ...t, room: t.room_id ? roomById.get(t.room_id) : undefined }))
  );
  const roomsWithoutOverride = $derived(
    data.rooms.filter((r) => r.is_active && !data.overrides.some((t) => t.room_id === r.id))
  );

  let creating = $state(false);
  let editing = $state<Row | null>(null);
  let editingGlobal = $state(false);
  let deleting = $state<Row | null>(null);
  const dialogOpen = $derived(creating || editing !== null || editingGlobal || deleting !== null);

  const pageSizes = [5, 10, 15, 20, 30];
  let query = $state('');
  let pageSize = $state(10);
  let page = $state(1);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return rows;
    return rows.filter((t) => {
      const r = t.room;
      return (
        (r?.room_number ?? '').toLowerCase().includes(q) ||
        (r?.name ?? '').toLowerCase().includes(q) ||
        (r?.floor != null && String(r.floor) === q) ||
        (t.room_id ?? '').startsWith(q)
      );
    });
  });

  type SortKey =
    'room' | 'temperature_min' | 'temperature_max' | 'humidity_min' | 'humidity_max' | 'updated_at';
  let sortKey = $state<SortKey>('room');
  let sortDir = $state<'asc' | 'desc'>('asc');

  function toggleSort(key: SortKey) {
    if (sortKey === key) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    else {
      sortKey = key;
      sortDir = key === 'updated_at' ? 'desc' : 'asc';
    }
    page = 1;
  }

  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
  const compare: Record<SortKey, (x: Row, y: Row) => number> = {
    room: (x, y) =>
      (x.room?.floor ?? Number.POSITIVE_INFINITY) - (y.room?.floor ?? Number.POSITIVE_INFINITY) ||
      collator.compare(x.room?.room_number ?? '', y.room?.room_number ?? ''),
    temperature_min: (x, y) => x.temperature_min - y.temperature_min,
    temperature_max: (x, y) => x.temperature_max - y.temperature_max,
    humidity_min: (x, y) => x.humidity_min - y.humidity_min,
    humidity_max: (x, y) => x.humidity_max - y.humidity_max,
    updated_at: (x, y) => x.updated_at.localeCompare(y.updated_at)
  };

  const sorted = $derived.by(() => {
    const list = [...filtered].sort(compare[sortKey]);
    return sortDir === 'asc' ? list : list.reverse();
  });

  const pageCount = $derived(Math.max(1, Math.ceil(sorted.length / pageSize)));
  $effect(() => {
    if (page > pageCount) page = pageCount;
  });
  const start = $derived((page - 1) * pageSize);
  const visible = $derived(sorted.slice(start, start + pageSize));

  const errorText = $derived.by(() => {
    if (!a.error) return null;
    if (a.error === 'bad_request' || a.error === 'constraint_violation')
      return m.threshold_invalid();
    if (a.error === 'not_found') return m.threshold_room_missing();
    return m.action_failed();
  });
</script>

{#snippet errorLine()}
  {#if errorText}
    <p class="mt-4 rounded-lg bg-danger/10 px-3 py-2 text-sm text-danger" role="alert">
      {errorText}
    </p>
  {/if}
{/snippet}

{#snippet sortHeader(key: SortKey, label: string)}
  {@const active = sortKey === key}
  <th
    class="px-4 py-2 font-medium"
    aria-sort={active ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'}
  >
    <button
      type="button"
      onclick={() => toggleSort(key)}
      class="inline-flex items-center gap-1 rounded hover:text-text {active ? 'text-text' : ''}"
    >
      {label}
      <span aria-hidden="true" class="text-xs {active ? '' : 'invisible'}">
        {sortDir === 'asc' ? '▲' : '▼'}
      </span>
    </button>
  </th>
{/snippet}

{#snippet limitCells(t: Threshold)}
  <td class="px-4 py-2 num">{fmtTemp(t.temperature_min)}</td>
  <td class="px-4 py-2 num">{fmtTemp(t.temperature_max)}</td>
  <td class="px-4 py-2 num">{fmtHumidity(t.humidity_min)}</td>
  <td class="px-4 py-2 num">{fmtHumidity(t.humidity_max)}</td>
  <td class="px-4 py-2 text-subtext num">{fmtDateTime(t.updated_at)}</td>
{/snippet}

<svelte:head>
  <title>{m.title_admin_thresholds()}</title>
</svelte:head>

<a
  class="inline-flex items-center gap-1 text-sm text-subtext hover:text-text"
  href={resolve('/admin')}
>
  <ArrowLeft aria-hidden="true" class="size-4" />
  {m.nav_admin()}
</a>

<div class="mt-3 flex flex-wrap items-center justify-between gap-4">
  <div class="flex items-baseline gap-3">
    <h1 class="text-3xl font-semibold tracking-tight">{m.admin_thresholds()}</h1>
    <span class="text-sm text-subtext num" title={m.thresholds_override_count()}>{rows.length}</span
    >
  </div>
  <Button
    onclick={() => a.open(() => (creating = true))}
    disabled={roomsWithoutOverride.length === 0}
  >
    {m.threshold_create()}
  </Button>
</div>

<div class="mt-6 flex flex-wrap items-center gap-2">
  <label class="relative min-w-0 flex-1 sm:w-64 sm:flex-none">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_rooms()}</span>
    <input
      type="search"
      bind:value={query}
      oninput={() => (page = 1)}
      placeholder={m.search_rooms()}
      class="w-full rounded-md border bg-canvas py-1 pr-7 pl-8 text-sm placeholder:text-overlay focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    />
    {#if query}
      <button
        type="button"
        onclick={() => {
          query = '';
          page = 1;
        }}
        aria-label={m.clear()}
        class="absolute top-1/2 right-1 -translate-y-1/2 rounded p-1 text-overlay hover:text-text"
      >
        <X class="size-3.5" aria-hidden="true" />
      </button>
    {/if}
  </label>

  <label class="shrink-0">
    <span class="sr-only">{m.per_page()}</span>
    <select
      bind:value={pageSize}
      onchange={() => (page = 1)}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      {#each pageSizes as n (n)}
        <option value={n}>{n} {m.per_page_suffix()}</option>
      {/each}
    </select>
  </label>

  <span class="ml-auto text-sm text-subtext num">
    {#if query}{filtered.length} /
    {/if}{rows.length}
  </span>
</div>

{#if !dialogOpen}
  {@render errorLine()}
{/if}

<div class="mt-4 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
  <table class="w-full text-sm">
    <thead class="bg-crust text-left text-subtext sm:sticky sm:top-14 sm:z-10">
      <tr>
        {@render sortHeader('room', m.room_heading())}
        {@render sortHeader('temperature_min', m.temperature_min())}
        {@render sortHeader('temperature_max', m.temperature_max())}
        {@render sortHeader('humidity_min', m.humidity_min())}
        {@render sortHeader('humidity_max', m.humidity_max())}
        {@render sortHeader('updated_at', m.updated_at())}
        <th class="px-4 py-2"></th>
      </tr>
    </thead>
    <tbody class="divide-y">
      <tr class="bg-mantle/60">
        <td class="px-4 py-2 font-medium">
          {m.threshold_global()}
          <span class="ml-2 rounded-full bg-accent/10 px-2 py-0.5 text-xs font-medium text-accent"
            >{m.limits_global_badge()}</span
          >
        </td>
        {@render limitCells(data.global)}
        <td class="px-4 py-2">
          <div class="flex justify-end">
            <Button
              variant="accent-soft"
              class="px-3 py-1 text-xs"
              onclick={() => a.open(() => (editingGlobal = true))}
            >
              {m.edit()}
            </Button>
          </div>
        </td>
      </tr>
      {#each visible as t (t.id)}
        <tr>
          <td class="px-4 py-2 font-medium num">
            {#if t.room}
              <a
                href={resolve('/(app)/staff/rooms/[id]', { id: t.room.id })}
                class="hover:text-accent">{roomLabel(t.room)}</a
              >
            {:else}
              <span class="text-subtext">{t.room_id}</span>
            {/if}
          </td>
          {@render limitCells(t)}
          <td class="px-4 py-2">
            <div class="flex justify-end gap-2">
              <Button
                variant="accent-soft"
                class="px-3 py-1 text-xs"
                onclick={() => a.open(() => (editing = t))}>{m.edit()}</Button
              >
              <Button
                variant="danger-soft"
                class="px-3 py-1 text-xs"
                onclick={() => a.open(() => (deleting = t))}>{m.delete()}</Button
              >
            </div>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

{#if rows.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.thresholds_none()}</p>
{:else if filtered.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.search_no_match()}</p>
{/if}

{#if pageCount > 1}
  <div class="mt-3 flex items-center justify-between text-sm text-subtext">
    <span class="num"
      >{start + 1}-{Math.min(start + pageSize, sorted.length)} {m.of()} {sorted.length}</span
    >
    <div class="flex items-center gap-1">
      <button
        type="button"
        onclick={() => (page = Math.max(1, page - 1))}
        disabled={page === 1}
        aria-label={m.page_prev()}
        class="rounded-md p-1.5 hover:bg-surface-0/60 hover:text-text disabled:opacity-40 disabled:hover:bg-transparent"
      >
        <ChevronLeft class="size-4" aria-hidden="true" />
      </button>
      <span class="px-1 num">{page} / {pageCount}</span>
      <button
        type="button"
        onclick={() => (page = Math.min(pageCount, page + 1))}
        disabled={page === pageCount}
        aria-label={m.page_next()}
        class="rounded-md p-1.5 hover:bg-surface-0/60 hover:text-text disabled:opacity-40 disabled:hover:bg-transparent"
      >
        <ChevronRight class="size-4" aria-hidden="true" />
      </button>
    </div>
  </div>
{/if}

<Dialog
  open={creating}
  onclose={() => a.close(() => (creating = false))}
  title={m.threshold_create()}
>
  <form
    id="threshold-create"
    method="POST"
    action="?/create"
    use:enhance={a.track(() => (creating = false))}
  >
    <p class="text-subtext">{m.threshold_create_help()}</p>
    <div class="mt-3">
      <span class="mb-1 block text-xs font-medium">{m.room_heading()}</span>
      <RoomPicker rooms={roomsWithoutOverride} />
    </div>
    <div class="mt-4">
      <ThresholdForm values={data.global} />
    </div>
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (creating = false))}
      >{m.cancel()}</Button
    >
    <Button type="submit" form="threshold-create">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={editing !== null}
  onclose={() => a.close(() => (editing = null))}
  title={m.threshold_edit()}
>
  {#if editing}
    <form
      id="threshold-edit"
      method="POST"
      action="?/update"
      use:enhance={a.track(() => (editing = null))}
    >
      <input type="hidden" name="room_id" value={editing.room_id} />
      <p class="mb-3 font-medium num">{roomLabel(editing.room)}</p>
      <ThresholdForm values={editing} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editing = null))}>{m.cancel()}</Button
    >
    <Button type="submit" form="threshold-edit">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={editingGlobal}
  onclose={() => a.close(() => (editingGlobal = false))}
  title={m.threshold_edit_global()}
>
  <form
    id="threshold-global"
    method="POST"
    action="?/updateGlobal"
    use:enhance={a.track(() => (editingGlobal = false))}
  >
    <p class="text-subtext">{m.threshold_global_help()}</p>
    <div class="mt-3">
      <ThresholdForm values={data.global} />
    </div>
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editingGlobal = false))}
      >{m.cancel()}</Button
    >
    <Button type="submit" form="threshold-global">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={deleting !== null}
  onclose={() => a.close(() => (deleting = null))}
  title={m.threshold_delete()}
>
  {#if deleting}
    <p>{m.threshold_delete_help()}</p>
    <p class="mt-2 font-medium num">{roomLabel(deleting.room)}</p>
    <form
      id="threshold-delete"
      method="POST"
      action="?/delete"
      use:enhance={a.track(() => (deleting = null))}
    >
      <input type="hidden" name="room_id" value={deleting.room_id} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (deleting = null))}
      >{m.cancel()}</Button
    >
    <Button type="submit" variant="danger" form="threshold-delete">{m.delete()}</Button>
  {/snippet}
</Dialog>
