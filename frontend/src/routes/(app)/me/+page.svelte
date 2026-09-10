<script lang="ts">
  import * as m from '$lib/paraglide/messages';
</script>

<svelte:head>
  <title>{m.title_my_room()}</title>
</svelte:head>

<h1 class="text-xl font-semibold">{m.nav_my_room()}</h1>
<p class="mt-2 text-subtext">Client view, issue #23.</p>
