<template>
  <v-container class="py-8">
    <OrdersTable :scope="authStore.isElevated ? 'all' : 'own'" />
  </v-container>
</template>

<script setup>
import { useAuthStore } from '../stores/auth';
import OrdersTable from '../components/OrdersTable.vue';

const authStore = useAuthStore();
</script>
