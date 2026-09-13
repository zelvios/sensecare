<script lang="ts">
  import { resolve } from '$app/paths';
  import { ArrowLeft, ChevronLeft, ChevronRight } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import AuditList from '$lib/components/audit/AuditList.svelte';

  let { data } = $props();

  const typeLabel: Record<string, () => string> = {
    user: m.audit_group_account,
    room: m.audit_group_room,
    device: m.audit_group_device,
    threshold: m.audit_group_threshold,
    stay: m.audit_group_stay,
    service_call: m.audit_group_call,
    alarm: m.audit_group_alarm
  };

  const from = $derived(data.offset + 1);
  const to = $derived(data.offset + data.entries.length);
  const newerOffset = $derived(Math.max(0, data.offset - data.pageSize));
  const olderOffset = $derived(data.offset + data.pageSize);
</script>

{#snippet pageForm(offset: number, disabled: boolean, label: string, older: boolean)}
  <form method="GET" action={resolve('/admin/audit')}>
    {#if data.entityType}<input type="hidden" name="entity_type" value={data.entityType} />{/if}
    {#if data.actorId}<input type="hidden" name="actor_id" value={data.actorId} />{/if}
    {#if offset > 0}<input type="hidden" name="offset" value={offset} />{/if}
    <Button type="submit" variant="secondary" class="px-3 py-1 text-xs" {disabled}>
      {#if !older}<ChevronLeft class="size-4" aria-hidden="true" />{/if}
      {label}
      {#if older}<ChevronRight class="size-4" aria-hidden="true" />{/if}
    </Button>
  </form>
{/snippet}

<svelte:head>
  <title>{m.title_admin_audit()}</title>
</svelte:head>

<a
  class="inline-flex items-center gap-1 text-sm text-subtext hover:text-text"
  href={resolve('/admin')}
>
  <ArrowLeft aria-hidden="true" class="size-4" />
  {m.nav_admin()}
</a>

<h1 class="mt-3 text-3xl font-semibold tracking-tight">{m.admin_audit()}</h1>

<form method="GET" action={resolve('/admin/audit')} class="mt-6 flex flex-wrap items-end gap-3">
  <label class="block">
    <span class="mb-1 block text-xs font-medium">{m.audit_group()}</span>
    <select
      name="entity_type"
      class="rounded-md border bg-canvas py-1.5 pr-7 pl-2 text-sm focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    >
      <option value="" selected={data.entityType === ''}>{m.all()}</option>
      {#each data.entityTypes as t (t)}
        <option value={t} selected={data.entityType === t}>{typeLabel[t]?.() ?? t}</option>
      {/each}
    </select>
  </label>
  <label class="block w-full sm:w-80">
    <span class="mb-1 block text-xs font-medium">{m.audit_actor_id()}</span>
    <input
      type="text"
      name="actor_id"
      value={data.actorId}
      autocomplete="off"
      spellcheck="false"
      class="w-full rounded-md border bg-canvas px-2 py-1.5 text-sm num placeholder:text-overlay focus:border-accent focus:ring-2 focus:ring-accent/30 focus:outline-none"
    />
  </label>
  <Button type="submit" variant="secondary" class="px-3 py-1.5 text-sm">{m.apply_filter()}</Button>
  {#if data.entityType || data.actorId}
    <a href={resolve('/admin/audit')} class="text-sm text-subtext hover:text-text">{m.clear()}</a>
  {/if}
</form>

<div class="mt-6">
  <AuditList entries={data.entries} actors={data.actors} empty={m.audit_none()} />
</div>

{#if data.offset > 0 || data.hasMore}
  <div class="mt-4 flex items-center justify-between text-sm text-subtext">
    <span class="num">
      {#if data.entries.length > 0}{m.audit_range()} {from}-{to}{/if}
    </span>
    <div class="flex gap-2">
      {@render pageForm(newerOffset, data.offset === 0, m.audit_newer(), false)}
      {@render pageForm(olderOffset, !data.hasMore, m.audit_older(), true)}
    </div>
  </div>
{/if}
