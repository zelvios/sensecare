<script lang="ts">
  import { enhance } from '$app/forms';
  import { resolve } from '$app/paths';
  import { ArrowLeft } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import AuditList from '$lib/components/audit/AuditList.svelte';
  import { fmtDateTime } from '$lib/utils/format';
  import { actionState } from '$lib/utils/forms.svelte';
  import UserForm from '../UserForm.svelte';
  import type { Role } from '$lib/api/types';

  let { data } = $props();
  const a = actionState();

  let editing = $state(false);
  let changingPassword = $state(false);
  const isSelf = $derived(data.account.id === data.user.id);

  const roleLabel: Record<Role, () => string> = {
    client: m.role_client,
    staff: m.role_staff,
    admin: m.role_admin
  };

  const errorText = $derived.by(() => {
    if (!a.error) return null;
    if (a.error === 'already_exists') return m.username_taken();
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

<svelte:head>
  <title>{data.account.display_name} - SenseCare</title>
</svelte:head>

<div class="min-h-0 flex-1 overflow-y-auto p-1">
  <a
    class="inline-flex items-center gap-1 text-sm text-subtext hover:text-text"
    href={resolve('/admin/accounts')}
  >
    <ArrowLeft aria-hidden="true" class="size-4" />
    {m.admin_accounts()}
  </a>

  <div class="mt-3 flex flex-wrap items-start justify-between gap-4">
    <div>
      <h1 class="text-3xl font-semibold tracking-tight">{data.account.display_name}</h1>
      <p class="mt-1 text-subtext">
        {data.account.username}
        <span class="mx-1">-</span>
        {roleLabel[data.account.role]()}
        <span
          class="ml-2 rounded-full px-2 py-0.5 text-xs font-medium {data.account.is_active
            ? 'bg-ok/10 text-ok'
            : 'bg-warn/10 text-warn'}"
        >
          {data.account.is_active ? m.active() : m.inactive()}
        </span>
      </p>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button variant="accent-soft" onclick={() => a.open(() => (editing = true))}>
        {m.edit()}
      </Button>
      <Button variant="secondary" onclick={() => a.open(() => (changingPassword = true))}>
        {m.password_change()}
      </Button>
      {#if data.account.is_active}
        <form method="POST" action="?/deactivate" use:enhance={a.track()}>
          <Button type="submit" variant="warn-soft" disabled={isSelf}>{m.deactivate()}</Button>
        </form>
      {:else}
        <form method="POST" action="?/activate" use:enhance={a.track()}>
          <Button type="submit" variant="ok-soft">{m.activate()}</Button>
        </form>
      {/if}
    </div>
  </div>

  {#if !editing && !changingPassword}
    {@render errorLine()}
  {/if}

  <dl
    class="mt-6 grid gap-x-8 gap-y-3 rounded-2xl bg-canvas p-5 ring-1 ring-surface-0 sm:grid-cols-2"
  >
    <div>
      <dt class="text-xs font-bold text-subtext">{m.created()}</dt>
      <dd class="num">{fmtDateTime(data.account.created_at)}</dd>
    </div>
    <div>
      <dt class="text-xs font-bold text-subtext">{m.last_login()}</dt>
      <dd class="num">
        {data.account.last_login_at ? fmtDateTime(data.account.last_login_at) : m.never()}
      </dd>
    </div>
  </dl>

  <section aria-labelledby="about-heading" class="mt-8">
    <div class="flex items-center gap-3">
      <h2 class="text-lg font-semibold" id="about-heading">{m.audit_about()}</h2>
      <span class="h-px flex-1 bg-surface-0" aria-hidden="true"></span>
      <span class="text-sm text-subtext num">{data.about.length}</span>
    </div>
    <div class="mt-3">
      <AuditList entries={[...data.about].reverse()} empty={m.audit_none()} />
    </div>
  </section>

  <section aria-labelledby="by-heading" class="mt-8">
    <div class="flex items-center gap-3">
      <h2 class="text-lg font-semibold" id="by-heading">{m.audit_by()}</h2>
      <span class="h-px flex-1 bg-surface-0" aria-hidden="true"></span>
      <span class="text-sm text-subtext num">{data.by.length}</span>
    </div>
    <div class="mt-3">
      <AuditList entries={data.by} empty={m.audit_none()} />
    </div>
  </section>
</div>

<Dialog open={editing} onclose={() => a.close(() => (editing = false))} title={m.user_edit()}>
  <form
    id="user-edit"
    method="POST"
    action="?/update"
    use:enhance={a.track(() => (editing = false))}
  >
    <UserForm user={data.account} self={isSelf} />
  </form>
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (editing = false))}
      >{m.cancel()}</Button
    >
    <Button type="submit" form="user-edit">{m.save()}</Button>
  {/snippet}
</Dialog>

<Dialog
  open={changingPassword}
  onclose={() => a.close(() => (changingPassword = false))}
  title={m.password_change()}
>
  <form
    id="user-password"
    method="POST"
    action="?/password"
    use:enhance={a.track(() => (changingPassword = false))}
  >
    <p class="text-subtext">{m.password_change_help()}</p>
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
  {@render errorLine()}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => a.close(() => (changingPassword = false))}>
      {m.cancel()}
    </Button>
    <Button type="submit" form="user-password">{m.save()}</Button>
  {/snippet}
</Dialog>
