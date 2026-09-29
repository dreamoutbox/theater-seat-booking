<template>
  <v-app-bar color="#131722" density="comfortable" elevation="3" class="px-4 border-b">
    <v-toolbar-title class="d-flex align-center cursor-pointer font-weight-bold" @click="$router.push('/')">
      <v-icon icon="mdi-theater" color="amber-darken-1" class="mr-2" size="28"></v-icon>
      <span class="text-white">DREAM<span class="text-amber-darken-1 font-weight-light">CINEMA</span></span>
    </v-toolbar-title>

    <v-spacer></v-spacer>

    <v-btn variant="text" prepend-icon="mdi-filmstrip" to="/" class="text-none font-weight-medium">
      Showtimes
    </v-btn>

    <v-btn v-if="authStore.isStaff" variant="text" prepend-icon="mdi-theater" to="/manage/theaters"
      class="text-none font-weight-medium text-amber-darken-1">
      Manage Theaters
    </v-btn>

    <v-btn v-if="authStore.isElevated" variant="text" prepend-icon="mdi-movie-edit" to="/manage/events"
      class="text-none font-weight-medium text-amber-darken-1">
      Manage Events
    </v-btn>

    <v-btn v-if="authStore.isGatekeeper || authStore.isElevated" variant="text" prepend-icon="mdi-qrcode-scan"
      to="/scan" class="text-none font-weight-medium text-emerald-lighten-2">
      Scan Passes
    </v-btn>

    <v-btn v-if="authStore.isAuthenticated && !authStore.isElevated && !authStore.isGatekeeper" variant="text"
      prepend-icon="mdi-ticket-confirmation" to="/orders" class="text-none font-weight-medium">
      My Orders
    </v-btn>

    <v-divider vertical class="mx-3 my-2" v-if="authStore.isAuthenticated"></v-divider>

    <!-- Demo Role Switcher Menu -->
    <!-- <v-menu location="bottom end">
      <template #activator="{ props }">
        <v-btn v-bind="props" variant="outlined" color="amber-darken-1" size="small" prepend-icon="mdi-account-switch"
          class="text-none mr-2">
          Demo Switcher
        </v-btn>
      </template>
<v-list density="compact" bg-color="#1e222d" class="pa-1">
  <v-list-subheader class="text-caption text-grey">SWITCH DEMO ACCOUNT</v-list-subheader>
  <v-list-item v-for="acc in demoAccounts" :key="acc.email" @click="quickLogin(acc.email, acc.password)"
    class="rounded mb-1" :active="authStore.user?.email === acc.email">
    <template #prepend>
            <v-chip size="x-small" :color="getRoleColor(acc.role)" class="mr-2 font-weight-bold">
              {{ acc.role.toUpperCase() }}
            </v-chip>
          </template>
    <v-list-item-title class="text-body-2">{{ acc.label }}</v-list-item-title>
    <v-list-item-subtitle class="text-caption text-grey">{{ acc.email }}</v-list-item-subtitle>
  </v-list-item>
</v-list>
</v-menu> -->

    <!-- User Profile or Login -->
    <template v-if="authStore.isAuthenticated">
      <v-chip size="small" :color="getRoleColor(authStore.role)" variant="tonal"
        class="mr-2 text-capitalize font-weight-bold">
        <v-icon start icon="mdi-shield-account" size="14"></v-icon>
        {{ formatRole(authStore.role) }}
      </v-chip>

      <span class="text-caption text-grey mr-3 d-none d-sm-inline">
        {{ authStore.user?.email }}
      </span>

      <v-btn variant="text" icon="mdi-logout" size="small" color="red-lighten-2" @click="handleLogout"
        title="Logout"></v-btn>
    </template>
    <template v-else>
      <v-btn variant="text" to="/login" class="text-none mr-2">
        Login
      </v-btn>
      <v-btn color="amber-darken-1" variant="flat" to="/register" class="text-none font-weight-bold">
        Sign Up
      </v-btn>
    </template>
  </v-app-bar>
</template>

<script setup>
import { useAuthStore } from '../stores/auth';
import { useRouter } from 'vue-router';

const authStore = useAuthStore();
const router = useRouter();

const demoAccounts = [
  { label: 'Admin', email: 'admin@theater.local', password: 'admin123', role: 'admin' },
  { label: 'Staff', email: 'staff@theater.local', password: 'staff123', role: 'staff' },
  { label: 'Organizer', email: 'organizer@theater.local', password: 'organizer123', role: 'event_organizer' },
  { label: 'Gatekeeper', email: 'gatekeeper@theater.local', password: 'gatekeeper123', role: 'gatekeeper' },
  { label: 'Customer 1 (Alice)', email: 'alice@customer.local', password: 'customer123', role: 'customer' },
  { label: 'Customer 2 (Bob)', email: 'bob@customer.local', password: 'customer123', role: 'customer' },
];

function formatRole(role) {
  if (!role) return '';
  return role.replace('_', ' ');
}

function getRoleColor(role) {
  switch (role) {
    case 'admin': return 'red-accent-3';
    case 'staff': return 'cyan-accent-3';
    case 'event_organizer': return 'purple-accent-3';
    case 'gatekeeper': return 'teal-accent-3';
    case 'customer': return 'amber-accent-3';
    default: return 'grey';
  }
}

async function quickLogin(email, password) {
  const res = await authStore.login(email, password);
  if (res.success && res.user?.role === 'gatekeeper') {
    router.push('/scan');
  }
}

async function handleLogout() {
  await authStore.logout();
  router.push('/login');
}
</script>

<style scoped>
.cursor-pointer {
  cursor: pointer;
}

.text-emerald-lighten-2 {
  color: #34d399 !important;
}
</style>
