<script lang="ts">
  import type {Snippet} from 'svelte';
  import type {HTMLButtonAttributes} from 'svelte/elements';

  type Variant = 'primary' | 'secondary' | 'danger' | 'ghost';

  let {
    variant = 'primary',
    type = 'button',
    loading = false,
    children,
    class: extra = '',
    ...rest
  }: HTMLButtonAttributes & { variant?: Variant; loading?: boolean; children: Snippet } = $props();

  const styles: Record<Variant, string> = {
    primary: 'bg-accent text-canvas shadow-md shadow-accent/25 hover:bg-accent/90 hover:shadow-lg hover:shadow-accent/30',
    secondary: 'bg-surface-0 text-text shadow-sm shadow-crust/50 ring-1 ring-surface-1 hover:bg-surface-1',
    danger: 'bg-danger text-canvas shadow-md shadow-danger/25 hover:bg-danger/90 hover:shadow-lg hover:shadow-danger/30',
    ghost: 'text-subtext hover:bg-surface-0/60 hover:text-text'
  };
</script>

<button
  {...rest}
  class="inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all
  duration-200 hover:-translate-y-px active:translate-y-0 active:shadow-sm focus-visible:ring-2
  focus-visible:ring-accent/40 focus-visible:outline-none disabled:opacity-60 disabled:hover:translate-y-0
  {styles[variant]} {extra}"
  disabled={loading || rest.disabled}
  {type}
>
  {@render children()}
</button>
