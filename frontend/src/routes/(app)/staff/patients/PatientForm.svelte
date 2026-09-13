<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import Input from '$lib/components/ui/Input.svelte';
  import type { User } from '$lib/api/types';

  let { patient }: { patient?: User | null } = $props();
</script>

{#if patient}
  <input type="hidden" name="id" value={patient.id} />
{/if}

<div class="space-y-4">
  <Input
    label={m.login_username()}
    name="username"
    value={patient?.username ?? ''}
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
    value={patient?.display_name ?? ''}
    required
    maxlength={128}
  />
  {#if !patient}
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
</div>
