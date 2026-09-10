<script lang="ts">
  import type {Snippet} from 'svelte';

  let {children, class: extra = ''}: { children: Snippet; class?: string } = $props();
</script>

<div class="rounded-2xl bg-base p-6 shadow-lg shadow-crust/60 ring-1 ring-surface-0 {extra}">
  {@render children()}
</div>
