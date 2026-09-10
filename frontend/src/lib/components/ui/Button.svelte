<script lang="ts">
  import type {Snippet} from 'svelte';
  import type {HTMLButtonAttributes} from 'svelte/elements';

  type Variant = 'primary' | 'secondary' | 'danger' | 'ghost';

  let {
    variant = 'primary',
    loading = false,
    children,
    class: extra = '',
    ...rest
  }: HTMLButtonAttributes & { variant?: Variant; loading?: boolean; children: Snippet } = $props();

  const styles: Record<Variant, string> = {
    primary: 'bg-accent text-base hover:bg-accent/90',
    secondary: 'bg-surface-0 text-text hover:bg-surface-1',
    danger: 'bg-danger text-base hover:bg-danger/90',
    ghost: 'text-subtext hover:bg-surface-0/60 hover:text-text'
  };
</script>

<button
  {...rest}
  disabled={loading || rest.disabled}
  class="inline-flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition focus:ring-2 focus:ring-accent/40 focus:outline-none disabled:opacity-60 {styles[variant]} {extra}"
>
  {@render children()}
</button>
