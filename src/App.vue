<template>
  <div id="app">
    <router-view />
  </div>
</template>

<script setup>
import { useRoute } from 'vue-router'
import { watch } from 'vue'

const route = useRoute()

// Prevent going back during exam
watch(route, (to, from) => {
  if (from && from.path === '/exam' && to.path === '/') {
    alert('Cannot go back during exam!')
    window.history.forward()
  }
}, { immediate: true })
</script>

<style>
@import url('https://fonts.googleapis.com/css2?family=Fraunces:wght@400;600;700&family=Space+Grotesk:wght@400;500;600;700&display=swap');

:root {
  --ink: #0f172a;
  --muted: #64748b;
  --paper: #f8f5f0;
  --surface: #ffffff;
  --accent: #f97316;
  --accent-deep: #1d4ed8;
  --success: #16a34a;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Space Grotesk', sans-serif;
  background: var(--paper);
  color: var(--ink);
}

#app {
  min-height: 100vh;
  background: var(--surface);
}

/* Remove default focus outlines */
*:focus {
  outline: none;
}

*:focus-visible {
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}
</style>
