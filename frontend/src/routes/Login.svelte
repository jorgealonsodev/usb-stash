<script lang="ts">
  import { Link } from "svelte-routing";
</script>

<div class="login">
  <h1>Abre tu USB Stash</h1>
  <p class="subtitle">Selecciona un stash existente para desbloquearlo.</p>

  <form on:submit|preventDefault>
    <div class="field">
      <label for="path">Ruta del stash</label>
      <input id="path" type="text" placeholder="/ruta/a/tu/stash" />
    </div>

    <div class="field">
      <label for="password">Contraseña</label>
      <input id="password" type="password" placeholder="Tu contraseña maestra" />
    </div>

    <button type="submit" class="btn btn-primary">Desbloquear</button>
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

  input {
    width: 100%;
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

  .btn-primary:hover {
    opacity: 0.9;
  }

  .create-link {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    text-align: center;
  }
</style>
