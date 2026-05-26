<script lang="ts">
  import { Link, navigate } from "svelte-routing";
  import zxcvbn from "zxcvbn";
  import { createStash } from "../lib/api";
  import { currentStashPath } from "../lib/stores";
  import PasswordInput from "../lib/components/PasswordInput.svelte";
  import EntropyBar from "../lib/components/EntropyBar.svelte";

  let path = "";
  let password = "";
  let confirm = "";
  let loading = false;
  let error = "";

  $: passwordValid = password.length >= 12;
  $: entropy = zxcvbn(password).score;
  $: passwordsMatch = password === confirm;
  $: canSubmit = passwordValid && entropy >= 3 && passwordsMatch && !loading;

  async function handleSubmit() {
    if (!canSubmit) return;

    loading = true;
    error = "";

    try {
      await createStash(path, password);
      currentStashPath.set(path);
      navigate("/explorer");
    } catch {
      error = "No se pudo crear el stash. Verificá la ruta y los permisos.";
      loading = false;
    }
  }
</script>

<div class="create">
  <h1>Crea tu USB Stash</h1>
  <p class="subtitle">Elegí una ubicación y una contraseña maestra para tu nuevo stash.</p>

  <form on:submit|preventDefault={handleSubmit}>
    <div class="field">
      <label for="path">Ubicación</label>
      <input
        id="path"
        type="text"
        placeholder="/ruta/para/nuevo/stash"
        bind:value={path}
        disabled={loading}
      />
    </div>

    <div class="field">
      <label for="password">Contraseña maestra</label>
      <PasswordInput
        id="password"
        placeholder="Mínimo 12 caracteres"
        bind:value={password}
        disabled={loading}
      />
      <EntropyBar score={entropy} />
      {#if password.length > 0 && !passwordValid}
        <p class="validation-error">Mínimo 12 caracteres</p>
      {/if}
      {#if passwordValid && entropy < 3}
        <p class="validation-error">La contraseña es demasiado débil</p>
      {/if}
    </div>

    <div class="field">
      <label for="confirm">Confirmar contraseña</label>
      <input
        id="confirm"
        type="password"
        placeholder="Repetí la contraseña"
        bind:value={confirm}
        disabled={loading}
      />
      {#if confirm.length > 0 && !passwordsMatch}
        <p class="validation-error">Las contraseñas no coinciden</p>
      {/if}
    </div>

    <button type="submit" class="btn btn-primary" disabled={!canSubmit}>
      {loading ? "Creando..." : "Crear stash"}
    </button>

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </form>

  <p class="back-link">
    <Link to="/">← Volver</Link>
  </p>
</div>

<style>
  .create {
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

  .validation-error {
    color: var(--danger);
    font-size: 0.75rem;
  }

  .error {
    color: var(--danger);
    font-size: 0.8125rem;
    text-align: center;
  }

  .back-link {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    text-align: center;
  }
</style>
