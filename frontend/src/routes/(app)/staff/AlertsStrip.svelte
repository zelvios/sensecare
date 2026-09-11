<script lang="ts">
  import { resolve } from '$app/paths';
  import { Bell, BellRing, Droplets, Thermometer } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTimeSec } from '$lib/utils/format';
  import type { RoomOverview } from '$lib/api/types';

  let { rooms }: { rooms: RoomOverview[] } = $props();

  type Alert = {
    roomId: string;
    roomLabel: string;
    kind: 'call' | 'alarm';
    alarmKind?: string;
    label: string;
    at: string;
    acknowledged: boolean;
  };

  /** "Floor 2, 12 - Kardiologi", the same wording as the room card. */
  function roomLabel(room: RoomOverview['room']): string {
    const floor = room.floor ?? null;
    const head = floor !== null ? `${m.floor()} ${floor}, ${room.room_number}` : room.room_number;
    return room.name ? `${head} - ${room.name}` : head;
  }

  const alarmLabel: Record<string, () => string> = {
    temperature_low: m.alarm_temperature_low,
    temperature_high: m.alarm_temperature_high,
    humidity_low: m.alarm_humidity_low,
    humidity_high: m.alarm_humidity_high
  };

  const alerts = $derived.by(() => {
    const list: Alert[] = [];
    for (const r of rooms) {
      for (const c of r.open_calls) {
        list.push({
          roomId: r.room.id,
          roomLabel: roomLabel(r.room),
          kind: 'call',
          label: c.status === 'in_progress' ? m.call_in_progress() : m.call_open(),
          at: c.created_at,
          acknowledged: c.status === 'in_progress'
        });
      }
      for (const a of r.open_alarms) {
        list.push({
          roomId: r.room.id,
          roomLabel: roomLabel(r.room),
          kind: 'alarm',
          alarmKind: a.kind,
          label: alarmLabel[a.kind]?.() ?? a.kind,
          at: a.raised_at,
          acknowledged: a.acknowledged
        });
      }
    }
    return list.sort((x, y) => x.at.localeCompare(y.at));
  });
</script>

<section aria-labelledby="alerts-heading">
  <div class="flex items-center gap-3">
    <h2 class="text-lg font-semibold" id="alerts-heading">{m.alerts()}</h2>
    <span class="h-px flex-1 bg-surface-0" aria-hidden="true"></span>
    <span class="text-sm text-subtext num">{alerts.length}</span>
  </div>

  {#if alerts.length === 0}
    <p class="mt-3 text-sm text-ok">{m.alerts_none()}</p>
  {:else}
    <ul class="mt-3 max-h-60 divide-y overflow-y-auto rounded-2xl bg-canvas ring-1 ring-surface-0">
      {#each alerts as a (a.roomId + a.kind + a.at)}
        <li>
          <a
            href={resolve('/(app)/staff/rooms/[id]', { id: a.roomId })}
            class="flex items-center gap-3 px-4 py-3 hover:bg-surface-0/40"
          >
            {#if a.kind === 'call'}
              {#if a.acknowledged}
                <Bell class="size-5 shrink-0 text-warn" aria-hidden="true" />
              {:else}
                <BellRing class="size-5 shrink-0 text-danger" aria-hidden="true" />
              {/if}
            {:else if a.alarmKind?.startsWith('temperature')}
              <Thermometer class="size-5 shrink-0 text-danger" aria-hidden="true" />
            {:else}
              <Droplets class="size-5 shrink-0 text-danger" aria-hidden="true" />
            {/if}

            <span class="shrink-0 font-medium num">{a.roomLabel}</span>
            <span class="min-w-0 flex-1 truncate text-subtext">{a.label}</span>
            <span class="shrink-0 text-sm text-subtext num">{fmtDateTimeSec(a.at)}</span>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</section>
