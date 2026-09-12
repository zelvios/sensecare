<script lang="ts">
  import { enhance } from '$app/forms';
  import { resolve } from '$app/paths';
  import { ArrowLeft } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import RoomFilterBar from '$lib/components/rooms/RoomFilterBar.svelte';
  import { filterRooms, type FloorFilter, floorsOf } from '$lib/utils/rooms';
  import { actionState } from '$lib/utils/forms.svelte';
  import RoomForm from './RoomForm.svelte';
  import type { Room } from '$lib/api/types';

  let { data } = $props();

  let creating = $state(false);
  let editing = $state<Room | null>(null);
  let deleting = $state<Room | null>(null);
  const dialogOpen = $derived(creating || editing !== null || deleting !== null);

  let query = $state('');
  let floor = $state<FloorFilter>('all');

  const floors = $derived(floorsOf(data.rooms));
  const visible = $derived(filterRooms(data.rooms, query, floor));
  const filtering = $derived(query.trim() !== '' || floor !== 'all');
  const activeCount = $derived(data.rooms.filter((r) => r.is_active).length);

  type SortKey = 'room_number' | 'name' | 'floor' | 'status';
  let sortKey = $state<SortKey>('floor');
  let sortDir = $state<'asc' | 'desc'>('asc');

  function toggleSort(key: SortKey) {
    if (sortKey === key) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    else {
      sortKey = key;
      sortDir = 'asc';
    }
  }

  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });

  const compare: Record<SortKey, (a: Room, b: Room) => number> = {
    room_number: (a, b) => collator.compare(a.room_number, b.room_number),
    name: (a, b) => collator.compare(a.name ?? '', b.name ?? ''),
    floor: (a, b) =>
      (a.floor ?? Number.POSITIVE_INFINITY) - (b.floor ?? Number.POSITIVE_INFINITY) ||
      collator.compare(a.room_number, b.room_number),
    status: (a, b) =>
      Number(b.is_active) - Number(a.is_active) || collator.compare(a.room_number, b.room_number)
  };

  const sorted = $derived.by(() => {
    const list = [...visible].sort(compare[sortKey]);
    return sortDir === 'asc' ? list : list.reverse();
  });

  const a = actionState();

  const errorText = $derived.by(() => {
    if (!a.error) return null;
    if (a.error === 'already_exists') return m.room_number_taken();
    if (a.error === 'invalid_reference') return m.room_in_use();
    if (a.error === 'bad_request') return m.room_invalid();
    return m.action_failed();
  });

  function floorLabel(r: Room) {
    return r.floor === null || r.floor === undefined ? '' : `${m.floor()} ${r.floor}`;
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
  <title>{m.title_admin_rooms()}</title>
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
    <h1 class="text-3xl font-semibold tracking-tight">{m.admin_rooms()}</h1>
    <span class="text-sm text-subtext num" title={m.rooms_active_of_total()}>
      {activeCount} / {data.rooms.length}
    </span>
  </div>
  <Button onclick={() => a.open(() => (creating = true))}>{m.room_create()}</Button>
</div>

<div class="mt-6 flex flex-wrap items-center justify-between gap-3">
  <RoomFilterBar bind:floor bind:query class="w-full sm:w-auto" {floors} />
  {#if filtering}
    <span class="text-sm text-subtext num">{visible.length} / {data.rooms.length}</span>
  {/if}
</div>

{#if !dialogOpen}
  {@render errorLine()}
{/if}

<div class="mt-4 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
  <table class="w-full text-sm">
    <thead class="sticky top-14 z-10 bg-crust text-left text-subtext">
      <tr>
        {@render sortHeader('room_number', m.room_number())}
        {@render sortHeader('name', m.room_name())}
        {@render sortHeader('floor', m.floor())}
        {@render sortHeader('status', m.status())}
        <th class="px-4 py-2"></th>
      </tr>
    </thead>
    <tbody class="divide-y">
      {#each sorted as r (r.id)}
        <tr class={r.is_active ? '' : 'text-subtext'}>
          <td class="px-4 py-2 font-medium num">{r.room_number}</td>
          <td class="px-4 py-2">{r.name ?? ''}</td>
          <td class="px-4 py-2 num">{floorLabel(r)}</td>
          <td class="px-4 py-2">
            <span
              class="rounded-full px-2 py-0.5 text-xs font-medium {r.is_active
                ? 'bg-ok/10 text-ok'
                : 'bg-warn/10 text-warn'}"
            >
              {r.is_active ? m.active() : m.inactive()}
            </span>
          </td>
          <td class="px-4 py-2">
            <div class="flex justify-end gap-2">
              <Button
                variant="accent-soft"
                class="px-3 py-1 text-xs"
                onclick={() => a.open(() => (editing = r))}
              >
                {m.edit()}
              </Button>
              {#if r.is_active}
                <form method="POST" action="?/deactivate" use:enhance={a.track()}>
                  <input type="hidden" name="id" value={r.id} />
                  <Button type="submit" variant="warn-soft" class="px-3 py-1 text-xs">
                    {m.deactivate()}
                  </Button>
                </form>
              {:else}
                <form method="POST" action="?/activate" use:enhance={a.track()}>
                  <input type="hidden" name="id" value={r.id} />
                  <Button type="submit" variant="ok-soft" class="px-3 py-1 text-xs">
                    {m.activate()}
                  </Button>
                </form>
                <Button
                  variant="danger-soft"
                  class="px-3 py-1 text-xs"
                  onclick={() => a.open(() => (deleting = r))}
                >
                  {m.delete()}
                </Button>
              {/if}
            </div>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

{#if data.rooms.length > 0 && visible.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.search_no_match()}</p>
{/if}

<Dialog open={creating} onclose={() => a.close(() => (creating = false))} title={m.room_create()}>
  <form
    action="?/create"
    id="room-create"
    method="POST"
    use:enhance={a.track(() => (creating = false))}
  >
    <RoomForm />
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (creating = false))}>
      {m.cancel()}
    </Button>
    <Button type="submit" form="room-create">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={editing !== null}
  onclose={() => a.close(() => (editing = null))}
  title={m.room_edit()}
>
  {#if editing}
    <form
      id="room-edit"
      method="POST"
      action="?/update"
      use:enhance={a.track(() => (editing = null))}
    >
      <RoomForm room={editing} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editing = null))}>{m.cancel()}</Button
    >
    <Button type="submit" form="room-edit">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={deleting !== null}
  onclose={() => a.close(() => (deleting = null))}
  title={m.room_delete()}
>
  {#if deleting}
    <p>{m.room_delete_help()}</p>
    <p class="mt-2 font-medium num">
      {deleting.room_number}{deleting.name ? ` - ${deleting.name}` : ''}
    </p>
    <form
      id="room-delete"
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
    <Button type="submit" variant="danger" form="room-delete">{m.delete()}</Button>
  {/snippet}
</Dialog>
