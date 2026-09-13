<script lang="ts">
  import { enhance } from '$app/forms';
  import { invalidateAll } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { ChevronLeft, ChevronRight, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { fmtDateTimeSec } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import type { ServiceCall } from '$lib/api/types';

  let { data } = $props();
  const a = actionState();

  type Status = ServiceCall['status'];
  let simulating = $state(false);
  let closing = $state<ServiceCall | null>(null);

  const statusLabel: Record<Status, () => string> = {
    open: m.status_open,
    in_progress: m.status_in_progress,
    closed: m.status_closed
  };
  const statusTone: Record<Status, string> = {
    open: 'bg-danger/10 text-danger',
    in_progress: 'bg-warn/10 text-warn',
    closed: 'bg-surface-0 text-subtext'
  };
  const statusRank: Record<Status, number> = { open: 0, in_progress: 1, closed: 2 };

  const pageSizes = [5, 10, 15, 20, 30];
  let query = $state('');
  let statusFilter = $state<Status | 'all'>('all');
  let pageSize = $state(10);
  let page = $state(1);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return data.calls.filter((c) => {
      if (statusFilter !== 'all' && c.status !== statusFilter) return false;
      if (!q) return true;
      return (
        c.room_number.toLowerCase().includes(q) ||
        statusLabel[c.status]().toLowerCase().includes(q) ||
        (c.note ?? '').toLowerCase().includes(q) ||
        fmtDateTimeSec(c.created_at).includes(q)
      );
    });
  });
  const filtering = $derived(query.trim() !== '' || statusFilter !== 'all');

  type SortKey = 'room' | 'status' | 'created_at' | 'acknowledged_at' | 'closed_at';
  let sortKey = $state<SortKey>('created_at');
  let sortDir = $state<'asc' | 'desc'>('desc');

  function toggleSort(key: SortKey) {
    if (sortKey === key) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    else {
      sortKey = key;
      sortDir = key === 'room' || key === 'status' ? 'asc' : 'desc';
    }
    page = 1;
  }

  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
  const byTime =
    (get: (c: ServiceCall) => string | null | undefined) => (x: ServiceCall, y: ServiceCall) =>
      (get(x) ?? '').localeCompare(get(y) ?? '');

  const compare: Record<SortKey, (x: ServiceCall, y: ServiceCall) => number> = {
    room: (x, y) => collator.compare(x.room_number, y.room_number),
    status: (x, y) =>
      statusRank[x.status] - statusRank[y.status] || byTime((c) => c.created_at)(y, x),
    created_at: byTime((c) => c.created_at),
    acknowledged_at: byTime((c) => c.acknowledged_at),
    closed_at: byTime((c) => c.closed_at)
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
    if (a.error === 'unauthorized') return m.device_auth_failed();
    if (a.error === 'conflict') return m.device_unassigned();
    return m.action_failed();
  });

  $effect(() => {
    const id = setInterval(() => {
      if (document.visibilityState === 'visible') invalidateAll();
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
  <title>{m.title_calls()}</title>
</svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
  <h1 class="text-3xl font-semibold tracking-tight">{m.calls()}</h1>
  {#if data.user.role === 'admin'}
    <Button onclick={() => a.open(() => (simulating = true))}>{m.call_simulate()}</Button>
  {/if}
</div>

<div class="mt-6 flex flex-wrap items-center gap-2">
  <label class="relative min-w-0 flex-1 sm:w-64 sm:flex-none">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_calls()}</span>
    <input
      type="search"
      bind:value={query}
      oninput={() => (page = 1)}
      placeholder={m.search_calls()}
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
      bind:value={statusFilter}
      onchange={() => (page = 1)}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="all">{m.all()}</option>
      <option value="open">{m.status_open()}</option>
      <option value="in_progress">{m.status_in_progress()}</option>
      <option value="closed">{m.status_closed()}</option>
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
    {/if}{data.calls.length}
  </span>
</div>

{#if !simulating && !closing}
  {@render errorLine()}
{/if}

{#if data.calls.length === 0}
  <p class="mt-4 text-sm text-subtext">{m.calls_none()}</p>
{:else if filtered.length === 0}
  <p class="mt-4 text-sm text-subtext">{m.calls_no_match()}</p>
{:else}
  <div class="mt-4 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
    <table class="w-full text-sm">
      <thead class="bg-crust text-left text-subtext sm:sticky sm:top-14 sm:z-10">
        <tr>
          {@render sortHeader('room', m.room_heading())}
          {@render sortHeader('status', m.status())}
          {@render sortHeader('created_at', m.created())}
          {@render sortHeader('acknowledged_at', m.acknowledged_at())}
          {@render sortHeader('closed_at', m.closed())}
          <th class="px-4 py-2 font-medium">{m.note()}</th>
          <th class="px-4 py-2"></th>
        </tr>
      </thead>
      <tbody class="divide-y">
        {#each visible as c (c.id)}
          <tr>
            <td class="px-4 py-2 font-medium num">
              <a
                href={resolve('/(app)/staff/rooms/[id]', { id: c.room_id })}
                class="hover:text-accent"
              >
                {c.room_number}
              </a>
            </td>
            <td class="px-4 py-2">
              <span class="rounded-full px-2 py-0.5 text-xs font-medium {statusTone[c.status]}">
                {statusLabel[c.status]()}
              </span>
            </td>
            <td class="px-4 py-2 num">{fmtDateTimeSec(c.created_at)}</td>
            <td class="px-4 py-2 num">
              {c.acknowledged_at ? fmtDateTimeSec(c.acknowledged_at) : ''}
            </td>
            <td class="px-4 py-2 num">{c.closed_at ? fmtDateTimeSec(c.closed_at) : ''}</td>
            <td class="max-w-xs truncate px-4 py-2 text-subtext" title={c.note ?? ''}>
              {c.note ?? ''}
            </td>
            <td class="px-4 py-2">
              <div class="flex justify-end gap-2">
                {#if c.status === 'open'}
                  <form method="POST" action="?/acknowledge" use:enhance={a.track()}>
                    <input type="hidden" name="id" value={c.id} />
                    <Button type="submit" variant="warn-soft" class="px-3 py-1 text-xs">
                      {m.acknowledge()}
                    </Button>
                  </form>
                {/if}
                {#if c.status !== 'closed'}
                  <Button
                    variant="danger-soft"
                    class="px-3 py-1 text-xs"
                    onclick={() => a.open(() => (closing = c))}
                  >
                    {m.close_call()}
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
  open={simulating}
  onclose={() => a.close(() => (simulating = false))}
  title={m.call_simulate()}
>
  <form
    id="call-simulate"
    method="POST"
    action="?/simulate"
    use:enhance={a.track(() => (simulating = false))}
  >
    <p class="text-subtext">{m.call_simulate_help()}</p>
    <div class="mt-3 space-y-4">
      <Input
        label={m.device_id()}
        name="device_id"
        required
        autocomplete="off"
        spellcheck="false"
      />
      <Input label={m.device_key()} name="device_key" type="password" required autocomplete="off" />
    </div>
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (simulating = false))}>
      {m.cancel()}
    </Button>
    <Button type="submit" form="call-simulate">{m.call_simulate()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={closing !== null}
  onclose={() => a.close(() => (closing = null))}
  title={m.confirm_close()}
>
  {#if closing}
    <form
      id="call-close"
      method="POST"
      action="?/close"
      use:enhance={a.track(() => (closing = null))}
    >
      <input type="hidden" name="id" value={closing.id} />
      <p class="text-subtext">{m.close_call_help()}</p>
      <label class="mt-3 block">
        <span class="mb-1 block text-xs font-medium">{m.note()}</span>
        <textarea
          name="note"
          rows="3"
          maxlength="500"
          placeholder={m.note_placeholder()}
          class="max-h-40 w-full resize-none overflow-y-auto rounded-md border bg-canvas px-2 py-1.5 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
          >{closing.note ?? ''}</textarea
        >
      </label>
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (closing = null))}>
      {m.cancel()}
    </Button>
    <Button type="submit" variant="danger" form="call-close">{m.confirm_close()}</Button>
  {/snippet}
</Dialog>
