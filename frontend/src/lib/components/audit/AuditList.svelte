<script lang="ts">
  import { ChevronLeft, ChevronRight, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTimeSec } from '$lib/utils/format';
  import type { AuditEntry } from '$lib/api/types';

  let { entries, empty }: { entries: AuditEntry[]; empty: string } = $props();

  const pageSizes = [5, 10, 15, 20, 30];
  let pageSize = $state(10);

  type Group = 'auth' | 'account' | 'room' | 'stay' | 'call' | 'alarm' | 'device' | 'threshold';

  const groupOf = (action: string): Group => {
    if (action.startsWith('user.log')) return 'auth';
    if (action.startsWith('user.')) return 'account';
    if (action.startsWith('room.')) return 'room';
    if (action.startsWith('stay.')) return 'stay';
    if (action.startsWith('service_call.')) return 'call';
    if (action.startsWith('alarm.')) return 'alarm';
    if (action.startsWith('device.')) return 'device';
    return 'threshold';
  };

  const groupLabel: Record<Group, () => string> = {
    auth: m.audit_group_auth,
    account: m.audit_group_account,
    room: m.audit_group_room,
    stay: m.audit_group_stay,
    call: m.audit_group_call,
    alarm: m.audit_group_alarm,
    device: m.audit_group_device,
    threshold: m.audit_group_threshold
  };

  const groupTone: Record<Group, string> = {
    auth: 'bg-info/10 text-info',
    account: 'bg-accent/10 text-accent',
    room: 'bg-ok/10 text-ok',
    stay: 'bg-ok/10 text-ok',
    call: 'bg-warn/10 text-warn',
    alarm: 'bg-danger/10 text-danger',
    device: 'bg-mauve/10 text-mauve',
    threshold: 'bg-teal/10 text-teal'
  };

  const labels: Record<string, () => string> = {
    'user.created': m.audit_user_created,
    'user.updated': m.audit_user_updated,
    'user.password_changed': m.audit_user_password_changed,
    'user.deactivated': m.audit_user_deactivated,
    'user.activated': m.audit_user_activated,
    'user.deleted': m.audit_user_deleted,
    'user.logged_in': m.audit_user_logged_in,
    'user.logged_out': m.audit_user_logged_out,
    'user.login_failed': m.audit_user_login_failed,
    'user.login_refused': m.audit_user_login_refused,
    'room.created': m.audit_room_created,
    'room.updated': m.audit_room_updated,
    'room.deactivated': m.audit_room_deactivated,
    'room.activated': m.audit_room_activated,
    'room.deleted': m.audit_room_deleted,
    'stay.checked_in': m.audit_stay_checked_in,
    'stay.checked_out': m.audit_stay_checked_out,
    'service_call.created': m.audit_call_created,
    'service_call.acknowledged': m.audit_call_acknowledged,
    'service_call.closed': m.audit_call_closed,
    'service_call.updated': m.audit_call_updated,
    'alarm.acknowledged': m.audit_alarm_acknowledged,
    'alarm.resolved': m.audit_alarm_resolved
  };
  const label = (action: string) => labels[action]?.() ?? action;

  const groups = $derived([...new Set(entries.map((e) => groupOf(e.action)))]);

  let query = $state('');
  let group = $state<Group | 'all'>('all');
  let page = $state(1);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return entries.filter((e) => {
      if (group !== 'all' && groupOf(e.action) !== group) return false;
      if (!q) return true;
      return (
        label(e.action).toLowerCase().includes(q) ||
        e.action.includes(q) ||
        JSON.stringify(e.details ?? '')
          .toLowerCase()
          .includes(q)
      );
    });
  });

  const pageCount = $derived(Math.max(1, Math.ceil(filtered.length / pageSize)));
  $effect(() => {
    if (page > pageCount) page = pageCount;
  });
  const start = $derived((page - 1) * pageSize);
  const visible = $derived(filtered.slice(start, start + pageSize));

  function detailPairs(details: unknown): { key: string; value: string }[] {
    if (!details || typeof details !== 'object') return [];
    return Object.entries(details as Record<string, unknown>).map(([key, v]) => {
      if (v && typeof v === 'object' && 'from' in v && 'to' in v) {
        const c = v as { from: unknown; to: unknown };
        return { key, value: `${fmt(c.from)} -> ${fmt(c.to)}` };
      }
      return { key, value: fmt(v) };
    });
  }
  const fmt = (v: unknown) =>
    v === null || v === undefined ? '-' : typeof v === 'object' ? JSON.stringify(v) : String(v);
</script>

{#if entries.length === 0}
  <p class="text-sm text-subtext">{empty}</p>
{:else}
  <div class="flex flex-wrap items-center gap-2">
    <label class="relative min-w-0 flex-1 sm:w-64 sm:flex-none">
      <Search
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
      />
      <span class="sr-only">{m.search_audit()}</span>
      <input
        type="search"
        bind:value={query}
        oninput={() => (page = 1)}
        placeholder={m.search_audit()}
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
      <span class="sr-only">{m.audit_group()}</span>
      <select
        bind:value={group}
        onchange={() => (page = 1)}
        class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
      >
        <option value="all">{m.all()}</option>
        {#each groups as g (g)}
          <option value={g}>{groupLabel[g]()}</option>
        {/each}
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
      {#if filtered.length !== entries.length}{filtered.length} /
      {/if}{entries.length}
    </span>
  </div>

  {#if filtered.length === 0}
    <p class="mt-3 text-sm text-subtext">{m.audit_no_match()}</p>
  {:else}
    <ul class="mt-3 divide-y rounded-2xl bg-canvas ring-1 ring-surface-0">
      {#each visible as e (e.id)}
        {@const g = groupOf(e.action)}
        {@const pairs = detailPairs(e.details)}
        <li class="px-4 py-3 text-sm">
          <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
            <span class="rounded-full px-2 py-0.5 text-xs font-medium {groupTone[g]}">
              {groupLabel[g]()}
            </span>
            <span class="font-medium">{label(e.action)}</span>
            <span class="ml-auto text-xs text-subtext num">{fmtDateTimeSec(e.created_at)}</span>
          </div>
          {#if pairs.length > 0}
            <dl class="mt-1.5 flex flex-wrap gap-x-4 gap-y-0.5 text-xs text-subtext num">
              {#each pairs as p (p.key)}
                <div class="flex gap-1">
                  <dt class="font-medium">{p.key}</dt>
                  <dd class="break-all">{p.value}</dd>
                </div>
              {/each}
            </dl>
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
