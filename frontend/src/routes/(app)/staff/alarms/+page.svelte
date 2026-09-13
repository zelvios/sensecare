<script lang="ts">
  import { enhance } from '$app/forms';
  import { invalidateAll } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { ChevronLeft, ChevronRight, RefreshCw, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { fmtDateTimeSec, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import type { Alarm } from '$lib/api/types';

  let { data } = $props();
  const a = actionState();

  let refreshing = $state(false);
  let resolving = $state<Alarm | null>(null);

  async function refresh() {
    refreshing = true;
    try {
      await invalidateAll();
    } finally {
      refreshing = false;
    }
  }

  type State = 'new' | 'seen' | 'resolved';
  const stateOf = (x: Alarm): State =>
    x.resolved_at ? 'resolved' : x.acknowledged_at ? 'seen' : 'new';

  const stateLabel: Record<State, () => string> = {
    new: m.unacknowledged,
    seen: m.acknowledged,
    resolved: m.alarm_resolved
  };
  const stateTone: Record<State, string> = {
    new: 'bg-danger/10 text-danger',
    seen: 'bg-warn/10 text-warn',
    resolved: 'bg-surface-0 text-subtext'
  };
  const stateRank: Record<State, number> = { new: 0, seen: 1, resolved: 2 };

  type Kind = Alarm['kind'];
  const kindLabel: Record<Kind, () => string> = {
    temperature_low: m.alarm_temperature_low,
    temperature_high: m.alarm_temperature_high,
    humidity_low: m.alarm_humidity_low,
    humidity_high: m.alarm_humidity_high
  };
  const kindShort: Record<Kind, () => string> = {
    temperature_low: m.alarm_short_temperature_low,
    temperature_high: m.alarm_short_temperature_high,
    humidity_low: m.alarm_short_humidity_low,
    humidity_high: m.alarm_short_humidity_high
  };
  const kindTone: Record<Kind, string> = {
    temperature_low: 'bg-info/10 text-info',
    temperature_high: 'bg-mauve/10 text-mauve',
    humidity_low: 'bg-teal/10 text-teal',
    humidity_high: 'bg-accent/10 text-accent'
  };
  const fmtValue = (x: Alarm, v: number) =>
    x.kind.startsWith('temperature') ? fmtTemp(v) : fmtHumidity(v);

  const pageSizes = [5, 10, 15, 20, 30];
  let query = $state('');
  let stateFilter = $state<State | 'all'>('all');
  let kindFilter = $state<Kind | 'all'>('all');
  let pageSize = $state(10);
  let page = $state(1);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return data.alarms.filter((x) => {
      if (stateFilter !== 'all' && stateOf(x) !== stateFilter) return false;
      if (kindFilter !== 'all' && x.kind !== kindFilter) return false;
      if (!q) return true;
      return (
        x.room_number.toLowerCase().includes(q) ||
        kindLabel[x.kind]().toLowerCase().includes(q) ||
        kindShort[x.kind]().toLowerCase().includes(q) ||
        fmtDateTimeSec(x.raised_at).includes(q)
      );
    });
  });
  const filtering = $derived(query.trim() !== '' || stateFilter !== 'all' || kindFilter !== 'all');

  type SortKey =
    'room' | 'state' | 'kind' | 'measured' | 'raised_at' | 'acknowledged_at' | 'resolved_at';
  let sortKey = $state<SortKey>('raised_at');
  let sortDir = $state<'asc' | 'desc'>('desc');

  function toggleSort(key: SortKey) {
    if (sortKey === key) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    else {
      sortKey = key;
      sortDir = key === 'room' || key === 'state' || key === 'kind' ? 'asc' : 'desc';
    }
    page = 1;
  }

  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
  const byTime = (get: (x: Alarm) => string | null | undefined) => (x: Alarm, y: Alarm) =>
    (get(x) ?? '').localeCompare(get(y) ?? '');

  const compare: Record<SortKey, (x: Alarm, y: Alarm) => number> = {
    room: (x, y) => collator.compare(x.room_number, y.room_number),
    state: (x, y) =>
      stateRank[stateOf(x)] - stateRank[stateOf(y)] || byTime((z) => z.raised_at)(y, x),
    kind: (x, y) => collator.compare(kindShort[x.kind](), kindShort[y.kind]()),
    measured: (x, y) => x.measured_value - y.measured_value,
    raised_at: byTime((z) => z.raised_at),
    acknowledged_at: byTime((z) => z.acknowledged_at),
    resolved_at: byTime((z) => z.resolved_at)
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
    if (a.error === 'conflict') return m.alarm_conflict();
    return m.action_failed();
  });

  $effect(() => {
    const id = setInterval(() => {
      if (document.visibilityState === 'visible') refresh();
    }, 15_000);
    return () => clearInterval(id);
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

<svelte:head>
  <title>{m.title_alarms()}</title>
</svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
  <h1 class="text-3xl font-semibold tracking-tight">{m.alarms()}</h1>
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

<div class="mt-6 flex flex-wrap items-center gap-2">
  <label class="relative min-w-0 flex-1 sm:w-64 sm:flex-none">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_alarms()}</span>
    <input
      type="search"
      bind:value={query}
      oninput={() => (page = 1)}
      placeholder={m.search_alarms()}
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
    <span class="sr-only">{m.status()}</span>
    <select
      bind:value={stateFilter}
      onchange={() => (page = 1)}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="all">{m.all()}</option>
      <option value="new">{m.unacknowledged()}</option>
      <option value="seen">{m.acknowledged()}</option>
      <option value="resolved">{m.alarm_resolved()}</option>
    </select>
  </label>

  <label class="shrink-0">
    <span class="sr-only">{m.alarm_kind()}</span>
    <select
      bind:value={kindFilter}
      onchange={() => (page = 1)}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="all">{m.all_kinds()}</option>
      <option value="temperature_low">{m.alarm_short_temperature_low()}</option>
      <option value="temperature_high">{m.alarm_short_temperature_high()}</option>
      <option value="humidity_low">{m.alarm_short_humidity_low()}</option>
      <option value="humidity_high">{m.alarm_short_humidity_high()}</option>
    </select>
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
    {#if filtering}{filtered.length} /
    {/if}{data.alarms.length}
  </span>
</div>

{#if !resolving}
  {@render errorLine()}
{/if}

{#if data.alarms.length === 0}
  <p class="mt-4 text-sm text-subtext">{m.alarms_none_ever()}</p>
{:else if filtered.length === 0}
  <p class="mt-4 text-sm text-subtext">{m.alarms_no_match()}</p>
{:else}
  <div class="mt-4 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
    <table class="w-full text-sm">
      <thead class="bg-crust text-left text-subtext sm:sticky sm:top-14 sm:z-10">
        <tr>
          {@render sortHeader('room', m.room_heading())}
          {@render sortHeader('state', m.status())}
          {@render sortHeader('kind', m.alarm_kind())}
          {@render sortHeader('measured', m.measured())}
          <th class="px-4 py-2 font-medium">{m.limit()}</th>
          {@render sortHeader('raised_at', m.raised_at())}
          {@render sortHeader('acknowledged_at', m.acknowledged_at())}
          {@render sortHeader('resolved_at', m.resolved_at())}
          <th class="px-4 py-2"></th>
        </tr>
      </thead>
      <tbody class="divide-y">
        {#each visible as x (x.id)}
          {@const s = stateOf(x)}
          <tr>
            <td class="px-4 py-2 font-medium whitespace-nowrap num">
              <a
                href={resolve('/(app)/staff/rooms/[id]', { id: x.room_id })}
                class="hover:text-accent"
              >
                {x.room_number}
              </a>
            </td>
            <td class="px-4 py-2 whitespace-nowrap">
              <span class="rounded-full px-2 py-0.5 text-xs font-medium {stateTone[s]}">
                {stateLabel[s]()}
              </span>
            </td>
            <td class="px-4 py-2 whitespace-nowrap">
              <span
                class="rounded-full px-2 py-0.5 text-xs font-medium {kindTone[x.kind]}"
                title={kindLabel[x.kind]()}
              >
                {kindShort[x.kind]()}
              </span>
            </td>
            <td class="px-4 py-2 num {s === 'resolved' ? 'text-subtext' : 'text-danger'}">
              {fmtValue(x, x.measured_value)}
            </td>
            <td class="px-4 py-2 text-subtext num">{fmtValue(x, x.threshold_value)}</td>
            <td class="px-4 py-2 num">{fmtDateTimeSec(x.raised_at)}</td>
            <td class="px-4 py-2 num">
              {x.acknowledged_at ? fmtDateTimeSec(x.acknowledged_at) : ''}
            </td>
            <td class="px-4 py-2 num">{x.resolved_at ? fmtDateTimeSec(x.resolved_at) : ''}</td>
            <td class="px-4 py-2 whitespace-nowrap">
              <div class="flex justify-end gap-2">
                {#if s === 'new'}
                  <form method="POST" action="?/acknowledge" use:enhance={a.track()}>
                    <input type="hidden" name="id" value={x.id} />
                    <Button type="submit" variant="warn-soft" class="px-3 py-1 text-xs">
                      {m.acknowledge()}
                    </Button>
                  </form>
                {/if}
                {#if s !== 'resolved'}
                  <Button
                    variant="danger-soft"
                    class="px-3 py-1 text-xs"
                    onclick={() => a.open(() => (resolving = x))}
                  >
                    {m.resolve()}
                  </Button>
                {/if}
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if pageCount > 1}
    <div class="mt-3 flex items-center justify-between text-sm text-subtext">
      <span class="num">
        {start + 1}-{Math.min(start + pageSize, sorted.length)}
        {m.of()}
        {sorted.length}
      </span>
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
{/if}

<Dialog
  open={resolving !== null}
  onclose={() => a.close(() => (resolving = null))}
  title={m.resolve_alarm()}
>
  {#if resolving}
    <p class="text-subtext">{m.resolve_alarm_help()}</p>
    <p class="mt-2 font-medium num">
      {resolving.room_number} - {kindLabel[resolving.kind]()}, {fmtValue(
        resolving,
        resolving.measured_value
      )}
    </p>
    <form
      id="alarm-resolve"
      method="POST"
      action="?/resolve"
      use:enhance={a.track(() => (resolving = null))}
    >
      <input type="hidden" name="id" value={resolving.id} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (resolving = null))}>
      {m.cancel()}
    </Button>
    <Button type="submit" variant="danger" form="alarm-resolve">{m.resolve()}</Button>
  {/snippet}
</Dialog>
