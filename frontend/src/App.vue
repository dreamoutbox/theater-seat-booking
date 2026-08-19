<template>
  <v-app theme="dark" class="app-root">
    <Navbar />
    <v-main class="main-content">
      <router-view />
    </v-main>
  </v-app>
</template>

<script setup>
import { onMounted } from 'vue';
import Navbar from './components/Navbar.vue';
import { useAuthStore } from './stores/auth';

const authStore = useAuthStore();

onMounted(async () => {
  await authStore.fetchMe();
});
</script>

<style>
.app-root {
  background-color: #0d111a !important;
  color: #f1f5f9;
  font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  min-height: 100vh;
}

.main-content {
  background-color: #0d111a;
}

@media print {
  header, nav, .v-app-bar, .v-navigation-drawer, .v-footer, .no-print {
    display: none !important;
  }
  .v-main, .main-content {
    padding: 0 !important;
    margin: 0 !important;
    background: #ffffff !important;
  }
  .app-root {
    background-color: #ffffff !important;
  }
}
</style>
