<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import Input from '$lib/components/ui/Input.svelte';
  import type { Threshold } from '$lib/api/types';

  let {
    values
  }: {
    values: Pick<
      Threshold,
      'temperature_min' | 'temperature_max' | 'humidity_min' | 'humidity_max'
    >;
  } = $props();
</script>

<div class="grid gap-4 sm:grid-cols-2">
  <Input
    label={m.temperature_min()}
    name="temperature_min"
    type="number"
    step="0.1"
    min={-40}
    max={85}
    required
    value={String(values.temperature_min)}
  />
  <Input
    label={m.temperature_max()}
    name="temperature_max"
    type="number"
    step="0.1"
    min={-40}
    max={85}
    required
    value={String(values.temperature_max)}
  />
  <Input
    label={m.humidity_min()}
    name="humidity_min"
    type="number"
    step="0.1"
    min={0}
    max={100}
    required
    value={String(values.humidity_min)}
  />
  <Input
    label={m.humidity_max()}
    name="humidity_max"
    type="number"
    step="0.1"
    min={0}
    max={100}
    required
    value={String(values.humidity_max)}
  />
</div>
