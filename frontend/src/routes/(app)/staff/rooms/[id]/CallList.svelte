<script lang="ts">
  import { enhance } from '$app/forms';
  import { ChevronLeft, ChevronRight, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { fmtDateTimeSec } from '$lib/utils/format';
  import type { ServiceCall } from '$lib/api/types';

  let { calls }: { calls: ServiceCall[] } = $props();
  let closing = $state<ServiceCall | null>(null);

  type Status = ServiceCall['status'];

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

  const pageSizes = [5, 10, 15, 20, 30];
  let query = $state('');
  let status = $state<Status | 'all'>('all');
  let pageSize = $state(5);
  let page = $state(1);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return calls.filter((c) => {
      if (status !== 'all' && c.status !== status) return false;
      if (!q) return true;
      return (
        statusLabel[c.status]().toLowerCase().includes(q) ||
        (c.note ?? '').toLowerCase().includes(q) ||
        fmtDateTimeSec(c.created_at).includes(q)
      );
    });
  });
  const filtering = $derived(query.trim() !== '' || status !== 'all');

  const pageCount = $derived(Math.max(1, Math.ceil(filtered.length / pageSize)));
  $effect(() => {
    if (page > pageCount) page = pageCount;
  });
  const start = $derived((page - 1) * pageSize);
  const visible = $derived(filtered.slice(start, start + pageSize));
</script>

<section aria-labelledby="calls-heading">
  <div class="flex items-center gap-3">
    <h2 class="text-lg font-semibold" id="calls-heading">{m.calls()}</h2>
    <span aria-hidden="true" class="h-px flex-1 bg-surface-0"></span>
    <span class="text-sm text-subtext num">
      {#if filtering}{filtered.length} /
      {/if}{calls.length}
    </span>
  </div>

  {#if calls.length === 0}
    <p class="mt-3 text-sm text-subtext">{m.calls_none()}</p>
  {:else}
    <div class="mt-3 flex flex-wrap items-center gap-2">
      <label class="relative min-w-0 flex-1 sm:w-56 sm:flex-none">
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
          bind:value={status}
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
    </div>

    {#if filtered.length === 0}
      <p class="mt-3 text-sm text-subtext">{m.calls_no_match()}</p>
    {:else}
      <ul class="mt-3 divide-y rounded-2xl bg-canvas ring-1 ring-surface-0">
        {#each visible as c (c.id)}
          <li class="px-4 py-3">
            <div class="flex flex-wrap items-center gap-3">
              <span class="rounded-full px-2.5 py-0.5 text-sm font-medium {statusTone[c.status]}">
                {statusLabel[c.status]()}
              </span>

              <dl class="flex min-w-0 flex-1 flex-wrap gap-x-4 gap-y-0.5 text-xs text-subtext num">
                <div class="flex gap-1">
                  <dt class="font-medium">{m.created()}</dt>
                  <dd>{fmtDateTimeSec(c.created_at)}</dd>
                </div>
                {#if c.acknowledged_at}
                  <div class="flex gap-1">
                    <dt class="font-medium">{m.acknowledged_at()}</dt>
                    <dd>{fmtDateTimeSec(c.acknowledged_at)}</dd>
                  </div>
                {/if}
                {#if c.closed_at}
                  <div class="flex gap-1">
                    <dt class="font-medium">{m.closed()}</dt>
                    <dd>{fmtDateTimeSec(c.closed_at)}</dd>
                  </div>
                {/if}
              </dl>

              {#if c.status === 'open'}
                <form method="POST" action="?/acknowledgeCall" use:enhance>
                  <input type="hidden" name="id" value={c.id} />
                  <Button type="submit" variant="warn-soft" class="px-3 py-1 text-sm">
                    {m.acknowledge()}
                  </Button>
                </form>
              {/if}
              {#if c.status !== 'closed'}
                <Button
                  variant="danger-soft"
                  class="px-3 py-1 text-sm"
                  onclick={() => (closing = c)}
                >
                  {m.close_call()}
                </Button>
              {/if}
            </div>

            {#if c.note}
              <p class="mt-1 text-subtext">{c.note}</p>
            {/if}
          </li>
        {/each}
      </ul>

      {#if pageCount > 1}
        <div class="mt-3 flex items-center justify-between text-sm text-subtext">
          <span class="num">
            {start + 1}-{Math.min(start + pageSize, filtered.length)}
            {m.of()}
            {filtered.length}
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
  {/if}
</section>

<Dialog open={closing !== null} onclose={() => (closing = null)} title={m.confirm_close()}>
  {#if closing}
    <form
      id="close-call-form"
      method="POST"
      action="?/closeCall"
      use:enhance={() =>
        async ({ update }) => {
          await update();
          closing = null;
        }}
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

  {#snippet footer()}
    <Button variant="secondary" onclick={() => (closing = null)}>{m.cancel()}</Button>
    <Button type="submit" variant="danger" form="close-call-form">{m.confirm_close()}</Button>
  {/snippet}
</Dialog>
