<template>
  <v-container class="py-12 d-flex justify-center">
    <v-card theme="dark" max-width="460" width="100%" class="rounded-xl pa-6 border" elevation="6">
      <div class="text-center mb-6">
        <v-icon icon="mdi-account-plus-outline" color="amber-darken-1" size="40" class="mb-2"></v-icon>
        <h1 class="text-h5 font-weight-bold text-white">Create Account</h1>
        <p class="text-caption text-grey">Join Horizon Cinema for instant seat reservations</p>
      </div>

      <v-form @submit.prevent="handleRegister">
        <v-text-field
          v-model="email"
          label="Email Address"
          type="email"
          variant="outlined"
          density="comfortable"
          prepend-inner-icon="mdi-email-outline"
          class="mb-3"
          required
        ></v-text-field>

        <v-text-field
          v-model="password"
          label="Password (min 6 characters)"
          type="password"
          variant="outlined"
          density="comfortable"
          prepend-inner-icon="mdi-lock-outline"
          class="mb-3"
          required
        ></v-text-field>

        <v-select
          v-model="role"
          label="Account Role (POC Demo)"
          :items="roleOptions"
          item-title="title"
          item-value="value"
          variant="outlined"
          density="comfortable"
          class="mb-4"
        ></v-select>

        <v-btn
          type="submit"
          color="amber-darken-1"
          variant="flat"
          block
          size="large"
          class="font-weight-bold text-none mb-4"
          :loading="authStore.loading"
        >
          Sign Up
        </v-btn>
      </v-form>

      <div class="text-center text-caption text-grey">
        Already have an account? 
        <router-link to="/login" class="text-amber-darken-1 font-weight-bold text-decoration-none">
          Sign in
        </router-link>
      </div>
    </v-card>

    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </v-container>
</template>

<script setup>
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const authStore = useAuthStore();
const router = useRouter();

const email = ref('');
const password = ref('');
const role = ref('customer');

const roleOptions = [
  { title: 'Customer', value: 'customer' },
  { title: 'Theater Staff', value: 'staff' },
  { title: 'Event Organizer', value: 'event_organizer' },
  { title: 'Administrator', value: 'admin' },
];

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

async function handleRegister() {
  const res = await authStore.register(email.value, password.value, role.value);
  if (res.success) {
    router.push('/');
  } else {
    snackbar.value = { show: true, text: res.error, color: 'error' };
  }
}
</script>
