<script lang="ts">
  import type { Snippet } from 'svelte';
  import { X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  let {
    open = $bindable(false),
    title,
    children,
    footer
  }: {
    open?: boolean;
    title: string;
    children: Snippet;
    footer?: Snippet;
  } = $props();

  let dialog: HTMLDialogElement;
  const titleId = $props.id();

  $effect(() => {
    if (open && !dialog.open) dialog.showModal();
    else if (!open && dialog.open) dialog.close();
  });

  function onBackdropClick(e: MouseEvent) {
    if (e.target === dialog) open = false;
  }
</script>

<dialog
  aria-labelledby={titleId}
  bind:this={dialog}
  class="fixed inset-x-0 bottom-0 m-0 w-full max-w-none bg-transparent p-0
    backdrop:bg-crust/60 sm:inset-auto sm:m-auto sm:max-w-md"
  onclick={onBackdropClick}
  onclose={() => (open = false)}
>
  <div
    class="rounded-t-2xl bg-canvas p-5 shadow-xl ring-1 shadow-crust/60 ring-surface-0 sm:rounded-2xl"
  >
    <div class="flex items-start justify-between gap-4">
      <h2 class="text-lg font-semibold" id={titleId}>{title}</h2>
      <button
        aria-label={m.close()}
        class="-mt-1 -mr-2 rounded-md p-2 text-subtext hover:bg-surface-0/60 hover:text-text"
        onclick={() => (open = false)}
        type="button"
      >
        <X aria-hidden="true" class="size-5" />
      </button>
    </div>

    <div class="mt-3 text-sm">
      {@render children()}
    </div>

    {#if footer}
      <div class="mt-5 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
        {@render footer()}
      </div>
    {/if}
  </div>
</dialog>
