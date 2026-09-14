<script lang="ts">
  import { ChevronLeft, ChevronRight } from '@lucide/svelte';
  import { resolve } from '$app/paths';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime } from '$lib/utils/format';
  import type { Stay } from '$lib/api/types';

  let {
    stays,
    show,
    linkTarget = 'none'
  }: {
    stays: Stay[];
    /** Which side of the stay to show in the first column. */
    show: 'patient' | 'room';
    /** Where the first column links. Admins can link patients, everyone can link rooms. */
    linkTarget?: 'none' | 'admin-account' | 'room';
  } = $props();

  const pageSizes = [5, 10, 15, 20, 30];
  let pageSize = $state(5);
  let page = $state(1);

  const pageCount = $derived(Math.max(1, Math.ceil(stays.length / pageSize)));
  $effect(() => {
    if (page > pageCount) page = pageCount;
  });
  const start = $derived((page - 1) * pageSize);
  const visible = $derived(stays.slice(start, start + pageSize));

  /** Whole hours and minutes between two timestamps, or since check-in when still open. */
  function duration(s: Stay): string {
    const end = s.checked_out_at ? Date.parse(s.checked_out_at) : Date.now();
    const mins = Math.max(0, Math.round((end - Date.parse(s.checked_in_at)) / 60_000));
    const d = Math.floor(mins / 1440);
    const h = Math.floor((mins % 1440) / 60);
    if (d > 0) return `${d} ${m.unit_days()} ${h} ${m.unit_hours()}`;
    return `${h} ${m.unit_hours()} ${mins % 60} ${m.unit_minutes()}`;
  }
</script>

<section aria-labelledby="stays-heading">
  <div class="flex items-center gap-3">
    <h2 class="text-lg font-semibold" id="stays-heading">{m.stays()}</h2>
    <span class="h-px flex-1 bg-surface-0" aria-hidden="true"></span>
    <span class="text-sm text-subtext num">{stays.length}</span>
  </div>

  {#if stays.length === 0}
    <p class="mt-3 text-sm text-subtext">{m.stays_none()}</p>
  {:else}
    {#if stays.length > pageSizes[0]}
      <div class="mt-3 flex justify-end">
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
    {/if}

    <ul class="mt-3 divide-y rounded-2xl bg-canvas ring-1 ring-surface-0">
      {#each visible as s (s.id)}
        {@const open = !s.checked_out_at}
        <li class="px-4 py-3">
          <div class="flex flex-wrap items-center gap-2">
            <span
              class="rounded-full px-2 py-0.5 text-xs font-medium {open
                ? 'bg-ok/10 text-ok'
                : 'bg-surface-0 text-subtext'}"
            >
              {open ? m.checked_in() : m.stay_ended()}
            </span>
            {#if show === 'patient'}
              {#if linkTarget === 'admin-account'}
                <a
                  href={resolve('/(app)/admin/accounts/[id]', { id: s.user_id })}
                  class="font-medium hover:text-accent"
                >
                  {s.user_display_name}
                </a>
              {:else}
                <span class="font-medium">{s.user_display_name}</span>
              {/if}
            {:else if linkTarget === 'room'}
              <a
                href={resolve('/(app)/staff/rooms/[id]', { id: s.room_id })}
                class="font-medium num hover:text-accent"
              >
                {m.room_heading()}
                {s.room_number}
              </a>
            {:else}
              <span class="font-medium num">{m.room_heading()} {s.room_number}</span>
            {/if}
            <span class="ml-auto text-xs text-subtext num">{duration(s)}</span>
          </div>
          <dl class="mt-1.5 flex flex-wrap gap-x-4 gap-y-0.5 text-xs text-subtext num">
            <div class="flex gap-1">
              <dt class="font-medium">{m.checked_in_at()}</dt>
              <dd>{fmtDateTime(s.checked_in_at)}</dd>
            </div>
            {#if s.checked_out_at}
              <div class="flex gap-1">
                <dt class="font-medium">{m.checked_out_at()}</dt>
                <dd>{fmtDateTime(s.checked_out_at)}</dd>
              </div>
            {/if}
          </dl>
        </li>
      {/each}
    </ul>

    {#if pageCount > 1}
      <div class="mt-3 flex items-center justify-between text-sm text-subtext">
        <span class="num"
          >{start + 1}-{Math.min(start + pageSize, stays.length)} {m.of()} {stays.length}</span
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
  {/if}
</section>
