<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import favicon from "#lib/assets/favicon.svg";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { checkSession } from "#lib/api.ts";
  import { showAlert } from "#lib/alerts.svelte.ts";
  import Alerts from "#lib/components/Alerts.svelte";
  import Navbar from "#lib/components/Navbar.svelte";
  import type { LayoutProps } from "./$types";

  let { children }: LayoutProps = $props();

  const isLogin = $derived(page.url.pathname === "/login");

  let ready = $state(false);
  onMount(async () => {
    if (!(await checkSession()) && !isLogin) await goto("/login");
    ready = true;
  });
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<svelte:window
  ononline={() => showAlert("網路連線已恢復", "success")}
  onoffline={() => showAlert("網路連線已中斷", "warning")}
/>

{#if ready}
  {#if isLogin}
    <Alerts />
    {@render children()}
  {:else}
    <Navbar />
    <main class="main-content">
      <Alerts />
      {@render children()}
    </main>
  {/if}
{/if}

<style>
  .main-content {
    max-width: 1200px;
    margin: 80px auto 2rem;
    padding: 0 1rem;
    min-height: calc(100vh - 200px);
  }

  @media (max-width: 768px) {
    .main-content {
      margin-top: 120px;
    }
  }
</style>
