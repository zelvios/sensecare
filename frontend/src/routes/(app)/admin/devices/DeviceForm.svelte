<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import Input from '$lib/components/ui/Input.svelte';
  import type { Device } from '$lib/api/types';

  let { device }: { device?: Device | null } = $props();
</script>

{#if device}
  <input type="hidden" name="id" value={device.id} />
{/if}

<div class="space-y-4">
  <Input label={m.device_label()} name="label" value={device?.label ?? ''} maxlength={64} />
  <Input
    label={m.device_firmware()}
    name="firmware_version"
    value={device?.firmware_version ?? ''}
    maxlength={32}
    autocapitalize="off"
    spellcheck="false"
  />
</div>
