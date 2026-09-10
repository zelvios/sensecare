<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    value,
    display,
    min,
    max,
    icon
  }: {
    label: string;
    value: number | null;
    display: string;
    min: number;
    max: number;
    icon: Snippet;
  } = $props();

  const tone = $derived(
    value === null ? 'text-overlay' : value < min || value > max ? 'text-danger' : 'text-ok'
  );
</script>

<div class="rounded-2xl bg-canvas p-5 shadow-lg ring-1 shadow-crust/60 ring-surface-0">
  <div class="flex items-center gap-2 text-sm text-subtext">
    {@render icon()}
    {label}
  </div>
  <p class="mt-2 text-4xl font-semibold num {tone}">{display}</p>
</div>
