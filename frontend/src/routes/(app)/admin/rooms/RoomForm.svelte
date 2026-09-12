<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import Input from '$lib/components/ui/Input.svelte';
  import type { Room } from '$lib/api/types';

  let { room }: { room?: Room | null } = $props();
</script>

{#if room}
  <input type="hidden" name="id" value={room.id} />
{/if}

<div class="space-y-4">
  <Input
    label={m.room_number()}
    name="room_number"
    value={room?.room_number ?? ''}
    required
    maxlength={16}
    autocapitalize="characters"
  />
  <Input label={m.room_name()} name="name" value={room?.name ?? ''} maxlength={64} />
  <Input
    label={m.floor()}
    name="floor"
    type="number"
    value={room?.floor?.toString() ?? ''}
    min={-5}
    max={100}
  />
</div>
