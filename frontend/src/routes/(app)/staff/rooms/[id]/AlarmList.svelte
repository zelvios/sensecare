<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime } from '$lib/utils/format';
  import type { OpenAlarm } from '$lib/api/types';

  let { alarms }: { alarms: OpenAlarm[] } = $props();

  const alarmLabel: Record<string, () => string> = {
    temperature_low: m.alarm_temperature_low,
    temperature_high: m.alarm_temperature_high,
    humidity_low: m.alarm_humidity_low,
    humidity_high: m.alarm_humidity_high
  };
</script>

<section aria-labelledby="alarms-heading">
  <div class="flex items-center gap-3">
    <h2 class="text-lg font-semibold" id="alarms-heading">{m.alarms()}</h2>
    <span class="h-px flex-1 bg-surface-0" aria-hidden="true"></span>
    <span class="text-sm text-subtext num">{alarms.length}</span>
  </div>

  {#if alarms.length === 0}
    <p class="mt-3 text-sm text-ok">{m.alarms_none()}</p>
  {:else}
    <ul class="mt-3 divide-y rounded-2xl bg-canvas ring-1 ring-surface-0">
      {#each alarms as a (a.id)}
        <li class="flex items-center gap-3 px-4 py-3 text-sm">
          <span
            class="rounded-full px-2 py-0.5 text-xs font-medium {a.acknowledged
              ? 'bg-warn/10 text-warn'
              : 'bg-danger/10 text-danger'}"
          >
            {a.acknowledged ? m.acknowledged() : m.unacknowledged()}
          </span>
          <span class="min-w-0 flex-1 truncate">{alarmLabel[a.kind]?.() ?? a.kind}</span>
          <span class="shrink-0 text-subtext num">{fmtDateTime(a.raised_at)}</span>
        </li>
      {/each}
    </ul>
  {/if}
</section>
