<script lang="ts">
  import { resolve } from '$app/paths';
  import { Bell, BellRing, User, WifiOff } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import type { RoomOverview } from '$lib/api/types';

  let { room: r }: { room: RoomOverview } = $props();

  // A device that has not reported for 2 minutes is treated as offline.
  const offline = $derived(
    !r.device ||
      !r.device.last_seen_at ||
      Date.now() - Date.parse(r.device.last_seen_at) > 2 * 60_000
  );
  const floor = $derived(r.room.floor ?? null);

  // Red until someone has acknowledged, orange while it is being handled.
  const openCall = $derived(r.open_calls[0] ?? null);
  const callUrgent = $derived(openCall?.status === 'open');
  const alarmUrgent = $derived(r.open_alarms.some((a) => !a.acknowledged));
  const anyOpen = $derived(openCall !== null || r.open_alarms.length > 0);

  const ring = $derived(
    callUrgent || alarmUrgent
      ? 'ring-2 ring-danger'
      : anyOpen
        ? 'ring-2 ring-warn'
        : 'ring-1 ring-surface-0'
  );

  const heading = $derived(
    floor !== null ? `${m.floor()} ${floor}, ${r.room.room_number}` : r.room.room_number
  );

  /** Colour for a reading: red while its alarm is unacknowledged, orange once acknowledged. */
  function tone(prefix: 'temperature' | 'humidity'): string {
    const alarms = r.open_alarms.filter((a) => a.kind.startsWith(prefix));
    if (alarms.length === 0) return '';
    return alarms.some((a) => !a.acknowledged) ? 'text-danger' : 'text-warn';
  }
</script>

<a
  class="block rounded-2xl bg-canvas p-4 shadow-md shadow-crust/50 transition hover:shadow-lg {ring}"
  href={resolve('/(app)/staff/rooms/[id]', { id: r.room.id })}
>
  <div class="flex items-start justify-between gap-2">
    <div class="min-w-0">
      <p class="text-lg font-semibold num">{heading}</p>
      {#if r.room.name}
        <p class="truncate text-sm text-subtext">{r.room.name}</p>
      {/if}
    </div>
    <div class="flex shrink-0 items-center gap-1.5">
      {#if openCall}
        {#if callUrgent}
          <BellRing class="size-5 text-danger" aria-label={m.call_open()} />
        {:else}
          <Bell class="size-5 text-warn" aria-label={m.call_in_progress()} />
        {/if}
      {/if}
      {#if offline}
        <WifiOff class="size-5 text-overlay" aria-label={m.device_offline()} />
      {/if}
    </div>
  </div>

  <dl class="mt-4 divide-y border-y text-sm">
    <div class="flex items-center justify-between py-2">
      <dt class="text-subtext">{m.temperature()}</dt>
      <dd class="font-medium num {tone('temperature')}">
        {r.latest ? fmtTemp(r.latest.temperature_c) : '--'}
      </dd>
    </div>
    <div class="flex items-center justify-between py-2">
      <dt class="text-subtext">{m.humidity()}</dt>
      <dd class="font-medium num {tone('humidity')}">
        {r.latest ? fmtHumidity(r.latest.humidity_pct) : '--'}
      </dd>
    </div>
  </dl>

  <div class="mt-3 flex items-center justify-between gap-3 text-xs text-subtext">
    <span class="flex min-w-0 items-center gap-1">
      <User aria-hidden="true" class="size-3.5 shrink-0" />
      <span class="truncate">{r.occupant ? r.occupant.display_name : m.room_empty()}</span>
    </span>
    {#if r.latest}
      <span class="shrink-0 num">{fmtDateTime(r.latest.measured_at)}</span>
    {/if}
  </div>
</a>
