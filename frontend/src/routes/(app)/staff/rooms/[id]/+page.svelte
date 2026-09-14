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
  import ThresholdForm from '$lib/components/rooms/ThresholdForm.svelte';
  import CallList from './CallList.svelte';
  import AlarmList from './AlarmList.svelte';

  let { data } = $props();
  const a = actionState();
  let managingStay = $state(false);
  let editingLimits = $state(false);
  const dialogOpen = $derived(managingStay || editingLimits);

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
    if (a.error === 'bad_request' || a.error === 'constraint_violation')
      return m.threshold_invalid();
    return m.action_failed();
  });

  $effect(() => {
    const id = setInterval(() => {
      if (document.visibilityState === 'visible') invalidateAll();
    }, 15_000);
    return () => clearInterval(id);
  });
</script>

{#snippet errorLine()}
  {#if errorText}
    <p class="mt-4 rounded-lg bg-danger/10 px-3 py-2 text-sm text-danger" role="alert">
      {errorText}
    </p>
  {/if}
{/snippet}

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

  <div class="mt-3">
    <h1 class="text-3xl font-semibold tracking-tight num">{heading}</h1>
    {#if r.room.name}
      <p class="mt-1 text-subtext">{r.room.name}</p>
    {/if}
  </div>

  {#if !dialogOpen}
    {@render errorLine()}
  {/if}

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
      <p class="flex items-center gap-2">
        <span>
          {m.limits_source()}:
          {data.thresholds.source === 'room' ? m.limits_room() : m.limits_global()}
        </span>
        {#if data.canManage}
          <Button
            variant="accent-soft"
            class="px-2.5 py-0.5 text-xs"
            onclick={() => a.open(() => (editingLimits = true))}
          >
            {m.edit_limits()}
          </Button>
        {/if}
      </p>
    {/if}
  </div>

  <section aria-labelledby="room-info-heading" class="mt-8">
    <div class="flex items-center gap-3">
      <h2 class="text-lg font-semibold" id="room-info-heading">{m.room_info()}</h2>
      <span aria-hidden="true" class="h-px flex-1 bg-surface-0"></span>
    </div>

    <div class="mt-3 grid gap-4 sm:grid-cols-2">
      <div
        class="flex items-center justify-between gap-3 rounded-2xl bg-canvas p-4 ring-1 ring-surface-0"
      >
        <div class="flex min-w-0 items-center gap-3">
          <User aria-hidden="true" class="size-5 shrink-0 text-subtext" />
          <div class="min-w-0">
            <p class="text-xs font-medium text-subtext">{m.occupant()}</p>
            {#if data.stay}
              <p class="truncate font-medium">{data.stay.user_display_name}</p>
              <p class="text-xs text-subtext num">
                {m.checked_in_at()}
                {fmtDateTime(data.stay.checked_in_at)}
              </p>
            {:else}
              <p class="text-subtext">{m.room_empty()}</p>
            {/if}
          </div>
        </div>
        <Button
          variant="accent-soft"
          class="shrink-0 px-3 py-1 text-xs"
          onclick={() => a.open(() => (managingStay = true))}
        >
          {m.manage()}
        </Button>
      </div>

      <div class="flex items-center gap-3 rounded-2xl bg-canvas p-4 ring-1 ring-surface-0">
        {#if offline}
          <WifiOff class="size-5 shrink-0 text-overlay" aria-hidden="true" />
        {:else}
          <Wifi class="size-5 shrink-0 text-ok" aria-hidden="true" />
        {/if}
        <div class="min-w-0">
          <p class="text-xs font-medium text-subtext">{m.device()}</p>
          <p class={offline ? 'text-subtext' : 'font-medium'}>
            {#if !r.device}
              {m.device_none()}
            {:else if offline}
              {m.device_offline()}
            {:else}
              {m.device_online()}
            {/if}
          </p>
          {#if r.device?.last_seen_at}
            <p class="text-xs text-subtext num">
              {m.last_seen()}
              {fmtDateTime(r.device.last_seen_at)}
            </p>
          {/if}
        </div>
      </div>
    </div>
  </section>

  <div class="mt-8 grid gap-8 xl:grid-cols-2">
    <AlarmList alarms={data.alarms} />
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
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (managingStay = false))}>
      {m.close()}
    </Button>
    {#if !data.stay && r.room.is_active}
      <Button type="submit" variant="ok-soft" form="stay-checkin">{m.check_in()}</Button>
    {/if}
  {/snippet}
</Dialog>

<Dialog
  open={editingLimits}
  onclose={() => a.close(() => (editingLimits = false))}
  title={m.edit_limits()}
>
  {#if data.thresholds}
    <form
      id="room-limits"
      method="POST"
      action="?/setThresholds"
      use:enhance={a.track(() => (editingLimits = false))}
    >
      <p class="text-subtext">
        {data.thresholds.source === 'room'
          ? m.limits_edit_room_help()
          : m.limits_edit_global_help()}
      </p>
      <div class="mt-3">
        <ThresholdForm values={data.thresholds} />
      </div>
    </form>
    {#if data.thresholds.source === 'room'}
      <form
        id="room-limits-clear"
        method="POST"
        action="?/clearThresholds"
        use:enhance={a.track(() => (editingLimits = false))}
      ></form>
    {/if}
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editingLimits = false))}>
      {m.cancel()}
    </Button>
    {#if data.thresholds?.source === 'room'}
      <Button type="submit" variant="warn-soft" form="room-limits-clear">
        {m.threshold_delete()}
      </Button>
    {/if}
    <Button type="submit" form="room-limits">{m.save()}</Button>
  {/snippet}
</Dialog>
