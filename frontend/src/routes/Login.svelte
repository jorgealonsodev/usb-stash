<script lang="ts">
  import { Link, navigate } from "svelte-routing";
  import { openStash } from "../lib/api";
  import { currentStashPath } from "../lib/stores";
  import PasswordInput from "../lib/components/PasswordInput.svelte";

  let path = "";
  let password = "";
  let loading = false;
  let error = "";

  async function handleSubmit() {
    loading = true;
    error = "";

    try {
      await openStash(path, password);
      currentStashPath.set(path);
      navigate("/explorer");
    } catch {
      error = "Credenciales incorrectas o archivo corrupto";
      loading = false;
    }
  }
</script>

<div class="login">
  <h1>Abre tu USB Stash</h1>
  <p class="subtitle">Selecciona un stash existente para desbloquearlo.</p>

  <form on:submit|preventDefault={handleSubmit}>
    <div class="field">
      <label for="path">Ruta del stash</label>
      <input
        id="path"
        type="text"
        placeholder="/ruta/a/tu/stash"
        bind:value={path}
        disabled={loading}
      />
    </div>

    <div class="field">
      <label for="password">Contraseña</label>
      <PasswordInput
        id="password"
        placeholder="Tu contraseña maestra"
        bind:value={password}
        disabled={loading}
      />
    </div>

    <button type="submit" class="btn btn-primary" disabled={loading}>
      {loading ? "Desbloqueando..." : "Desbloquear"}
    </button>

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </form>

  <p class="create-link">
    ¿No tenés un stash? <Link to="/create">Crear stash</Link>
  </p>
</div>

<style>
  .login {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    max-width: 400px;
    margin: 2rem auto;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  label {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .btn {
    padding: 0.625rem 1rem;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .btn-primary {
    background: var(--accent);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error {
    color: var(--danger);
    font-size: 0.8125rem;
    text-align: center;
  }

  .create-link {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    text-align: center;
  }
</style>
