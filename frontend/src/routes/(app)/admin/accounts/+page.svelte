<script lang="ts">
  import { enhance } from '$app/forms';
  import { resolve } from '$app/paths';
  import { ArrowLeft, Search, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { fmtDateTime } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import UserForm from './UserForm.svelte';
  import type { Role, User } from '$lib/api/types';

  let { data } = $props();
  const a = actionState();

  let creating = $state(false);
  let editing = $state<User | null>(null);
  let deleting = $state<User | null>(null);
  const dialogOpen = $derived(creating || editing !== null || deleting !== null);

  let query = $state('');
  let roleFilter = $state<Role | 'all'>('all');

  const roleLabel: Record<Role, () => string> = {
    client: m.role_client,
    staff: m.role_staff,
    admin: m.role_admin
  };
  const roleRank: Record<Role, number> = { admin: 0, staff: 1, client: 2 };

  const visible = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return data.users.filter(
      (u) =>
        (roleFilter === 'all' || u.role === roleFilter) &&
        (!q || u.username.toLowerCase().includes(q) || u.display_name.toLowerCase().includes(q))
    );
  });
  const filtering = $derived(query.trim() !== '' || roleFilter !== 'all');
  const activeCount = $derived(data.users.filter((u) => u.is_active).length);

  type SortKey = 'username' | 'display_name' | 'role' | 'status' | 'last_login_at';
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
    username: (x, y) => collator.compare(x.username, y.username),
    display_name: (x, y) => collator.compare(x.display_name, y.display_name),
    role: (x, y) =>
      roleRank[x.role] - roleRank[y.role] || collator.compare(x.display_name, y.display_name),
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
    if (a.error === 'invalid_reference') return m.user_in_use();
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
  <title>{m.title_admin_accounts()}</title>
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
    <h1 class="text-3xl font-semibold tracking-tight">{m.admin_accounts()}</h1>
    <span class="text-sm text-subtext num" title={m.accounts_active_of_total()}>
      {activeCount} / {data.users.length}
    </span>
  </div>
  <Button onclick={() => a.open(() => (creating = true))}>{m.user_create()}</Button>
</div>

<div class="mt-6 flex flex-wrap items-center justify-between gap-3">
  <div class="flex w-full items-center gap-2 sm:w-auto">
    <label class="relative min-w-0 flex-1 sm:w-72 sm:flex-none">
      <Search
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-overlay"
      />
      <span class="sr-only">{m.search_users()}</span>
      <input
        type="search"
        bind:value={query}
        placeholder={m.search_users()}
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
      <span class="sr-only">{m.role()}</span>
      <select
        bind:value={roleFilter}
        class="rounded-md border bg-canvas py-1 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
      >
        <option value="all">{m.all_roles()}</option>
        <option value="admin">{m.role_admin()}</option>
        <option value="staff">{m.role_staff()}</option>
        <option value="client">{m.role_client()}</option>
      </select>
    </label>
  </div>
  {#if filtering}
    <span class="text-sm text-subtext num">{visible.length} / {data.users.length}</span>
  {/if}
</div>

{#if !dialogOpen}
  {@render errorLine()}
{/if}

<div class="mt-4 rounded-2xl bg-canvas ring-1 ring-surface-0 max-sm:overflow-x-auto">
  <table class="w-full text-sm">
    <thead class="sticky top-14 z-10 bg-crust text-left text-subtext">
      <tr>
        {@render sortHeader('display_name', m.display_name())}
        {@render sortHeader('username', m.login_username())}
        {@render sortHeader('role', m.role())}
        {@render sortHeader('status', m.status())}
        {@render sortHeader('last_login_at', m.last_login())}
        <th class="px-4 py-2"></th>
      </tr>
    </thead>
    <tbody class="divide-y">
      {#each sorted as u (u.id)}
        {@const self = u.id === data.user.id}
        <tr class={u.is_active ? '' : 'text-subtext'}>
          <td class="px-4 py-2 font-medium">
            <a href={resolve('/(app)/admin/accounts/[id]', { id: u.id })} class="hover:text-accent">
              {u.display_name}
            </a>
          </td>
          <td class="px-4 py-2 text-subtext">{u.username}</td>
          <td class="px-4 py-2">{roleLabel[u.role]()}</td>
          <td class="px-4 py-2">
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
          <td class="px-4 py-2">
            <div class="flex justify-end gap-2">
              <Button
                variant="accent-soft"
                class="px-3 py-1 text-xs"
                onclick={() => a.open(() => (editing = u))}
              >
                {m.edit()}
              </Button>
              {#if u.is_active}
                <form method="POST" action="?/deactivate" use:enhance={a.track()}>
                  <input type="hidden" name="id" value={u.id} />
                  <Button
                    type="submit"
                    variant="warn-soft"
                    class="px-3 py-1 text-xs"
                    disabled={self}
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
                <Button
                  variant="danger-soft"
                  class="px-3 py-1 text-xs"
                  onclick={() => a.open(() => (deleting = u))}
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

{#if data.users.length > 0 && visible.length === 0}
  <p class="mt-3 text-sm text-subtext">{m.search_no_match_users()}</p>
{/if}

<Dialog open={creating} onclose={() => a.close(() => (creating = false))} title={m.user_create()}>
  <form
    id="user-create"
    method="POST"
    action="?/create"
    use:enhance={a.track(() => (creating = false))}
  >
    <UserForm />
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (creating = false))}
      >{m.cancel()}</Button
    >
    <Button type="submit" form="user-create">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={editing !== null}
  onclose={() => a.close(() => (editing = null))}
  title={m.user_edit()}
>
  {#if editing}
    <form
      id="user-edit"
      method="POST"
      action="?/update"
      use:enhance={a.track(() => (editing = null))}
    >
      <UserForm user={editing} self={editing.id === data.user.id} />
    </form>
  {/if}
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editing = null))}>{m.cancel()}</Button
    >
    <Button type="submit" form="user-edit">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={deleting !== null}
  onclose={() => a.close(() => (deleting = null))}
  title={m.user_delete()}
>
  {#if deleting}
    <p>{m.user_delete_help()}</p>
    <p class="mt-2 font-medium">
      {deleting.display_name} <span class="text-subtext">({deleting.username})</span>
    </p>
    <form
      id="user-delete"
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
    <Button type="submit" variant="danger" form="user-delete">{m.delete()}</Button>
  {/snippet}
</Dialog>
