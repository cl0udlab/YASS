<script lang="ts">
  import { alerts } from "#lib/alerts.svelte.ts";
</script>

<div class="alert-container">
  {#each alerts as alert (alert.id)}
    <div class="alert alert-{alert.type}">{alert.message}</div>
  {/each}
</div>

<style>
  .alert-container {
    position: relative;
    z-index: 1100;
  }

  .alert {
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .alert-info {
    background-color: var(--item-bg);
    border-left: 4px solid var(--info-color);
  }

  .alert-success {
    background-color: var(--success-light);
    color: var(--success-dark);
    border-left: 4px solid var(--success-color);
  }

  .alert-error {
    background-color: var(--danger-light);
    color: var(--danger-dark);
    border-left: 4px solid var(--danger-color);
  }

  .alert-warning {
    background-color: var(--warning-light);
    color: var(--warning-dark);
    border-left: 4px solid var(--warning-color);
  }
</style>
