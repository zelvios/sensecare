<script lang="ts">
  import { Languages } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';
  import { getLocale, type Locale, locales, setLocale } from '$lib/paraglide/runtime';

  let { class: extra = '' }: { class?: string } = $props();

  const languageNames: Record<Locale, string> = { da: 'Dansk', en: 'English' };
</script>

<label class="inline-flex items-center gap-1.5 text-subtext {extra}">
  <Languages aria-hidden="true" class="size-4" />
  <span class="sr-only">{m.language()}</span>
  <select
    class="rounded-md border border-surface-0 bg-canvas px-2 py-1 text-sm text-subtext focus:ring-2 focus:ring-accent/30 focus:outline-none"
    onchange={(e) => setLocale(e.currentTarget.value as Locale)}
    value={getLocale()}
  >
    {#each locales as locale (locale)}
      <option value={locale}>{languageNames[locale]}</option>
    {/each}
  </select>
</label>
