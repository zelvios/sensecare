<script lang="ts">
  import { invalidateAll } from '$app/navigation';
  import { Droplets, Thermometer } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import ReadingCard from '$lib/components/rooms/ReadingCard.svelte';
  import PeriodPicker from '$lib/components/rooms/PeriodPicker.svelte';
  import HistoryTable from '$lib/components/rooms/HistoryTable.svelte';

  let { data } = $props();

  // Refresh every minute while the tab is visible. Reruns the load, Svelte patches only what changed.
  $effect(() => {
    const id = setInterval(() => {
      if (document.visibilityState === 'visible') invalidateAll();
    }, 60_000);
    return () => clearInterval(id);
  });
</script>

<svelte:head>
  <title>{m.title_my_room()}</title>
</svelte:head>

{#if !data.stay}
  <h1 class="text-xl font-semibold">{m.nav_my_room()}</h1>
  <p class="mt-4 rounded-xl bg-surface-0/40 px-4 py-6 text-center text-subtext">
    {m.not_checked_in()}
  </p>
{:else}
  <div class="flex flex-wrap items-end justify-between gap-4">
    <div>
      <h1 class="text-xl font-semibold num">
        {m.room_heading()}
        {data.stay.room_number}
      </h1>
      <p class="mt-1 text-sm text-subtext num">
        {m.checked_in_at()}
        {fmtDateTime(data.stay.checked_in_at)}
      </p>
    </div>
    {#if data.latest}
      <p class="text-sm text-subtext num">
        {m.last_reading()}
        {fmtDateTime(data.latest.measured_at)}
      </p>
    {/if}
  </div>

  <div class="mt-6 grid gap-4 sm:grid-cols-2">
    <ReadingCard
      label={m.temperature()}
      value={data.latest?.temperature_c ?? null}
      display={data.latest ? fmtTemp(data.latest.temperature_c) : '--'}
      min={data.thresholds?.temperature_min ?? -Infinity}
      max={data.thresholds?.temperature_max ?? Infinity}
    >
      {#snippet icon()}
        <Thermometer class="size-4" aria-hidden="true" />
      {/snippet}
    </ReadingCard>
    <ReadingCard
      label={m.humidity()}
      value={data.latest?.humidity_pct ?? null}
      display={data.latest ? fmtHumidity(data.latest.humidity_pct) : '--'}
      min={data.thresholds?.humidity_min ?? -Infinity}
      max={data.thresholds?.humidity_max ?? Infinity}
    >
      {#snippet icon()}
        <Droplets class="size-4" aria-hidden="true" />
      {/snippet}
    </ReadingCard>
  </div>

  <div class="mt-8 flex items-center gap-3">
    <h2 class="text-lg font-semibold">{m.history()}</h2>
    <span class="h-px flex-1 bg-surface-0" aria-hidden="true"></span>
    <PeriodPicker current={data.hours} />
  </div>
  <div class="mt-3">
    <HistoryTable rows={data.history} />
  </div>
{/if}
