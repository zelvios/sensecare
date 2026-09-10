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
</script>

<nav aria-label={m.period()} class="inline-flex rounded-lg bg-surface-0 p-1 text-sm">
  {#each options as o (o.hours)}
    <a
      href="{resolve(page.url.pathname as '/me')}?hours={o.hours}"
      aria-current={o.hours === current ? 'page' : undefined}
      class="rounded-md px-3 py-1.5 transition {o.hours === current
        ? 'bg-canvas font-medium shadow-sm'
        : 'text-subtext hover:text-text'}"
    >
      {o.label}
    </a>
  {/each}
</nav>
