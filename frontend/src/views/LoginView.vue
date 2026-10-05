<template>
  <v-container class="py-12 d-flex justify-center">
    <v-card theme="dark" max-width="460" width="100%" class="rounded-xl pa-6 border" elevation="6">
      <div class="text-center mb-6">
        <v-icon icon="mdi-shield-lock-outline" color="amber-darken-1" size="40" class="mb-2"></v-icon>
        <h1 class="text-h5 font-weight-bold text-white">Sign In</h1>
        <p class="text-caption text-grey">Access your theater bookings and tickets</p>
      </div>

      <v-form @submit.prevent="handleLogin">
        <v-text-field v-model="email" label="Email Address" type="email" variant="outlined" density="comfortable"
          prepend-inner-icon="mdi-email-outline" class="mb-3" required></v-text-field>

        <v-text-field v-model="password" label="Password" type="password" variant="outlined" density="comfortable"
          prepend-inner-icon="mdi-lock-outline" class="mb-4" required></v-text-field>

        <v-btn type="submit" color="amber-darken-1" variant="flat" block size="large"
          class="font-weight-bold text-none mb-4" :loading="authStore.loading">
          Sign In
        </v-btn>
      </v-form>

      <div class="text-center text-caption text-grey mb-6">
        Don't have an account?
        <router-link to="/register" class="text-amber-darken-1 font-weight-bold text-decoration-none">
          Sign up
        </router-link>
      </div>

      <!-- <v-divider class="my-4"></v-divider> -->

      <!-- Quick Demo Logins -->
      <!-- <div class="mb-2">
        <div class="text-caption text-grey font-weight-bold mb-2 text-center text-uppercase">
          Quick Demo Accounts
        </div>
        <div class="d-flex flex-column gap-2">
          <v-btn v-for="acc in demoAccounts" :key="acc.email" variant="tonal" size="small"
            class="text-none justify-start px-3" @click="quickLogin(acc.email, acc.password)">
            <v-chip size="x-small" :color="acc.color" class="mr-2 font-weight-bold">
              {{ acc.role }}
            </v-chip>
            <span class="text-caption">{{ acc.email }}</span>
          </v-btn>
        </div>
      </div> -->

    </v-card>

    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </v-container>
</template>

<script setup>
import { ref } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const authStore = useAuthStore();
const router = useRouter();
const route = useRoute();

const email = ref('');
const password = ref('');

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

const demoAccounts = [
  { role: 'CUSTOMER', email: 'alice@customer.local', password: 'customer123', color: 'amber-accent-3' },
  { role: 'ADMIN', email: 'admin@theater.local', password: 'admin123', color: 'red-accent-3' },
  { role: 'STAFF', email: 'staff@theater.local', password: 'staff123', color: 'cyan-accent-3' },
  { role: 'ORGANIZER', email: 'organizer@theater.local', password: 'organizer123', color: 'purple-accent-3' },
];

async function handleLogin() {
  const res = await authStore.login(email.value, password.value);
  if (res.success) {
    const redirect = route.query.redirect || '/';
    router.push(redirect);
  } else {
    snackbar.value = { show: true, text: res.error, color: 'error' };
  }
}

async function quickLogin(e, p) {
  email.value = e;
  password.value = p;
  await handleLogin();
}
</script>

<style scoped>
.gap-2 {
  gap: 8px;
}
</style>
