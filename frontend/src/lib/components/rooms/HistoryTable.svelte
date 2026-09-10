<script lang="ts">
  import * as m from '$lib/paraglide/messages';
  import { fmtDateTime, fmtHumidity, fmtTemp } from '$lib/utils/format';
  import type { components } from '$lib/api/schema';

  type Measurement = components['schemas']['MeasurementResponse'];
  let { rows }: { rows: Measurement[] } = $props();
</script>

{#if rows.length === 0}
  <p class="rounded-xl bg-surface-0/40 px-4 py-6 text-center text-subtext">{m.history_empty()}</p>
{:else}
  <div class="overflow-x-auto rounded-2xl ring-1 ring-surface-0">
    <table class="w-full text-sm">
      <thead class="bg-mantle text-left text-subtext">
        <tr>
          <th class="px-4 py-2 font-medium">{m.col_time()}</th>
          <th class="px-4 py-2 text-right font-medium">{m.temperature()}</th>
          <th class="px-4 py-2 text-right font-medium">{m.humidity()}</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-surface-0 bg-canvas">
        {#each rows as r (r.id)}
          <tr>
            <td class="px-4 py-2 num">{fmtDateTime(r.measured_at)}</td>
            <td class="px-4 py-2 text-right num">{fmtTemp(r.temperature_c)}</td>
            <td class="px-4 py-2 text-right num">{fmtHumidity(r.humidity_pct)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
