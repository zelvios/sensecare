<script lang="ts">
  import { enhance } from '$app/forms';
  import { invalidateAll } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { ArrowLeft, User, Wifi, WifiOff } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import ReadingCard from '$lib/components/rooms/ReadingCard.svelte';
  import PeriodPicker from '$lib/components/rooms/PeriodPicker.svelte';
  import HistoryTable from '$lib/components/rooms/HistoryTable.svelte';
  import ClientPicker from '$lib/components/rooms/ClientPicker.svelte';
  import CallList from './CallList.svelte';
  import AlarmList from './AlarmList.svelte';

  let { data } = $props();
  const a = actionState();
  let managingStay = $state(false);

  const r = $derived(data.room);
  const floor = $derived(r.room.floor ?? null);
  const heading = $derived(
    floor !== null ? `${m.floor()} ${floor}, ${r.room.room_number}` : r.room.room_number
  );
  const offline = $derived(
    !r.device ||
      !r.device.last_seen_at ||
      Date.now() - Date.parse(r.device.last_seen_at) > 2 * 60_000
  );

  const errorText = $derived.by(() => {
    if (!a.error) return null;
    if (a.error === 'conflict') return m.stay_conflict();
    return m.action_failed();
  });

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

<div>
  <a
    class="inline-flex items-center gap-1 text-sm text-subtext hover:text-text"
    href={resolve('/staff')}
  >
    <ArrowLeft aria-hidden="true" class="size-4" />
    {m.nav_overview()}
  </a>

  <div class="mt-3 flex flex-wrap items-start justify-between gap-4">
    <div>
      <h1 class="text-3xl font-semibold tracking-tight num">{heading}</h1>
      {#if r.room.name}
        <p class="mt-1 text-subtext">{r.room.name}</p>
      {/if}
    </div>

    <dl class="flex flex-wrap gap-x-6 gap-y-2 text-sm">
      <div class="flex items-center gap-1.5">
        <User aria-hidden="true" class="size-4 text-subtext" />
        <dt class="sr-only">{m.occupant()}</dt>
        <dd>
          <button
            type="button"
            onclick={() => a.open(() => (managingStay = true))}
            class="rounded underline-offset-2 hover:text-accent hover:underline"
          >
            {data.stay ? data.stay.user_display_name : m.room_empty()}
          </button>
        </dd>
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

<Dialog
  open={managingStay}
  onclose={() => a.close(() => (managingStay = false))}
  title={m.manage_occupant()}
>
  {#if data.stay}
    <div class="flex items-center justify-between gap-3">
      <div class="min-w-0">
        <p class="truncate font-medium">{data.stay.user_display_name}</p>
        <p class="text-xs text-subtext num">
          {m.checked_in_at()}
          {fmtDateTime(data.stay.checked_in_at)}
        </p>
      </div>
      <form method="POST" action="?/checkOut" use:enhance={a.track()}>
        <input type="hidden" name="stay_id" value={data.stay.id} />
        <Button type="submit" variant="warn-soft" class="px-3 py-1 text-xs">{m.check_out()}</Button>
      </form>
    </div>
    <p class="mt-3 text-xs text-subtext">{m.change_occupant_help()}</p>
  {:else if !r.room.is_active}
    <p class="text-subtext">{m.room_inactive_no_checkin()}</p>
  {:else}
    <form id="stay-checkin" method="POST" action="?/checkIn" use:enhance={a.track()}>
      <ClientPicker clients={data.clients} occupiedIn={data.occupiedIn} />
    </form>
  {/if}
  {#if errorText}
    <p class="mt-4 rounded-lg bg-danger/10 px-3 py-2 text-sm text-danger" role="alert">
      {errorText}
    </p>
  {/if}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (managingStay = false))}>
      {m.close()}
    </Button>
    {#if !data.stay && r.room.is_active}
      <Button type="submit" variant="ok-soft" form="stay-checkin">{m.check_in()}</Button>
    {/if}
  {/snippet}
</Dialog>
