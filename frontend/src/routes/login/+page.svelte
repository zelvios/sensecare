<script lang="ts">
  import {enhance} from '$app/forms';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Input from '$lib/components/ui/Input.svelte';

  let {form} = $props();
  let submitting = $state(false);
</script>

<svelte:head>
  <title>Log ind - SenseCare</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center px-4">
  <Card class="w-full max-w-sm p-8">
    <div class="mb-8">
      <h1 class="text-2xl font-semibold tracking-tight">SenseCare</h1>
      <p class="mt-1 text-sm text-subtext">Indeklima og tilkald</p>
    </div>

    <form
      method="POST"
      use:enhance={() => {
				submitting = true;
				return async ({ update }) => {
					await update();
					submitting = false;
				};
			}}
      class="space-y-5"
    >
      <Input
        label="Brugernavn"
        name="username"
        value={form?.username ?? ''}
        invalid={!!form?.error}
        autocomplete="username"
        autocapitalize="off"
        spellcheck="false"
        required
      />

      <Input
        label="Adgangskode"
        name="password"
        type="password"
        invalid={!!form?.error}
        autocomplete="current-password"
        required
      />

      {#if form?.error}
        <p class="rounded-lg bg-danger/10 px-3 py-2 text-sm text-danger" role="alert">
          {form.error}
        </p>
      {/if}

      <Button type="submit" loading={submitting} class="w-full py-2.5">
        {submitting ? 'Logger ind…' : 'Log ind'}
      </Button>
    </form>
  </Card>
</main>
