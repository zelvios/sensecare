<script lang="ts">
  import { invalidateAll } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { ArrowLeft, User, Wifi, WifiOff } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import ReadingCard from '$lib/components/rooms/ReadingCard.svelte';
  import PeriodPicker from '$lib/components/rooms/PeriodPicker.svelte';
  import HistoryTable from '$lib/components/rooms/HistoryTable.svelte';
  import CallList from './CallList.svelte';
  import AlarmList from './AlarmList.svelte';

  let { data } = $props();

  const r = $derived(data.room);
  const floor = $derived(r.room.floor ?? null);
  const offline = $derived(
    !r.device ||
      !r.device.last_seen_at ||
      Date.now() - Date.parse(r.device.last_seen_at) > 2 * 60_000
  );

  $effect(() => {
    const id = setInterval(() => {
      if (document.visibilityState === 'visible') invalidateAll();
    }, 15_000);
    return () => clearInterval(id);
  });
</script>

<svelte:head>
  <title>{m.room_heading()} {r.room.room_number} - SenseCare</title>
</svelte:head>

<div class="min-h-0 flex-1 overflow-y-auto p-1">
  <a
    class="inline-flex items-center gap-1 text-sm text-subtext hover:text-text"
    href={resolve('/staff')}
  >
    <ArrowLeft aria-hidden="true" class="size-4" />
    {m.nav_overview()}
  </a>

  <div class="mt-3 flex flex-wrap items-start justify-between gap-4">
    <div>
      <h1 class="text-3xl font-semibold tracking-tight num">
        {#if floor !== null}{m.floor()} {floor},
        {/if}{r.room.room_number}
      </h1>
      {#if r.room.name}
        <p class="mt-1 text-subtext">{r.room.name}</p>
      {/if}
    </div>

    <dl class="flex flex-wrap gap-x-6 gap-y-2 text-sm">
      <div class="flex items-center gap-1.5">
        <User aria-hidden="true" class="size-4 text-subtext" />
        <dt class="sr-only">{m.occupant()}</dt>
        <dd>{r.occupant ? r.occupant.display_name : m.room_empty()}</dd>
      </div>
      <div class="flex items-center gap-1.5">
        {#if offline}
          <WifiOff class="size-4 text-overlay" aria-hidden="true" />
        {:else}
          <Wifi class="size-4 text-ok" aria-hidden="true" />
        {/if}
        <dt class="sr-only">{m.device()}</dt>
        <dd class={offline ? 'text-subtext' : ''}>
          {#if !r.device}
            {m.device_none()}
          {:else if offline}
            {m.device_offline()}
          {:else}
            {m.device_online()}
          {/if}
        </dd>
      </div>
    </dl>
  </div>

  <div class="mt-6 grid gap-4 sm:grid-cols-2">
    <ReadingCard
      display={data.latest ? fmtTemp(data.latest.temperature_c) : '--'}
      label={m.temperature()}
      margin={1}
      max={data.thresholds?.temperature_max ?? Infinity}
      min={data.thresholds?.temperature_min ?? -Infinity}
      range={data.thresholds
        ? `${fmtTemp(data.thresholds.temperature_min)} - ${fmtTemp(data.thresholds.temperature_max)}`
        : undefined}
      value={data.latest?.temperature_c ?? null}
    />
    <ReadingCard
      display={data.latest ? fmtHumidity(data.latest.humidity_pct) : '--'}
      label={m.humidity()}
      margin={5}
      max={data.thresholds?.humidity_max ?? Infinity}
      min={data.thresholds?.humidity_min ?? -Infinity}
      range={data.thresholds
        ? `${fmtHumidity(data.thresholds.humidity_min)} - ${fmtHumidity(data.thresholds.humidity_max)}`
        : undefined}
      value={data.latest?.humidity_pct ?? null}
    />
  </div>
  <div class="mt-2 flex flex-wrap justify-between gap-x-6 gap-y-1 text-sm text-subtext">
    {#if data.latest}
      <p class="num">{m.last_reading()} {fmtDateTime(data.latest.measured_at)}</p>
    {/if}
    {#if data.thresholds}
      <p>
        {m.limits_source()}:
        {data.thresholds.source === 'room' ? m.limits_room() : m.limits_global()}
      </p>
    {/if}
  </div>

  <div class="mt-8 grid gap-8 lg:grid-cols-2">
    <AlarmList alarms={r.open_alarms} />
    <CallList calls={data.calls} />
  </div>

  <div class="mt-8 flex items-center gap-3">
    <h2 class="text-lg font-semibold">{m.history()}</h2>
    <span aria-hidden="true" class="h-px flex-1 bg-surface-0"></span>
    <PeriodPicker current={data.hours} />
  </div>
  <div class="mt-3">
    <HistoryTable rows={data.history} />
  </div>
</div>
