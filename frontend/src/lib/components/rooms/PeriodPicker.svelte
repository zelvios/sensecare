<script lang="ts">
  import { page } from '$app/state';
  import { resolve } from '$app/paths';
  import * as m from '$lib/paraglide/messages';
  import type { PeriodHours } from '$lib/api/types';

  let { current }: { current: PeriodHours } = $props();

  const options: { hours: PeriodHours; label: string }[] = [
    { hours: 24, label: m.period_24h() },
    { hours: 72, label: m.period_3d() },
    { hours: 168, label: m.period_7d() }
  ];

  let hovered = $state<number | null>(null);

  // The pill sits under the hovered option, or under the active one when nothing is hovered.
  const activeIndex = $derived(options.findIndex((o) => o.hours === current));
  const pillIndex = $derived(hovered ?? activeIndex);
</script>

<nav
  aria-label={m.period()}
  class="relative inline-grid grid-cols-3 rounded-lg bg-surface-0 p-1 text-sm"
  onmouseleave={() => (hovered = null)}
>
  <span
    aria-hidden="true"
    class="absolute top-1 bottom-1 left-1 w-[calc((100%-0.5rem)/3)] rounded-md bg-canvas shadow-sm transition-transform duration-200 ease-out"
    style="transform: translateX({pillIndex * 100}%)"
  ></span>

  {#each options as o, i (o.hours)}
    <a
      href="{resolve(page.url.pathname as '/me')}?hours={o.hours}"
      aria-current={o.hours === current ? 'page' : undefined}
      onmouseenter={() => (hovered = i)}
      onfocus={() => (hovered = i)}
      onblur={() => (hovered = null)}
      class="relative z-10 rounded-md px-3 py-1.5 text-center transition-colors {o.hours === current
        ? 'font-medium text-text'
        : 'text-subtext hover:text-text'}"
    >
      {o.label}
    </a>
  {/each}
</nav>
