<script lang="ts">
  import {enhance} from '$app/forms';
  import * as m from '$lib/paraglide/messages';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import LanguageSwitch from '$lib/components/layout/LanguageSwitch.svelte';

  let {form} = $props();
  let submitting = $state(false);

  const errorText = $derived(
    form?.error === 'account_deactivated' ? m.login_deactivated() : form?.error ? m.login_failed() : null
  );
</script>

<svelte:head>
  <title>{m.title_login()}</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center px-4">
  <Card class="w-full max-w-sm p-8">
    <div class="mb-8 flex items-start justify-between gap-4">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight">SenseCare</h1>
        <p class="mt-1 text-sm text-subtext">{m.app_tagline()}</p>
      </div>
      <LanguageSwitch />
    </div>

    <form
      class="space-y-5"
      method="POST"
      use:enhance={() => {
				submitting = true;
				return async ({ update }) => {
					await update();
					submitting = false;
				};
			}}
    >
      <Input
        autocapitalize="off"
        autocomplete="username"
        invalid={!!form?.error}
        label={m.login_username()}
        name="username"
        required
        spellcheck="false"
        value={form?.username ?? ''}
      />

      <Input
        autocomplete="current-password"
        invalid={!!form?.error}
        label={m.login_password()}
        name="password"
        required
        type="password"
      />

      {#if errorText}
        <p class="rounded-lg bg-danger/10 px-3 py-2 text-sm text-danger" role="alert">
          {errorText}
        </p>
      {/if}

      <Button class="w-full py-2.5" loading={submitting} type="submit">
        {submitting ? m.login_submitting() : m.login_submit()}
      </Button>
    </form>
  </Card>
</main>
