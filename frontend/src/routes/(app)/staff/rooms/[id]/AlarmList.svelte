<script lang="ts">
  import { enhance } from '$app/forms';
  import { ChevronLeft, ChevronRight, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { fmtDateTimeSec, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import type { Alarm } from '$lib/api/types';

  let { alarms }: { alarms: Alarm[] } = $props();
  let resolving = $state<Alarm | null>(null);

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
  let pageSize = $state(5);
  let page = $state(1);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return alarms.filter((x) => {
      if (stateFilter !== 'all' && stateOf(x) !== stateFilter) return false;
      if (kindFilter !== 'all' && x.kind !== kindFilter) return false;
      if (!q) return true;
      return (
        kindLabel[x.kind]().toLowerCase().includes(q) ||
        kindShort[x.kind]().toLowerCase().includes(q) ||
        stateLabel[stateOf(x)]().toLowerCase().includes(q) ||
        fmtDateTimeSec(x.raised_at).includes(q)
      );
    });
  });
  const filtering = $derived(query.trim() !== '' || stateFilter !== 'all' || kindFilter !== 'all');
  const openCount = $derived(alarms.filter((x) => !x.resolved_at).length);

  const pageCount = $derived(Math.max(1, Math.ceil(filtered.length / pageSize)));
  $effect(() => {
    if (page > pageCount) page = pageCount;
  });
  const start = $derived((page - 1) * pageSize);
  const visible = $derived(filtered.slice(start, start + pageSize));
</script>

<section aria-labelledby="alarms-heading">
  <div class="flex items-center gap-3">
    <h2 class="text-lg font-semibold" id="alarms-heading">{m.alarms()}</h2>
    {#if openCount > 0}
      <span class="rounded-full bg-danger/10 px-2 py-0.5 text-xs font-medium text-danger">
        {openCount}
        {m.open_lower()}
      </span>
    {/if}
    <span aria-hidden="true" class="h-px flex-1 bg-surface-0"></span>
    <span class="text-sm text-subtext num">
      {#if filtering}{filtered.length} /
      {/if}{alarms.length}
    </span>
  </div>

  {#if alarms.length === 0}
    <p class="mt-3 text-sm text-ok">{m.alarms_none_ever()}</p>
  {:else}
    <div class="mt-3 space-y-2">
      <label class="relative block">
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
      <div class="flex flex-wrap items-center gap-2">
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
      </div>
    </div>

    {#if filtered.length === 0}
      <p class="mt-3 text-sm text-subtext">{m.alarms_no_match()}</p>
    {:else}
      <ul class="mt-3 divide-y rounded-2xl bg-canvas ring-1 ring-surface-0">
        {#each visible as x (x.id)}
          {@const s = stateOf(x)}
          <li class="px-4 py-3">
            <div class="flex flex-wrap items-center gap-2">
              <span class="rounded-full px-2 py-0.5 text-xs font-medium {stateTone[s]}">
                {stateLabel[s]()}
              </span>
              <span
                class="rounded-full px-2 py-0.5 text-xs font-medium {kindTone[x.kind]}"
                title={kindLabel[x.kind]()}
              >
                {kindShort[x.kind]()}
              </span>
              <span
                class="text-sm font-medium num {s === 'resolved' ? 'text-subtext' : 'text-danger'}"
              >
                {fmtValue(x, x.measured_value)}
              </span>
              <span class="text-xs text-subtext num">
                {m.limit()}
                {fmtValue(x, x.threshold_value)}
              </span>
            </div>

            <div class="mt-1.5 flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
              <dl class="flex flex-wrap gap-x-4 gap-y-0.5 text-xs text-subtext num">
                <div class="flex gap-1">
                  <dt class="font-medium">{m.raised_at()}</dt>
                  <dd>{fmtDateTimeSec(x.raised_at)}</dd>
                </div>
                {#if x.acknowledged_at}
                  <div class="flex gap-1">
                    <dt class="font-medium">{m.acknowledged_at()}</dt>
                    <dd>{fmtDateTimeSec(x.acknowledged_at)}</dd>
                  </div>
                {/if}
                {#if x.resolved_at}
                  <div class="flex gap-1">
                    <dt class="font-medium">{m.resolved_at()}</dt>
                    <dd>{fmtDateTimeSec(x.resolved_at)}</dd>
                  </div>
                {/if}
              </dl>

              {#if s !== 'resolved'}
                <div class="flex shrink-0 gap-2">
                  {#if s === 'new'}
                    <form method="POST" action="?/acknowledgeAlarm" use:enhance>
                      <input type="hidden" name="id" value={x.id} />
                      <Button type="submit" variant="warn-soft" class="px-3 py-1 text-xs">
                        {m.acknowledge()}
                      </Button>
                    </form>
                  {/if}
                  <Button
                    variant="danger-soft"
                    class="px-3 py-1 text-xs"
                    onclick={() => (resolving = x)}
                  >
                    {m.resolve()}
                  </Button>
                </div>
              {/if}
            </div>
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

<Dialog onclose={() => (resolving = null)} open={resolving !== null} title={m.resolve_alarm()}>
  {#if resolving}
    <p class="text-subtext">{m.resolve_alarm_help()}</p>
    <p class="mt-2 font-medium num">
      {kindLabel[resolving.kind]()}, {fmtValue(resolving, resolving.measured_value)}
    </p>
    <form
      id="resolve-alarm-form"
      method="POST"
      action="?/resolveAlarm"
      use:enhance={() =>
        async ({ update }) => {
          await update();
          resolving = null;
        }}
    >
      <input type="hidden" name="id" value={resolving.id} />
    </form>
  {/if}

  {#snippet footer()}
    <Button variant="secondary" onclick={() => (resolving = null)}>{m.cancel()}</Button>
    <Button type="submit" variant="danger" form="resolve-alarm-form">{m.resolve()}</Button>
  {/snippet}
</Dialog>
