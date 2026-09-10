<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  let {
    label,
    name,
    value = $bindable(''),
    error,
    invalid = false,
    class: extra = '',
    ...rest
  }: HTMLInputAttributes & {
    label: string;
    name: string;
    value?: string;
    /** Message shown under the field. Also marks it invalid. */
    error?: string;
    /** Mark as invalid without a message, e.g. when the form as a whole failed. */
    invalid?: boolean;
  } = $props();

  const id = $props.id();
  let focused = $state(false);

  const bad = $derived(invalid || !!error);
  const floated = $derived(focused || (value ?? '') !== '');
</script>

<div class={extra}>
  <div class="relative">
		<span
      class="absolute top-2 bottom-2 -left-0.5 w-1.5 rounded transition-opacity duration-300 {bad
				? 'bg-danger opacity-90'
				: 'bg-accent'} {focused ? 'opacity-100' : 'opacity-60'}"
    ></span>

    <input
      {id}
      {name}
      {...rest}
      bind:value
      onfocus={() => (focused = true)}
      onblur={() => (focused = false)}
      aria-invalid={bad ? 'true' : undefined}
      aria-describedby={error ? `${id}-error` : undefined}
      class="w-full rounded-lg border bg-canvas pt-6 pr-4 pb-2 pl-6 text-sm text-text shadow-sm transition focus:ring-2 focus:outline-none {bad
				? 'border-danger/40 focus:border-transparent focus:ring-danger/30'
				: 'border-surface-0 focus:border-transparent focus:ring-accent/30'}"
    />

    <label
      for={id}
      class="pointer-events-none absolute left-6 transition-all duration-200 ease-in-out {floated
				? 'top-1.5 text-xs font-medium'
				: 'top-3.5 text-sm'} {bad ? 'text-danger' : focused ? 'text-accent' : 'text-subtext'}"
    >
      {label}
    </label>
  </div>

  {#if error}
    <p id="{id}-error" class="mt-1 pl-1 text-sm text-danger">{error}</p>
  {/if}
</div>
