<script lang="ts">
  import { resolve } from '$app/paths';
  import * as m from '$lib/paraglide/messages';

  const sections = [
    { href: resolve('/admin/rooms'), title: m.admin_rooms(), text: m.admin_rooms_text() }
    // accounts, devices and audit are added as their pages land
  ];
</script>

<svelte:head>
  <title>{m.title_admin()}</title>
</svelte:head>

<h1 class="text-3xl font-semibold tracking-tight">{m.nav_admin()}</h1>

<div class="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
  {#each sections as s (s.href)}
    <a
      href={s.href}
      class="group rounded-2xl bg-canvas p-5 shadow-md ring-1 shadow-crust/50 ring-surface-0 transition hover:bg-accent/10 hover:shadow-lg hover:ring-accent/40"
    >
      <h2 class="text-lg font-semibold transition-colors group-hover:text-accent">{s.title}</h2>
      <p class="mt-1 text-sm text-subtext transition-colors group-hover:text-accent/80">{s.text}</p>
    </a>
  {/each}
</div>
