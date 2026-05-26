<script lang="ts">
  import { Router, Route } from "svelte-routing";
  import Login from "./routes/Login.svelte";
  import Create from "./routes/Create.svelte";
  import Explorer from "./routes/Explorer.svelte";
  import Settings from "./routes/Settings.svelte";
  import "./app.css";
</script>

<Router>
  <header class="wordmark">
    <span>USB Stash</span>
  </header>

  <main>
    <Route path="/" component={Login} />
    <Route path="/create" component={Create} />
    <Route path="/explorer" component={Explorer} />
    <Route path="/settings" component={Settings} />
  </main>
</Router>

<style>
  .wordmark {
    display: flex;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
  }

  .wordmark span {
    font-size: 1.125rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    color: var(--text-primary);
  }

  main {
    padding: 2rem 1.5rem;
    max-width: 800px;
    margin: 0 auto;
  }
</style>
