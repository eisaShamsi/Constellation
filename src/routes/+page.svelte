<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let greetMsg = $state("");

  async function checkBackend() {
    greetMsg = await invoke("greet", { name: "Constellation" });
  }
</script>

<main class="container">
  <div class="hero">
    <h1>Constellation</h1>
    <p class="tagline">A Map of Maps</p>
    <p class="description">
      Your unified dashboard for multiple Obsidian vaults.
      One view across all your knowledge universes.
    </p>
    <button onclick={checkBackend}>Test Backend Connection</button>
    {#if greetMsg}
      <p class="status">{greetMsg}</p>
    {/if}
  </div>
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
  color: #e0e0e0;
  background-color: #0d1117;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.container {
  margin: 0;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  padding: 2rem;
}

.hero {
  max-width: 600px;
}

h1 {
  font-size: 3.5rem;
  font-weight: 700;
  background: linear-gradient(135deg, #7c3aed, #3b82f6, #06b6d4);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 0.25rem;
}

.tagline {
  font-size: 1.25rem;
  color: #8b949e;
  margin-top: 0;
  margin-bottom: 2rem;
  font-style: italic;
}

.description {
  font-size: 1rem;
  color: #8b949e;
  line-height: 1.6;
  margin-bottom: 2rem;
}

button {
  border-radius: 8px;
  border: 1px solid #30363d;
  padding: 0.75em 1.5em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #e0e0e0;
  background-color: #21262d;
  cursor: pointer;
  transition: all 0.2s ease;
}

button:hover {
  border-color: #7c3aed;
  background-color: #30363d;
}

.status {
  margin-top: 1rem;
  color: #3fb950;
  font-size: 0.9rem;
}
</style>
