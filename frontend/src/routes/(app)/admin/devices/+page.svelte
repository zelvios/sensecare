<script lang="ts">
  import { enhance } from '$app/forms';
  import { resolve } from '$app/paths';
  import { ArrowLeft, Check, Copy, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import RoomPicker from '$lib/components/rooms/RoomPicker.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { fmtDateTime } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import DeviceForm from './DeviceForm.svelte';
  import { page } from '$app/state';
  import type { Device, DeviceWithKey, Room } from '$lib/api/types';

  let { data } = $props();
  const a = actionState();
  let statusFilter = $state<'all' | 'active' | 'inactive'>('all');
  let assignedFilter = $state<'all' | 'assigned' | 'unassigned'>('all');
  let query = $state(page.url.searchParams.get('q') ?? '');

  let registering = $state(false);
  let editing = $state<Device | null>(null);
  let assigning = $state<Device | null>(null);
  let rotating = $state<Device | null>(null);
  let simulating = $state<Device | null>(null);
  let deleting = $state<Device | null>(null);
  let revealed = $state<DeviceWithKey | null>(null);
  let copiedId = $state<string | null>(null);
  let copied = $state(false);
  const dialogOpen = $derived(
    registering ||
      editing !== null ||
      assigning !== null ||
      rotating !== null ||
      simulating !== null ||
      deleting !== null ||
      revealed !== null
  );

  const roomById = $derived(new Map(data.rooms.map((r) => [r.id, r])));
  const roomOf = (d: Device) => (d.room_id ? roomById.get(d.room_id) : undefined);
  const roomLabel = (r: Room | undefined) =>
    r ? `${r.room_number}${r.name ? ` - ${r.name}` : ''}` : '';
  const roomsFree = $derived(
    data.rooms.filter(
      (r) => r.is_active && !data.devices.some((d) => d.is_active && d.room_id === r.id)
    )
  );

  const offline = (d: Device) =>
    !d.last_seen_at || Date.now() - Date.parse(d.last_seen_at) > 2 * 60_000;

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return data.devices.filter((d) => {
      if (statusFilter === 'active' && !d.is_active) return false;
      if (statusFilter === 'inactive' && d.is_active) return false;
      if (assignedFilter === 'assigned' && !d.room_id) return false;
      if (assignedFilter === 'unassigned' && d.room_id) return false;
      if (!q) return true;
      return (
        d.id.toLowerCase().startsWith(q) ||
        (d.label ?? '').toLowerCase().includes(q) ||
        (d.firmware_version ?? '').toLowerCase().includes(q) ||
        roomLabel(roomOf(d)).toLowerCase().includes(q)
      );
    });
  });
  const filtering = $derived(
    query.trim() !== '' || statusFilter !== 'all' || assignedFilter !== 'all'
  );
  const activeCount = $derived(data.devices.filter((d) => d.is_active).length);

  type SortKey = 'label' | 'room' | 'firmware' | 'status' | 'last_seen_at';
  let sortKey = $state<SortKey>('room');
  let sortDir = $state<'asc' | 'desc'>('asc');

  function toggleSort(key: SortKey) {
    if (sortKey === key) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    else {
      sortKey = key;
      sortDir = key === 'last_seen_at' ? 'desc' : 'asc';
    }
  }

  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
  const compare: Record<SortKey, (x: Device, y: Device) => number> = {
    label: (x, y) => collator.compare(x.label ?? '', y.label ?? ''),
    room: (x, y) => collator.compare(roomLabel(roomOf(x)) || '~', roomLabel(roomOf(y)) || '~'),
    firmware: (x, y) => collator.compare(x.firmware_version ?? '', y.firmware_version ?? ''),
    status: (x, y) =>
      Number(y.is_active) - Number(x.is_active) || Number(offline(x)) - Number(offline(y)),
    last_seen_at: (x, y) => (x.last_seen_at ?? '').localeCompare(y.last_seen_at ?? '')
  };

  const sorted = $derived.by(() => {
    const list = [...visible].sort(compare[sortKey]);
    return sortDir === 'asc' ? list : list.reverse();
  });

  const errorText = $derived.by(() => {
    if (!a.error) return null;
    if (a.error === 'unauthorized') return m.device_auth_failed();
    if (a.error === 'conflict') return simulating ? m.device_unassigned() : m.device_room_taken();
    if (a.error === 'invalid_reference') return m.device_in_use();
    if (a.error === 'bad_request') return simulating ? m.reading_invalid() : m.device_invalid();
    return m.action_failed();
  });

  function reveal(result: unknown) {
    registering = false;
    rotating = null;
    copied = false;
    revealed = result as DeviceWithKey;
  }

  async function copyKey() {
    if (!revealed) return;
    await navigator.clipboard.writeText(revealed.key);
    copied = true;
  }

  async function copyId(id: string) {
    await navigator.clipboard.writeText(id);
    copiedId = id;
    setTimeout(() => {
      if (copiedId === id) copiedId = null;
    }, 1500);
  }
</script>

{#snippet errorLine()}
  {#if errorText}
    <p class="mt-4 rounded-lg bg-danger/10 px-3 py-2 text-sm text-danger" role="alert">
      {errorText}
    </p>
  {/if}
{/snippet}

{#snippet sortHeader(key: SortKey, label: string)}
  {@const active = sortKey === key}
  <th
    class="px-4 py-2 font-medium"
    aria-sort={active ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'}
  >
    <button
      type="button"
      onclick={() => toggleSort(key)}
      class="inline-flex items-center gap-1 rounded hover:text-text {active ? 'text-text' : ''}"
    >
      {label}
      <span aria-hidden="true" class="text-xs {active ? '' : 'invisible'}">
        {sortDir === 'asc' ? '▲' : '▼'}
      </span>
    </button>
  </th>
{/snippet}

<svelte:head>
  <title>{m.title_admin_devices()}</title>
</svelte:head>

<a
  class="inline-flex items-center gap-1 text-sm text-subtext hover:text-text"
  href={resolve('/admin')}
>
  <ArrowLeft aria-hidden="true" class="size-4" />
  {m.nav_admin()}
</a>

<div class="mt-3 flex flex-wrap items-center justify-between gap-4">
  <div class="flex items-baseline gap-3">
    <h1 class="text-3xl font-semibold tracking-tight">{m.admin_devices()}</h1>
    <span class="text-sm text-subtext num" title={m.devices_active_of_total()}>
      {activeCount} / {data.devices.length}
    </span>
  </div>
  <Button onclick={() => a.open(() => (registering = true))}>{m.device_register()}</Button>
</div>

<div class="mt-6 flex flex-wrap items-center gap-2">
  <label class="relative min-w-0 flex-1 sm:w-64 sm:flex-none">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_devices()}</span>
    <input
      type="search"
      bind:value={query}
      placeholder={m.search_devices()}
      class="w-full rounded-md border bg-canvas py-1 pr-7 pl-8 text-sm placeholder:text-overlay focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    />
    {#if query}
      <button
        type="button"
        onclick={() => (query = '')}
        aria-label={m.clear()}
        class="absolute top-1/2 right-1 -translate-y-1/2 rounded p-1 text-overlay hover:text-text"
      >
        <X class="size-3.5" aria-hidden="true" />
      </button>
    {/if}
  </label>

  <label class="shrink-0">
    <span class="sr-only">{m.status()}</span>
    <select
      bind:value={statusFilter}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="all">{m.all()}</option>
      <option value="active">{m.active()}</option>
      <option value="inactive">{m.inactive()}</option>
    </select>
  </label>

  <label class="shrink-0">
    <span class="sr-only">{m.room_heading()}</span>
    <select
      bind:value={assignedFilter}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="all">{m.all_rooms()}</option>
      <option value="assigned">{m.device_assigned()}</option>
      <option value="unassigned">{m.device_unassigned_short()}</option>
    </select>
  </label>

  {#if filtering}
    <span class="ml-auto text-sm text-subtext num">{visible.length} / {data.devices.length}</span>
  {/if}
</div>

{#if !dialogOpen}
  {@render errorLine()}
{/if}

<div class="mt-4 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
  <table class="w-full text-sm">
    <thead class="bg-crust text-left text-subtext sm:sticky sm:top-14 sm:z-10">
      <tr>
        {@render sortHeader('label', m.device_label())}
        {@render sortHeader('room', m.room_heading())}
        {@render sortHeader('firmware', m.device_firmware())}
        {@render sortHeader('status', m.status())}
        {@render sortHeader('last_seen_at', m.last_seen())}
        <th class="px-4 py-2"></th>
      </tr>
    </thead>
    <tbody class="divide-y">
      {#each sorted as d (d.id)}
        {@const room = roomOf(d)}
        <tr class={d.is_active ? '' : 'text-subtext'}>
          <td class="w-full max-w-0 px-4 py-2 font-medium">
            <div class="flex items-center gap-1.5">
              <span class="truncate">{d.label || d.id.slice(0, 8)}</span>
              <span class="shrink-0 text-xs font-normal text-overlay num">{d.id.slice(0, 8)}</span>
              <button
                type="button"
                onclick={() => copyId(d.id)}
                aria-label={m.copy_id()}
                title={d.id}
                class="shrink-0 rounded p-1 text-overlay hover:bg-surface-0/60 hover:text-text"
              >
                {#if copiedId === d.id}
                  <Check class="size-3.5 text-ok" aria-hidden="true" />
                {:else}
                  <Copy class="size-3.5" aria-hidden="true" />
                {/if}
              </button>
            </div>
          </td>
          <td class="px-4 py-2 whitespace-nowrap num">
            {#if room}
              <a
                href={resolve('/(app)/staff/rooms/[id]', { id: room.id })}
                class="hover:text-accent">{roomLabel(room)}</a
              >
            {:else}
              <span class="text-subtext">{m.device_unassigned_short()}</span>
            {/if}
          </td>
          <td class="px-4 py-2 whitespace-nowrap text-subtext num">{d.firmware_version ?? ''}</td>
          <td class="px-4 py-2 whitespace-nowrap">
            {#if !d.is_active}
              <span class="rounded-full bg-warn/10 px-2 py-0.5 text-xs font-medium text-warn"
                >{m.inactive()}</span
              >
            {:else if offline(d)}
              <span class="rounded-full bg-surface-0 px-2 py-0.5 text-xs font-medium text-subtext"
                >{m.offline()}</span
              >
            {:else}
              <span class="rounded-full bg-ok/10 px-2 py-0.5 text-xs font-medium text-ok"
                >{m.online()}</span
              >
            {/if}
          </td>
          <td class="px-4 py-2 text-subtext num"
            >{d.last_seen_at ? fmtDateTime(d.last_seen_at) : m.never()}</td
          >
          <td class="px-4 py-2 whitespace-nowrap">
            <div class="flex justify-end gap-2">
              <Button
                variant="accent-soft"
                class="px-3 py-1 text-xs"
                onclick={() => a.open(() => (editing = d))}>{m.edit()}</Button
              >
              {#if d.is_active}
                <Button
                  variant="secondary"
                  class="px-3 py-1 text-xs"
                  onclick={() => a.open(() => (assigning = d))}>{m.device_assign()}</Button
                >
                <Button
                  variant="secondary"
                  class="px-3 py-1 text-xs"
                  onclick={() => a.open(() => (rotating = d))}>{m.device_rotate()}</Button
                >
                <Button
                  variant="secondary"
                  class="px-3 py-1 text-xs"
                  onclick={() => a.open(() => (simulating = d))}
                >
                  {m.simulate_reading()}
                </Button>
                <form method="POST" action="?/deactivate" use:enhance={a.track()}>
                  <input type="hidden" name="id" value={d.id} />
                  <Button type="submit" variant="warn-soft" class="px-3 py-1 text-xs"
                    >{m.deactivate()}</Button
                  >
                </form>
              {:else}
                <form method="POST" action="?/activate" use:enhance={a.track()}>
                  <input type="hidden" name="id" value={d.id} />
                  <Button type="submit" variant="ok-soft" class="px-3 py-1 text-xs"
                    >{m.activate()}</Button
                  >
                </form>
                <Button
                  variant="danger-soft"
                  class="px-3 py-1 text-xs"
                  onclick={() => a.open(() => (deleting = d))}>{m.delete()}</Button
                >
              {/if}
            </div>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

{#if data.devices.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.devices_none()}</p>
{:else if visible.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.search_no_match_devices()}</p>
{/if}

<Dialog
  open={registering}
  onclose={() => a.close(() => (registering = false))}
  title={m.device_register()}
>
  <form id="device-register" method="POST" action="?/register" use:enhance={a.track(reveal)}>
    <p class="text-subtext">{m.device_register_help()}</p>
    <div class="mt-3">
      <DeviceForm />
    </div>
    <div class="mt-4">
      <span class="mb-1 block text-xs font-medium">{m.device_assign_optional()}</span>
      <RoomPicker rooms={roomsFree} />
    </div>
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (registering = false))}
      >{m.cancel()}</Button
    >
    <Button type="submit" form="device-register">{m.device_register()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={editing !== null}
  onclose={() => a.close(() => (editing = null))}
  title={m.device_edit()}
>
  {#if editing}
    <form
      id="device-edit"
      method="POST"
      action="?/update"
      use:enhance={a.track(() => (editing = null))}
    >
      <DeviceForm device={editing} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editing = null))}>{m.cancel()}</Button
    >
    <Button type="submit" form="device-edit">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={assigning !== null}
  onclose={() => a.close(() => (assigning = null))}
  title={m.device_assign()}
>
  {#if assigning}
    {@const current = roomOf(assigning)}
    <form
      id="device-assign"
      method="POST"
      action="?/assign"
      use:enhance={a.track(() => (assigning = null))}
    >
      <input type="hidden" name="id" value={assigning.id} />
      {#if current}
        <p class="text-subtext">
          {m.device_assign_current()}
          <span class="font-medium text-text num">{roomLabel(current)}</span>
        </p>
      {/if}
      <div class="mt-3">
        <RoomPicker rooms={roomsFree} />
      </div>
    </form>
    {#if current}
      <form
        id="device-unassign"
        method="POST"
        action="?/assign"
        use:enhance={a.track(() => (assigning = null))}
      >
        <input type="hidden" name="id" value={assigning.id} />
        <input type="hidden" name="room_id" value="" />
      </form>
    {/if}
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (assigning = null))}
      >{m.cancel()}</Button
    >
    {#if assigning && roomOf(assigning)}
      <Button type="submit" variant="warn-soft" form="device-unassign">{m.device_unassign()}</Button
      >
    {/if}
    <Button type="submit" form="device-assign">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={rotating !== null}
  onclose={() => a.close(() => (rotating = null))}
  title={m.device_rotate()}
>
  {#if rotating}
    <p class="text-subtext">{m.device_rotate_help()}</p>
    <p class="mt-2 font-medium">{rotating.label || rotating.id.slice(0, 8)}</p>
    <form id="device-rotate" method="POST" action="?/rotateKey" use:enhance={a.track(reveal)}>
      <input type="hidden" name="id" value={rotating.id} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (rotating = null))}
      >{m.cancel()}</Button
    >
    <Button type="submit" variant="danger" form="device-rotate">{m.device_rotate()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={simulating !== null}
  onclose={() => a.close(() => (simulating = null))}
  title={m.simulate_reading()}
>
  {#if simulating}
    <form
      id="device-simulate"
      method="POST"
      action="?/simulateReading"
      use:enhance={a.track(() => (simulating = null))}
    >
      <input type="hidden" name="device_id" value={simulating.id} />
      <p class="text-subtext">{m.simulate_reading_help()}</p>
      <p class="mt-2 font-medium">{simulating.label || simulating.id.slice(0, 8)}</p>
      <div class="mt-3 space-y-4">
        <Input
          label={m.device_key()}
          name="device_key"
          type="password"
          required
          autocomplete="off"
        />
        <div class="grid gap-4 sm:grid-cols-2">
          <Input
            label={m.temperature()}
            name="temperature_c"
            type="number"
            step="0.1"
            min={-40}
            max={85}
            required
            value="21.5"
          />
          <Input
            label={m.humidity()}
            name="humidity_pct"
            type="number"
            step="0.1"
            min={0}
            max={100}
            required
            value="45"
          />
        </div>
      </div>
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (simulating = null))}>
      {m.cancel()}
    </Button>
    <Button type="submit" form="device-simulate">{m.send_reading()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={revealed !== null}
  onclose={() => a.close(() => (revealed = null))}
  title={m.device_key_title()}
>
  {#if revealed}
    <p class="text-subtext">{m.device_key_help()}</p>
    <dl class="mt-3 space-y-2 text-sm">
      <div>
        <dt class="text-xs font-medium text-subtext">{m.device_id()}</dt>
        <dd class="rounded-md bg-surface-0/60 px-2 py-1.5 break-all num">{revealed.id}</dd>
      </div>
      <div>
        <dt class="text-xs font-medium text-subtext">{m.device_key()}</dt>
        <dd class="flex items-center gap-2">
          <code class="min-w-0 flex-1 rounded-md bg-surface-0/60 px-2 py-1.5 break-all num"
            >{revealed.key}</code
          >
          <Button
            variant="secondary"
            class="shrink-0 px-2.5 py-1.5"
            onclick={copyKey}
            aria-label={m.copy()}
          >
            {#if copied}<Check class="size-4 text-ok" aria-hidden="true" />{:else}<Copy
                class="size-4"
                aria-hidden="true"
              />{/if}
          </Button>
        </dd>
      </div>
    </dl>
  {/if}
  {#snippet footer()}
    <Button onclick={() => a.close(() => (revealed = null))}>{m.device_key_done()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={deleting !== null}
  onclose={() => a.close(() => (deleting = null))}
  title={m.device_delete()}
>
  {#if deleting}
    <p>{m.device_delete_help()}</p>
    <p class="mt-2 font-medium">{deleting.label || deleting.id.slice(0, 8)}</p>
    <form
      id="device-delete"
      method="POST"
      action="?/delete"
      use:enhance={a.track(() => (deleting = null))}
    >
      <input type="hidden" name="id" value={deleting.id} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (deleting = null))}
      >{m.cancel()}</Button
    >
    <Button type="submit" variant="danger" form="device-delete">{m.delete()}</Button>
  {/snippet}
</Dialog>
