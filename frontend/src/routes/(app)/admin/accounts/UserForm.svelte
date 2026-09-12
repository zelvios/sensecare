<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import Input from '$lib/components/ui/Input.svelte';
  import type { User } from '$lib/api/types';

  let { user, self = false }: { user?: User | null; self?: boolean } = $props();

  const roles = [
    { value: 'client', label: m.role_client() },
    { value: 'staff', label: m.role_staff() },
    { value: 'admin', label: m.role_admin() }
  ];
</script>

{#if user}
  <input type="hidden" name="id" value={user.id} />
{/if}

<div class="space-y-4">
  <Input
    label={m.login_username()}
    name="username"
    value={user?.username ?? ''}
    required
    minlength={3}
    maxlength={64}
    autocapitalize="off"
    autocomplete="off"
    spellcheck="false"
  />
  <Input
    label={m.display_name()}
    name="display_name"
    value={user?.display_name ?? ''}
    required
    maxlength={128}
  />
  {#if !user}
    <Input
      label={m.login_password()}
      name="password"
      type="password"
      required
      minlength={10}
      maxlength={128}
      autocomplete="new-password"
    />
  {/if}
  <label class="block">
    <span class="mb-1 block text-xs font-medium">{m.role()}</span>
    <select
      name="role"
      disabled={self}
      class="w-full rounded-md border bg-canvas px-2 py-1.5 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none disabled:opacity-60"
    >
      {#each roles as r (r.value)}
        <option value={r.value} selected={(user?.role ?? 'client') === r.value}>{r.label}</option>
      {/each}
    </select>
    {#if self}
      <span class="mt-1 block text-xs text-subtext">{m.role_self_locked()}</span>
    {/if}
  </label>
</div>
