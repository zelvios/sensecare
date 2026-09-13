<script lang="ts">
  import { enhance } from '$app/forms';
  import { resolve } from '$app/paths';
  import { Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import RoomPicker from '$lib/components/rooms/RoomPicker.svelte';
  import { fmtDateTime } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import PatientForm from './PatientForm.svelte';
  import type { User } from '$lib/api/types';

  let { data } = $props();
  const a = actionState();

  let creating = $state(false);
  let editing = $state<User | null>(null);
  let changingPassword = $state<User | null>(null);
  let placing = $state<User | null>(null);
  const dialogOpen = $derived(
    creating || editing !== null || changingPassword !== null || placing !== null
  );

  const stayOf = (u: User) => data.stayByUser[u.id] ?? null;

  let query = $state('');
  let statusFilter = $state<'all' | 'active' | 'inactive'>('all');
  let stayFilter = $state<'all' | 'in' | 'out'>('all');

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return data.patients.filter((u) => {
      if (statusFilter === 'active' && !u.is_active) return false;
      if (statusFilter === 'inactive' && u.is_active) return false;
      if (stayFilter === 'in' && !stayOf(u)) return false;
      if (stayFilter === 'out' && stayOf(u)) return false;
      if (!q) return true;
      return (
        u.username.toLowerCase().includes(q) ||
        u.display_name.toLowerCase().includes(q) ||
        (stayOf(u)?.room_number ?? '').toLowerCase().includes(q)
      );
    });
  });
  const filtering = $derived(query.trim() !== '' || statusFilter !== 'all' || stayFilter !== 'all');
  const checkedInCount = $derived(data.patients.filter((u) => stayOf(u)).length);

  type SortKey = 'display_name' | 'username' | 'room' | 'status' | 'last_login_at';
  let sortKey = $state<SortKey>('display_name');
  let sortDir = $state<'asc' | 'desc'>('asc');

  function toggleSort(key: SortKey) {
    if (sortKey === key) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    else {
      sortKey = key;
      sortDir = key === 'last_login_at' ? 'desc' : 'asc';
    }
  }

  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
  const compare: Record<SortKey, (x: User, y: User) => number> = {
    display_name: (x, y) => collator.compare(x.display_name, y.display_name),
    username: (x, y) => collator.compare(x.username, y.username),
    room: (x, y) => collator.compare(stayOf(x)?.room_number ?? '~', stayOf(y)?.room_number ?? '~'),
    status: (x, y) =>
      Number(y.is_active) - Number(x.is_active) || collator.compare(x.display_name, y.display_name),
    last_login_at: (x, y) => (x.last_login_at ?? '').localeCompare(y.last_login_at ?? '')
  };

  const sorted = $derived.by(() => {
    const list = [...visible].sort(compare[sortKey]);
    return sortDir === 'asc' ? list : list.reverse();
  });

  const errorText = $derived.by(() => {
    if (!a.error) return null;
    if (a.error === 'already_exists') return m.username_taken();
    if (a.error === 'conflict') return m.stay_conflict();
    if (a.error === 'bad_request') return m.user_invalid();
    if (a.error === 'forbidden') return m.user_forbidden();
    return m.action_failed();
  });
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
  <title>{m.title_patients()}</title>
</svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
  <div class="flex items-baseline gap-3">
    <h1 class="text-3xl font-semibold tracking-tight">{m.patients()}</h1>
    <span class="text-sm text-subtext num" title={m.patients_checked_in_of_total()}>
      {checkedInCount} / {data.patients.length}
    </span>
  </div>
  <Button onclick={() => a.open(() => (creating = true))}>{m.patient_create()}</Button>
</div>

<div class="mt-6 flex flex-wrap items-center gap-2">
  <label class="relative min-w-0 flex-1 sm:w-64 sm:flex-none">
    <Search
      aria-hidden="true"
      class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
    />
    <span class="sr-only">{m.search_patients()}</span>
    <input
      type="search"
      bind:value={query}
      placeholder={m.search_patients()}
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
    <span class="sr-only">{m.room_heading()}</span>
    <select
      bind:value={stayFilter}
      class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="all">{m.all()}</option>
      <option value="in">{m.checked_in()}</option>
      <option value="out">{m.not_checked_in_short()}</option>
    </select>
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

  {#if filtering}
    <span class="ml-auto text-sm text-subtext num">{visible.length} / {data.patients.length}</span>
  {/if}
</div>

{#if !dialogOpen}
  {@render errorLine()}
{/if}

<div class="mt-4 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
  <table class="w-full text-sm">
    <thead class="bg-crust text-left text-subtext sm:sticky sm:top-14 sm:z-10">
      <tr>
        {@render sortHeader('display_name', m.display_name())}
        {@render sortHeader('username', m.login_username())}
        {@render sortHeader('room', m.room_heading())}
        {@render sortHeader('status', m.status())}
        {@render sortHeader('last_login_at', m.last_login())}
        <th class="px-4 py-2"></th>
      </tr>
    </thead>
    <tbody class="divide-y">
      {#each sorted as u (u.id)}
        {@const stay = stayOf(u)}
        <tr class={u.is_active ? '' : 'text-subtext'}>
          <td class="w-full max-w-0 truncate px-4 py-2 font-medium" title={u.display_name}>
            {u.display_name}
          </td>
          <td class="px-4 py-2 whitespace-nowrap text-subtext">{u.username}</td>
          <td class="px-4 py-2 whitespace-nowrap num">
            {#if stay}
              <a
                href={resolve('/(app)/staff/rooms/[id]', { id: stay.room_id })}
                class="hover:text-accent"
              >
                {stay.room_number}
              </a>
            {:else}
              <span class="text-subtext">{m.not_checked_in_short()}</span>
            {/if}
          </td>
          <td class="px-4 py-2 whitespace-nowrap">
            <span
              class="rounded-full px-2 py-0.5 text-xs font-medium {u.is_active
                ? 'bg-ok/10 text-ok'
                : 'bg-warn/10 text-warn'}"
            >
              {u.is_active ? m.active() : m.inactive()}
            </span>
          </td>
          <td class="px-4 py-2 text-subtext num">
            {u.last_login_at ? fmtDateTime(u.last_login_at) : m.never()}
          </td>
          <td class="px-4 py-2 whitespace-nowrap">
            <div class="flex justify-end gap-2">
              <Button
                variant="accent-soft"
                class="px-3 py-1 text-xs"
                onclick={() => a.open(() => (editing = u))}
              >
                {m.edit()}
              </Button>
              <Button
                variant="secondary"
                class="px-3 py-1 text-xs"
                onclick={() => a.open(() => (changingPassword = u))}
              >
                {m.password_change()}
              </Button>
              {#if stay}
                <form method="POST" action="?/checkOut" use:enhance={a.track()}>
                  <input type="hidden" name="stay_id" value={stay.id} />
                  <Button type="submit" variant="warn-soft" class="px-3 py-1 text-xs"
                    >{m.check_out()}</Button
                  >
                </form>
              {:else if u.is_active}
                <Button
                  variant="ok-soft"
                  class="px-3 py-1 text-xs"
                  onclick={() => a.open(() => (placing = u))}
                >
                  {m.check_in()}
                </Button>
              {/if}
              {#if u.is_active}
                <form method="POST" action="?/deactivate" use:enhance={a.track()}>
                  <input type="hidden" name="id" value={u.id} />
                  <Button
                    type="submit"
                    variant="warn-soft"
                    class="px-3 py-1 text-xs"
                    disabled={stay !== null}
                  >
                    {m.deactivate()}
                  </Button>
                </form>
              {:else}
                <form method="POST" action="?/activate" use:enhance={a.track()}>
                  <input type="hidden" name="id" value={u.id} />
                  <Button type="submit" variant="ok-soft" class="px-3 py-1 text-xs"
                    >{m.activate()}</Button
                  >
                </form>
              {/if}
            </div>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

{#if data.patients.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.patients_none()}</p>
{:else if visible.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.search_no_match_users()}</p>
{/if}

<Dialog
  open={creating}
  onclose={() => a.close(() => (creating = false))}
  title={m.patient_create()}
>
  <form
    id="patient-create"
    method="POST"
    action="?/create"
    use:enhance={a.track(() => (creating = false))}
  >
    <PatientForm />
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (creating = false))}
      >{m.cancel()}</Button
    >
    <Button type="submit" form="patient-create">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={editing !== null}
  onclose={() => a.close(() => (editing = null))}
  title={m.patient_edit()}
>
  {#if editing}
    <form
      id="patient-edit"
      method="POST"
      action="?/update"
      use:enhance={a.track(() => (editing = null))}
    >
      <PatientForm patient={editing} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editing = null))}>{m.cancel()}</Button
    >
    <Button type="submit" form="patient-edit">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={changingPassword !== null}
  onclose={() => a.close(() => (changingPassword = null))}
  title={m.password_change()}
>
  {#if changingPassword}
    <form
      id="patient-password"
      method="POST"
      action="?/password"
      use:enhance={a.track(() => (changingPassword = null))}
    >
      <input type="hidden" name="id" value={changingPassword.id} />
      <p class="font-medium">{changingPassword.display_name}</p>
      <p class="mt-1 text-subtext">{m.password_change_help()}</p>
      <div class="mt-3">
        <Input
          label={m.password_new()}
          name="password"
          type="password"
          required
          minlength={10}
          maxlength={128}
          autocomplete="new-password"
        />
      </div>
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (changingPassword = null))}
      >{m.cancel()}</Button
    >
    <Button type="submit" form="patient-password">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={placing !== null}
  onclose={() => a.close(() => (placing = null))}
  title={m.check_in()}
>
  {#if placing}
    <form
      id="patient-checkin"
      method="POST"
      action="?/checkIn"
      use:enhance={a.track(() => (placing = null))}
    >
      <input type="hidden" name="user_id" value={placing.id} />
      <p class="font-medium">{placing.display_name}</p>
      <p class="mt-1 text-subtext">{m.check_in_help()}</p>
      <div class="mt-3">
        <RoomPicker rooms={data.freeRooms} />
      </div>
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (placing = null))}>{m.cancel()}</Button
    >
    <Button type="submit" variant="ok-soft" form="patient-checkin">{m.check_in()}</Button>
  {/snippet}
</Dialog>
