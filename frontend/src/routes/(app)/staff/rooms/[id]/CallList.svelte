<script lang="ts">
  import { enhance } from '$app/forms';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { fmtDateTimeSec } from '$lib/utils/format';
  import type { ServiceCall } from '$lib/api/types';

  let { calls }: { calls: ServiceCall[] } = $props();
  let closing = $state<ServiceCall | null>(null);
  let dialogOpen = $derived(closing !== null);

  const statusLabel: Record<ServiceCall['status'], () => string> = {
    open: m.status_open,
    in_progress: m.status_in_progress,
    closed: m.status_closed
  };
  const statusTone: Record<ServiceCall['status'], string> = {
    open: 'bg-danger/10 text-danger',
    in_progress: 'bg-warn/10 text-warn',
    closed: 'bg-surface-0 text-subtext'
  };
</script>

<section aria-labelledby="calls-heading">
  <div class="flex items-center gap-3">
    <h2 class="text-lg font-semibold" id="calls-heading">{m.calls()}</h2>
    <span aria-hidden="true" class="h-px flex-1 bg-surface-0"></span>
    <span class="text-sm text-subtext num">{calls.length}</span>
  </div>

  {#if calls.length === 0}
    <p class="mt-3 text-sm text-subtext">{m.calls_none()}</p>
  {:else}
    <ul class="mt-3 max-h-80 divide-y overflow-y-auto rounded-2xl bg-canvas ring-1 ring-surface-0">
      {#each calls as c (c.id)}
        <li class="px-4 py-3">
          <div class="flex flex-wrap items-center gap-3">
            <span class="rounded-full px-2.5 py-0.5 text-sm font-medium {statusTone[c.status]}">
              {statusLabel[c.status]()}
            </span>

            <dl class="flex min-w-0 flex-1 flex-wrap gap-x-4 gap-y-0.5 text-xs text-subtext num">
              <div class="flex gap-1">
                <dt class="font-medium">{m.created()}</dt>
                <dd>{fmtDateTimeSec(c.created_at)}</dd>
              </div>
              {#if c.acknowledged_at}
                <div class="flex gap-1">
                  <dt class="font-medium">{m.acknowledged_at()}</dt>
                  <dd>{fmtDateTimeSec(c.acknowledged_at)}</dd>
                </div>
              {/if}
              {#if c.closed_at}
                <div class="flex gap-1">
                  <dt class="font-medium">{m.closed()}</dt>
                  <dd>{fmtDateTimeSec(c.closed_at)}</dd>
                </div>
              {/if}
            </dl>

            {#if c.status === 'open'}
              <form method="POST" action="?/acknowledgeCall" use:enhance>
                <input type="hidden" name="id" value={c.id} />
                <Button type="submit" variant="warn-soft" class="px-3 py-1 text-sm">
                  {m.acknowledge()}
                </Button>
              </form>
            {/if}
            {#if c.status !== 'closed'}
              <Button variant="danger-soft" class="px-3 py-1 text-sm" onclick={() => (closing = c)}>
                {m.close_call()}
              </Button>
            {/if}
          </div>

          {#if c.note}
            <p class="mt-1 text-subtext">{c.note}</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<Dialog
  bind:open={
    () => dialogOpen,
    (v) => {
      if (!v) closing = null;
    }
  }
  title={m.confirm_close()}
>
  {#if closing}
    <form
      id="close-call-form"
      method="POST"
      action="?/closeCall"
      use:enhance={() =>
        async ({ update }) => {
          await update();
          closing = null;
        }}
    >
      <input type="hidden" name="id" value={closing.id} />
      <p class="text-subtext">{m.close_call_help()}</p>
      <label class="mt-3 block">
        <span class="mb-1 block text-xs font-medium">{m.note()}</span>
        <textarea
          name="note"
          rows="3"
          maxlength="500"
          placeholder={m.note_placeholder()}
          class="max-h-40 w-full resize-none overflow-y-auto rounded-md border bg-canvas px-2 py-1.5 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
          >{closing.note ?? ''}</textarea
        >
      </label>
    </form>
  {/if}

  {#snippet footer()}
    <Button variant="secondary" onclick={() => (closing = null)}>{m.cancel()}</Button>
    <Button type="submit" variant="danger" form="close-call-form">{m.confirm_close()}</Button>
  {/snippet}
</Dialog>
