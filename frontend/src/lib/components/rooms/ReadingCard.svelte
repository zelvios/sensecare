<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as m from '$lib/paraglide/messages';

  let {
    label,
    value,
    display,
    min,
    max,
    margin = 0,
    range,
    icon
  }: {
    label: string;
    value: number | null;
    display: string;
    min: number;
    max: number;
    margin?: number;
    /** The allowed range as text, e.g. "19 °C - 26 °C". */
    range?: string;
    icon?: Snippet;
  } = $props();

  const tone = $derived.by(() => {
    if (value === null) return 'text-overlay';
    if (value < min || value > max) return 'text-danger';
    if (value < min + margin || value > max - margin) return 'text-warn';
    return 'text-ok';
  });
</script>

<div class="rounded-2xl bg-canvas p-5 shadow-lg ring-1 shadow-crust/60 ring-surface-0">
  <div class="flex items-center gap-2 text-sm font-bold text-subtext">
    {#if icon}{@render icon()}{/if}
    {label}
  </div>
  <p class="mt-2 text-4xl font-semibold num {tone}">{display}</p>
  {#if range}
    <p class="mt-1 text-xs text-subtext num">
      <span class="font-semibold">{m.limits()}:</span>
      {range}
    </p>
  {/if}
</div>
