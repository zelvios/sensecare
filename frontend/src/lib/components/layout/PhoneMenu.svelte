<script lang="ts">
  import {afterNavigate} from '$app/navigation';
  import {resolve} from '$app/paths';
  import {Menu, X} from '@lucide/svelte';
  import LanguageSwitch from '$lib/components/layout/LanguageSwitch.svelte';
  import * as m from '$lib/paraglide/messages';
  import type {AuthenticatedUser} from '$lib/api/types';

  type Link = { href: string; label: string };

  let {
    user,
    links,
    roleLabel,
    isActive
  }: {
    user: AuthenticatedUser;
    links: Link[];
    roleLabel: string;
    isActive: (href: string) => boolean;
  } = $props();

  let dialog: HTMLDialogElement;

  // Navigation happened, close the sheet.
  afterNavigate(() => dialog?.close());
</script>

<button
  aria-label={m.menu_open()}
  class="-ml-2 rounded-md p-3 text-subtext hover:bg-surface-0/60 hover:text-text"
  onclick={() => dialog.showModal()}
  type="button"
>
  <Menu aria-hidden="true" class="size-6"/>
</button>

<dialog
  aria-label={m.nav_label()}
  bind:this={dialog}
  class="m-0 h-dvh max-h-none w-screen max-w-none bg-canvas text-text backdrop:bg-crust/60"
>
  <div class="flex h-full flex-col">
    <div class="flex h-14 items-center justify-between border-b border-surface-0 px-4">
      <span class="font-semibold tracking-tight">SenseCare</span>
      <button
        aria-label={m.menu_close()}
        class="-mr-2 rounded-md p-3 text-subtext hover:bg-surface-0/60 hover:text-text"
        onclick={() => dialog.close()}
        type="button"
      >
        <X aria-hidden="true" class="size-6"/>
      </button>
    </div>

    <nav class="flex flex-1 flex-col gap-1 p-4">
      {#each links as link (link.href)}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
        <a href={link.href}
           aria-current={isActive(link.href) ? 'page' : undefined}
           class="rounded-xl px-4 py-4 text-lg {isActive(link.href)
        ? 'bg-accent/10 font-medium text-accent'
        : 'hover:bg-surface-0/60'}"
        >
          {link.label}
        </a>
      {/each}
    </nav>

    <div class="flex items-center gap-3 border-t border-surface-0 px-4 py-3">
      <LanguageSwitch class="shrink-0" />
      <p class="@container flex min-w-0 flex-1 items-center gap-1">
        <span class="min-w-0 truncate" title={user.display_name}>{user.display_name}</span>
        <span class="hidden shrink-0 text-subtext @[12rem]:inline">- {roleLabel}</span>
      </p>
      <form action={resolve('/logout')} method="POST" class="shrink-0">
        <button
          class="rounded-lg bg-surface-0 px-3 py-1.5 transition hover:bg-danger/10 hover:text-danger focus-visible:ring-2 focus-visible:ring-danger/40 focus-visible:outline-none"
          type="submit"
        >
          {m.logout()}
        </button>
      </form>
    </div>
  </div>
</dialog>
