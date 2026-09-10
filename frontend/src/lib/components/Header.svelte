<script lang="ts">
  import {page} from '$app/state';
  import {resolve} from '$app/paths';
  import type {AuthenticatedUser} from '$lib/api/types';

  let {user}: { user: AuthenticatedUser } = $props();

  const isStaff = $derived(user.role !== 'client');

  const links = $derived(
    isStaff
      ? [{href: resolve('/staff'), label: 'Oversigt'}]
      : [{href: resolve('/me'), label: 'Mit værelse'}]
  );

  const roleLabel: Record<AuthenticatedUser['role'], string> = {
    client: 'Klient',
    staff: 'Personale',
    admin: 'Administrator'
  };
</script>

<header class="border-b border-surface-0 bg-base">
  <div class="mx-auto flex h-14 max-w-6xl items-center justify-between px-4">
    <div class="flex items-center gap-8">
      <a class="font-semibold tracking-tight" href={resolve('/')}>SenseCare</a>

      <nav aria-label="Hovedmenu" class="flex gap-1 overflow-x-auto text-sm">
        {#each links as link (link.href)}
          {@const active = page.url.pathname.startsWith(link.href)}
          <a
            href={link.href}
            aria-current={active ? 'page' : undefined}
            class="rounded-md px-3 py-1.5 transition {active
              ? 'bg-accent/10 font-medium text-accent'
              : 'text-subtext hover:bg-surface-0/60 hover:text-text'}"
          >
            {link.label}
          </a>
        {/each}
      </nav>
    </div>

    <div class="flex items-center gap-4 text-sm">
			<span class="hidden sm:inline">
        {user.display_name}
        <span class="text-subtext">· {roleLabel[user.role]}</span>
      </span>
      <form action={resolve('/logout')} method="POST">
        <button class="rounded-md px-3 py-1.5 text-subtext transition hover:bg-surface-0/60 hover:text-text">
          Log ud
        </button>
      </form>
    </div>
  </div>
</header>
