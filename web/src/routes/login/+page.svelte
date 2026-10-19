<script lang="ts">
  import { goto } from "$app/navigation";
  import { login } from "#lib/api.ts";
  import { showAlert } from "#lib/alerts.svelte.ts";

  let username = $state("");
  let password = $state("");
  let submitting = $state(false);

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    submitting = true;
    try {
      await login(username.trim(), password);
      await goto("/");
    } catch (error) {
      showAlert((error as Error).message || "帳號或密碼錯誤", "error");
    } finally {
      submitting = false;
    }
  }
</script>

<div class="login-page">
  <form class="login-card card" onsubmit={submit}>
    <div class="login-brand">YASS</div>
    <div class="form-group">
      <label for="login-user">帳號</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="login-user"
        type="text"
        autocomplete="username"
        bind:value={username}
        autofocus
      />
    </div>
    <div class="form-group">
      <label for="login-pass">密碼</label>
      <input
        id="login-pass"
        type="password"
        autocomplete="current-password"
        bind:value={password}
      />
    </div>
    <div class="actions">
      <button type="submit" class="btn btn-primary" disabled={submitting}
        >登入</button
      >
    </div>
  </form>
</div>

<style>
  .login-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }

  .login-card {
    width: 100%;
    max-width: 380px;
    border: 1px solid var(--border-color);
  }

  .login-brand {
    text-align: center;
    font-size: 1.75rem;
    font-weight: 600;
    color: var(--primary-color);
    margin-bottom: 1.5rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 1.25rem;
  }
</style>
