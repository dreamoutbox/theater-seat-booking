<template>
  <div class="orders-table-wrapper">
    <div class="d-flex justify-space-between align-center mb-4">
      <div>
        <h2 class="text-h6 font-weight-bold text-white mb-1">
          {{ scope === 'all' ? 'All System Orders (Admin/Staff)' : 'Your Booking History' }}
        </h2>
        <span class="text-caption text-grey">
          {{ scope === 'all' ? 'Monitoring real-time transactions & seat status' : 'Track and pay for your reserved seats' }}
        </span>
      </div>
      <v-btn
        prepend-icon="mdi-refresh"
        variant="outlined"
        size="small"
        :loading="loading"
        @click="fetchOrders"
      >
        Refresh
      </v-btn>
    </div>

    <!-- Empty state -->
    <v-card v-if="!loading && orders.length === 0" variant="tonal" class="text-center pa-8 rounded-lg">
      <v-icon icon="mdi-ticket-outline" size="48" color="grey" class="mb-2"></v-icon>
      <div class="text-body-1 text-grey font-weight-medium">No orders found</div>
      <div class="text-caption text-grey-darken-1 mt-1">Book some seats from the showtimes page to see them here.</div>
      <v-btn color="amber-darken-1" variant="flat" to="/" class="mt-4 text-none">Browse Showtimes</v-btn>
    </v-card>

    <!-- Orders Table / Cards -->
    <v-table v-else theme="dark" class="rounded-lg border">
      <thead>
        <tr>
          <th class="text-left font-weight-bold">Order ID</th>
          <th v-if="scope === 'all'" class="text-left font-weight-bold">Customer</th>
          <th class="text-left font-weight-bold">Event & Theater</th>
          <th class="text-left font-weight-bold">Showtime</th>
          <th class="text-left font-weight-bold">Seats</th>
          <th class="text-left font-weight-bold">Total</th>
          <th class="text-left font-weight-bold">Status</th>
          <th class="text-right font-weight-bold">Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="order in orders" :key="order.id" class="hover-row">
          <!-- Order ID & Date -->
          <td class="py-3">
            <div class="font-weight-mono text-caption text-amber-darken-1">
              #{{ order.id.substring(0, 8) }}
            </div>
            <div class="text-caption text-grey">
              {{ formatDate(order.created_at) }}
            </div>
          </td>

          <!-- Customer (Admin/Staff view) -->
          <td v-if="scope === 'all'">
            <div class="text-body-2">{{ order.user_email || '—' }}</div>
          </td>

          <!-- Event & Theater -->
          <td>
            <div class="font-weight-bold text-body-2">{{ order.event_title }}</div>
            <div class="text-caption text-grey">{{ order.theater_name }}</div>
          </td>

          <!-- Showtime -->
          <td>
            <div class="text-caption font-weight-medium text-white">
              {{ formatDate(order.starts_at) }}
            </div>
          </td>

          <!-- Seats -->
          <td>
            <div class="d-flex flex-wrap gap-1">
              <v-chip
                v-for="s in order.seats"
                :key="s.seat_booking_id"
                size="x-small"
                variant="outlined"
                color="grey-lighten-1"
                class="font-weight-bold"
              >
                {{ s.row_label }}{{ s.seat_number }} ({{ s.zone_name }})
              </v-chip>
            </div>
          </td>

          <!-- Total Amount -->
          <td>
            <span class="font-weight-bold text-white">
              ${{ (order.total_amount_cents / 100).toFixed(2) }}
            </span>
          </td>

          <!-- Status -->
          <td>
            <v-chip
              size="small"
              :color="getStatusColor(order.status)"
              variant="flat"
              class="font-weight-bold text-uppercase text-caption"
            >
              {{ formatStatus(order.status) }}
            </v-chip>
          </td>

          <!-- Actions -->
          <td class="text-right">
            <v-btn
              v-if="order.status === 'pending_payment'"
              color="amber-darken-1"
              size="small"
              variant="flat"
              prepend-icon="mdi-credit-card-check"
              class="text-none font-weight-bold"
              :loading="payingOrderId === order.id"
              @click="handlePayMock(order.id)"
            >
              Pay Now
            </v-btn>
            <span v-else class="text-caption text-grey">Completed</span>
          </td>
        </tr>
      </tbody>
    </v-table>

    <!-- Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import api from '../api/client';

const props = defineProps({
  scope: {
    type: String,
    default: 'own', // 'own' | 'all'
  },
});

const orders = ref([]);
const loading = ref(false);
const payingOrderId = ref(null);

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

function formatDate(isoStr) {
  if (!isoStr) return '—';
  const d = new Date(isoStr);
  return d.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function getStatusColor(status) {
  switch (status) {
    case 'paid': return 'success';
    case 'pending_payment': return 'warning';
    case 'cancelled': return 'grey';
    case 'expired': return 'red-accent-3';
    default: return 'info';
  }
}

function formatStatus(status) {
  if (!status) return '';
  return status.replace('_', ' ');
}

async function fetchOrders() {
  try {
    loading.value = true;
    const res = await api.get('/orders');
    orders.value = res.data;
  } catch (err) {
    console.error('Failed to fetch orders', err);
    showToast('Failed to fetch orders', 'error');
  } finally {
    loading.value = false;
  }
}

async function handlePayMock(orderId) {
  try {
    payingOrderId.value = orderId;
    await api.post(`/orders/${orderId}/confirm-mock`);
    showToast('Payment successful! Seats confirmed.', 'success');
    await fetchOrders();
  } catch (err) {
    const errorMsg = err.response?.data?.error || 'Payment failed';
    showToast(errorMsg, 'error');
  } finally {
    payingOrderId.value = null;
  }
}

onMounted(() => {
  fetchOrders();
});

defineExpose({
  fetchOrders,
});
</script>

<style scoped>
.orders-table-wrapper {
  max-width: 1200px;
  margin: 0 auto;
}

.hover-row:hover {
  background: rgba(255, 255, 255, 0.03);
}

.gap-1 {
  gap: 4px;
}
</style>
