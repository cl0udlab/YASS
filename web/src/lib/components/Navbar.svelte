<script lang="ts">
  import { page } from "$app/state";
  import { logout } from "#lib/api.ts";
  import { THEMES, setTheme, theme, type Theme } from "#lib/theme.svelte.ts";
</script>

<nav class="navbar">
  <div class="nav-container">
    <a class="nav-brand" href="/">YASS</a>
    <ul class="nav-menu">
      <li>
        <a href="/" class="nav-link" class:active={page.url.pathname === "/"}
          >首頁</a
        >
      </li>
      <li>
        <button class="nav-link" onclick={logout}>登出</button>
      </li>
      <li>
        <select
          value={theme.current}
          onchange={(e) => setTheme(e.currentTarget.value as Theme)}
          aria-label="主題"
        >
          {#each THEMES as t (t.value)}
            <option value={t.value}>{t.label}</option>
          {/each}
        </select>
      </li>
    </ul>
  </div>
</nav>

<style>
  .navbar {
    background-color: var(--nav-bg);
    padding: 1rem 0;
    box-shadow: 0 2px 4px var(--shadow-color);
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 1000;
  }

  .nav-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .nav-brand {
    font-size: 1.5rem;
    font-weight: bold;
    text-decoration: none;
    color: var(--primary-color);
  }

  .nav-menu {
    display: flex;
    list-style: none;
    align-items: center;
    gap: 1rem;
  }

  .nav-link {
    text-decoration: none;
    color: var(--text-color);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    transition: background-color 0.3s;
    cursor: pointer;
    background: none;
    border: none;
    font: inherit;
  }

  .nav-link:hover,
  .nav-link.active {
    background-color: var(--hover-bg);
  }

  select {
    padding: 0.5rem;
  }

  @media (max-width: 768px) {
    .nav-container {
      flex-direction: column;
      gap: 1rem;
    }

    .nav-menu {
      flex-wrap: wrap;
      justify-content: center;
    }
  }
</style>
