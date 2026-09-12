<script lang="ts">
  import { page } from '$app/state';
  import { resolve } from '$app/paths';
  import * as m from '$lib/paraglide/messages';
  import LanguageSwitch from '$lib/components/layout/LanguageSwitch.svelte';
  import PhoneMenu from '$lib/components/layout/PhoneMenu.svelte';
  import type { AuthenticatedUser } from '$lib/api/types';

  let { user }: { user: AuthenticatedUser } = $props();

  const links = $derived.by(() => {
    if (user.role === 'client') return [{ href: resolve('/me'), label: m.nav_my_room() }];
    const staff = [
      { href: resolve('/staff'), label: m.nav_overview() },
      { href: resolve('/staff/calls'), label: m.nav_calls() }
    ];
    return user.role === 'admin'
      ? [...staff, { href: resolve('/admin'), label: m.nav_admin() }]
      : staff;
  });

  const roleLabels = $derived<Record<AuthenticatedUser['role'], string>>({
    client: m.role_client(),
    staff: m.role_staff(),
    admin: m.role_admin()
  });

  const isActive = (href: string) => page.url.pathname.startsWith(href);
</script>

<header class="border-b border-surface-0 bg-canvas">
  <div class="mx-auto flex h-14 max-w-6xl items-center justify-between gap-4 px-4">
    <div class="flex items-center gap-6">
      <div class="md:hidden">
        <PhoneMenu {isActive} {links} roleLabel={roleLabels[user.role]} {user} />
      </div>

      <a class="font-semibold tracking-tight" href={resolve('/')}>SenseCare</a>

      <nav aria-label={m.nav_label()} class="hidden gap-1 text-sm md:flex">
        {#each links as link (link.href)}
          <a
            href={link.href}
            aria-current={isActive(link.href) ? 'page' : undefined}
            class="rounded-md px-3 py-1.5 transition {isActive(link.href)
              ? 'bg-accent/10 font-medium text-accent'
              : 'text-subtext hover:bg-surface-0/60 hover:text-text'}"
          >
            {link.label}
          </a>
        {/each}
      </nav>
    </div>

    <div class="hidden min-w-0 items-center gap-3 text-sm md:flex">
      <LanguageSwitch class="shrink-0" />

      <p class="@container flex w-56 items-center gap-1">
        <span class="min-w-0 truncate" title={user.display_name}>{user.display_name}</span>
        <span class="hidden shrink-0 text-subtext @[12rem]:inline">- {roleLabels[user.role]}</span>
      </p>

      <form action={resolve('/logout')} class="shrink-0" method="POST">
        <button
          class="rounded-lg bg-surface-0 px-3 py-1.5 transition hover:bg-danger/10 hover:text-danger focus-visible:ring-2 focus-visible:ring-danger/40 focus-visible:outline-none"
          type="submit"
        >
          {m.logout()}
        </button>
      </form>
    </div>
  </div>
</header>
