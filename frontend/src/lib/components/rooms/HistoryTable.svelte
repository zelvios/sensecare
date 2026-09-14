<script lang="ts">
  import { ChevronLeft, ChevronRight, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTimeSec, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import type { Measurement } from '$lib/api/types';

  let { rows }: { rows: Measurement[] } = $props();

  const pageSizes = [10, 20, 30, 50, 100];
  let query = $state('');
  let pageSize = $state(10);
  let page = $state(1);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return rows;
    return rows.filter(
      (r) =>
        fmtDateTimeSec(r.measured_at).toLowerCase().includes(q) ||
        fmtTemp(r.temperature_c).includes(q) ||
        fmtHumidity(r.humidity_pct).includes(q)
    );
  });

  const pageCount = $derived(Math.max(1, Math.ceil(filtered.length / pageSize)));
  $effect(() => {
    if (page > pageCount) page = pageCount;
  });
  const start = $derived((page - 1) * pageSize);
  const visible = $derived(filtered.slice(start, start + pageSize));
</script>

{#if rows.length === 0}
  <p class="rounded-xl bg-surface-0/40 px-4 py-6 text-center text-subtext">{m.history_empty()}</p>
{:else}
  <div class="flex flex-wrap items-center gap-2">
    <label class="relative min-w-0 flex-1 sm:w-64 sm:flex-none">
      <Search
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
      />
      <span class="sr-only">{m.search_history()}</span>
      <input
        type="search"
        bind:value={query}
        oninput={() => (page = 1)}
        placeholder={m.search_history()}
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

  {#if filtered.length === 0}
    <p class="mt-3 text-sm text-subtext">{m.search_no_match_history()}</p>
  {:else}
    <div class="mt-3 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
      <table class="w-full text-sm">
        <thead class="bg-crust text-left text-subtext sm:sticky sm:top-14 sm:z-10">
          <tr>
            <th class="px-4 py-2 font-medium">{m.col_time()}</th>
            <th class="px-4 py-2 text-right font-medium">{m.temperature()}</th>
            <th class="px-4 py-2 text-right font-medium">{m.humidity()}</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          {#each visible as r (r.id)}
            <tr>
              <td class="px-4 py-2 num">{fmtDateTimeSec(r.measured_at)}</td>
              <td class="px-4 py-2 text-right num">{fmtTemp(r.temperature_c)}</td>
              <td class="px-4 py-2 text-right num">{fmtHumidity(r.humidity_pct)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

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
