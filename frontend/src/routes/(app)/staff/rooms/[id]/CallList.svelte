<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime } from '$lib/utils/format';
  import type { ServiceCall } from '$lib/api/types';

  let { calls }: { calls: ServiceCall[] } = $props();

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
    <span class="h-px flex-1 bg-surface-0" aria-hidden="true"></span>
    <span class="text-sm text-subtext num">{calls.length}</span>
  </div>

  {#if calls.length === 0}
    <p class="mt-3 text-sm text-subtext">{m.calls_none()}</p>
  {:else}
    <ul class="mt-3 max-h-80 divide-y overflow-y-auto rounded-2xl bg-canvas ring-1 ring-surface-0">
      {#each calls as c (c.id)}
        <li class="flex items-center gap-3 px-4 py-3 text-sm">
          <span class="rounded-full px-2 py-0.5 text-xs font-medium {statusTone[c.status]}">
            {statusLabel[c.status]()}
          </span>
          <span class="min-w-0 flex-1 truncate text-subtext num">{fmtDateTime(c.created_at)}</span>
          {#if c.note}
            <span class="min-w-0 truncate">{c.note}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>
