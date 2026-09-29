<template>
  <div class="orders-table-wrapper">
    <!-- Header -->
    <div class="d-flex flex-wrap justify-space-between align-center gap-4 mb-4">
      <div>
        <h2 class="text-h6 font-weight-bold text-white mb-1">
          {{ tableHeaderTitle }}
        </h2>
        <span class="text-caption text-grey">
          {{ tableHeaderSubtitle }}
        </span>
      </div>
      <div class="d-flex align-center gap-2">
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
    </div>

    <!-- Search & Filter Controls Bar -->
    <v-card theme="dark" class="pa-3 mb-4 rounded-xl border bg-surface" elevation="2">
      <div class="d-flex flex-wrap align-center justify-space-between gap-4">
        <!-- Search Input -->
        <v-text-field
          v-model="searchQuery"
          placeholder="Search by Order ID, Event, Customer Email, Showtime, Status..."
          prepend-inner-icon="mdi-magnify"
          variant="outlined"
          density="compact"
          hide-details
          clearable
          class="search-input"
        ></v-text-field>

        <!-- Status Filter Chips -->
        <div class="d-flex align-center gap-2 flex-wrap">
          <v-chip
            size="small"
            :variant="selectedStatus === 'all' ? 'flat' : 'outlined'"
            color="amber-darken-1"
            class="font-weight-bold cursor-pointer"
            @click="selectedStatus = 'all'"
          >
            All ({{ orders.length }})
          </v-chip>
          <v-chip
            size="small"
            :variant="selectedStatus === 'paid' ? 'flat' : 'outlined'"
            color="success"
            class="font-weight-bold cursor-pointer"
            @click="selectedStatus = 'paid'"
          >
            Paid
          </v-chip>
          <v-chip
            size="small"
            :variant="selectedStatus === 'pending_payment' ? 'flat' : 'outlined'"
            color="warning"
            class="font-weight-bold cursor-pointer"
            @click="selectedStatus = 'pending_payment'"
          >
            Pending
          </v-chip>
          <v-chip
            size="small"
            :variant="selectedStatus === 'partially_refunded' ? 'flat' : 'outlined'"
            color="deep-purple-accent-2"
            class="font-weight-bold cursor-pointer"
            @click="selectedStatus = 'partially_refunded'"
          >
            Partially Refunded
          </v-chip>
          <v-chip
            size="small"
            :variant="selectedStatus === 'refunded' ? 'flat' : 'outlined'"
            color="grey-darken-1"
            class="font-weight-bold cursor-pointer"
            @click="selectedStatus = 'refunded'"
          >
            Refunded
          </v-chip>
          <v-chip
            size="small"
            :variant="selectedStatus === 'expired' ? 'flat' : 'outlined'"
            color="red-accent-3"
            class="font-weight-bold cursor-pointer"
            @click="selectedStatus = 'expired'"
          >
            Expired
          </v-chip>
        </div>
      </div>
    </v-card>

    <!-- Empty State -->
    <v-card v-if="!loading && filteredOrders.length === 0" variant="tonal" class="text-center pa-8 rounded-xl border">
      <v-icon icon="mdi-ticket-outline" size="48" color="grey" class="mb-2"></v-icon>
      <div class="text-body-1 text-grey font-weight-medium">
        {{ orders.length === 0 ? 'No orders found' : 'No orders matching your search' }}
      </div>
      <div class="text-caption text-grey-darken-1 mt-1">
        {{ orders.length === 0 ? 'Book some seats from the showtimes page to see them here.' : 'Try adjusting your search query or filters.' }}
      </div>
      <v-btn v-if="orders.length === 0" color="amber-darken-1" variant="flat" to="/" class="mt-4 text-none font-weight-bold">
        Browse Showtimes
      </v-btn>
    </v-card>

    <!-- Orders Table -->
    <div v-else class="table-container rounded-xl border overflow-hidden">
      <v-table theme="dark" class="orders-table">
        <thead>
          <tr>
            <th class="text-left font-weight-bold">Order / Tax Inv No</th>
            <th v-if="scope === 'all'" class="text-left font-weight-bold">Customer</th>
            <th class="text-left font-weight-bold">Event & Theater</th>
            <th class="text-left font-weight-bold">Showtime</th>
            <th class="text-left font-weight-bold">Seats</th>
            <th class="text-left font-weight-bold">Total</th>
            <th class="text-left font-weight-bold">Status</th>
            <th class="text-right font-weight-bold">Action</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="order in paginatedOrders"
            :key="order.id"
            class="clickable-row"
            @click="goToOrderDetail(order.id)"
          >
            <!-- Order ID & Date -->
            <td class="py-3">
              <div class="font-weight-mono text-caption text-amber-darken-1 font-weight-bold">
                {{ order.tax_invoice_number || ('#' + order.id.substring(0, 8)) }}
              </div>
              <div v-if="order.tax_invoice_number" class="text-caption text-grey font-weight-mono">
                #{{ order.id.substring(0, 8) }}
              </div>
              <div class="text-caption text-grey">
                {{ formatDate(order.created_at) }}
              </div>
            </td>

            <!-- Customer (Admin/Staff view) -->
            <td v-if="scope === 'all'">
              <div class="text-body-2 text-white">{{ order.user_email || '—' }}</div>
            </td>

            <!-- Event & Theater -->
            <td>
              <div class="font-weight-bold text-body-2 text-white">{{ order.event_title }}</div>
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
              <div v-if="order.refunded_amount_cents > 0">
                <span class="font-weight-bold font-weight-mono text-white">
                  ${{ ((order.total_amount_cents - order.refunded_amount_cents) / 100).toFixed(2) }}
                </span>
                <div class="text-caption text-grey">
                  <span class="text-decoration-line-through">${{ (order.total_amount_cents / 100).toFixed(2) }}</span>
                  <span class="text-red-accent-2 ml-1">(-${{ (order.refunded_amount_cents / 100).toFixed(2) }})</span>
                </div>
              </div>
              <span v-else class="font-weight-bold font-weight-mono text-white">
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
              <div class="d-flex align-center justify-end gap-2" @click.stop>
                <v-btn
                  v-if="order.status === 'pending_payment'"
                  color="amber-darken-1"
                  size="small"
                  variant="flat"
                  prepend-icon="mdi-credit-card-check"
                  class="text-none font-weight-bold"
                  :loading="payingOrderId === order.id"
                  @click.stop="handlePayNow(order.id)"
                >
                  Pay Now
                </v-btn>
                <v-btn
                  icon="mdi-chevron-right"
                  size="small"
                  variant="text"
                  color="grey"
                  title="View order details"
                  @click.stop="goToOrderDetail(order.id)"
                ></v-btn>
              </div>
            </td>
          </tr>
        </tbody>
      </v-table>

      <!-- Pagination & Limit Footer -->
      <div class="d-flex flex-wrap justify-space-between align-center pa-4 bg-surface border-t gap-4">
        <!-- Range Info -->
        <div class="text-caption text-grey">
          Showing <strong>{{ paginationStart }}</strong>–<strong>{{ paginationEnd }}</strong> of <strong>{{ filteredOrders.length }}</strong> orders
        </div>

        <div class="d-flex align-center gap-4">
          <!-- Limit Selector Dropdown -->
          <div class="d-flex align-center gap-2">
            <span class="text-caption text-grey">Rows per page:</span>
            <v-select
              v-model="itemsPerPage"
              :items="itemsPerPageOptions"
              variant="outlined"
              density="compact"
              hide-details
              style="width: 80px;"
            ></v-select>
          </div>

          <!-- Page Controls -->
          <div class="d-flex align-center gap-1">
            <v-btn
              icon="mdi-chevron-left"
              variant="outlined"
              size="small"
              :disabled="page <= 1"
              @click="page--"
            ></v-btn>
            <span class="text-caption text-grey mx-2 font-weight-mono">
              Page {{ page }} of {{ totalPages || 1 }}
            </span>
            <v-btn
              icon="mdi-chevron-right"
              variant="outlined"
              size="small"
              :disabled="page >= totalPages"
              @click="page++"
            ></v-btn>
          </div>
        </div>
      </div>
    </div>

    <!-- Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import api from '../api/client';

const props = defineProps({
  scope: {
    type: String,
    default: 'own', // 'own' | 'all'
  },
  eventId: {
    type: String,
    default: null,
  },
  eventTitle: {
    type: String,
    default: '',
  },
});

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const tableHeaderTitle = computed(() => {
  if (props.eventTitle) {
    return `Orders for ${props.eventTitle}`;
  }
  if (authStore.role === 'event_organizer') {
    return 'Event Orders (Organizer)';
  }
  if (props.scope === 'all') {
    return 'All System Orders (Admin/Staff)';
  }
  return 'Your Booking History';
});

const tableHeaderSubtitle = computed(() => {
  if (props.eventTitle || props.eventId || route.query.event_id) {
    return 'Viewing customer orders and ticket passes for this event';
  }
  if (authStore.role === 'event_organizer') {
    return 'Monitoring customer orders for your owned events';
  }
  if (props.scope === 'all') {
    return 'Monitoring real-time transactions & seat status across all venues';
  }
  return 'Track and view receipt details for your reserved seats';
});

const orders = ref([]);
const loading = ref(false);
const payingOrderId = ref(null);

const searchQuery = ref('');
const selectedStatus = ref('all');

const page = ref(1);
const itemsPerPage = ref(10);
const itemsPerPageOptions = [5, 10, 25, 50];

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
    hour12: false,
  });
}

function getStatusColor(status) {
  switch (status) {
    case 'paid': return 'success';
    case 'partially_refunded': return 'deep-purple-accent-2';
    case 'refunded': return 'grey-darken-1';
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

function goToOrderDetail(orderId) {
  router.push(`/orders/${orderId}`);
}

// Filtered Orders based on search bar query & status filter
const filteredOrders = computed(() => {
  let list = orders.value;

  if (selectedStatus.value !== 'all') {
    list = list.filter((o) => o.status === selectedStatus.value);
  }

  if (searchQuery.value && searchQuery.value.trim()) {
    const q = searchQuery.value.trim().toLowerCase();
    list = list.filter((o) => {
      const matchId = o.id.toLowerCase().includes(q);
      const matchTaxInv = (o.tax_invoice_number || '').toLowerCase().includes(q);
      const matchEmail = (o.user_email || '').toLowerCase().includes(q);
      const matchEvent = (o.event_title || '').toLowerCase().includes(q);
      const matchTheater = (o.theater_name || '').toLowerCase().includes(q);
      const matchStatus = o.status.toLowerCase().includes(q);
      const matchDate = formatDate(o.starts_at).toLowerCase().includes(q);
      const matchSeats = (o.seats || []).some(
        (s) =>
          `${s.row_label}${s.seat_number}`.toLowerCase().includes(q) ||
          s.zone_name.toLowerCase().includes(q)
      );

      return matchId || matchTaxInv || matchEmail || matchEvent || matchTheater || matchStatus || matchDate || matchSeats;
    });
  }

  return list;
});

// Total pages calculation
const totalPages = computed(() => {
  return Math.ceil(filteredOrders.value.length / itemsPerPage.value) || 1;
});

// Paginated Orders slice
const paginatedOrders = computed(() => {
  const start = (page.value - 1) * itemsPerPage.value;
  return filteredOrders.value.slice(start, start + itemsPerPage.value);
});

const paginationStart = computed(() => {
  if (filteredOrders.value.length === 0) return 0;
  return (page.value - 1) * itemsPerPage.value + 1;
});

const paginationEnd = computed(() => {
  return Math.min(page.value * itemsPerPage.value, filteredOrders.value.length);
});

watch([searchQuery, selectedStatus, itemsPerPage], () => {
  page.value = 1;
});

watch(
  () => [props.eventId, route.query.event_id],
  () => {
    fetchOrders();
  }
);

async function fetchOrders() {
  try {
    loading.value = true;
    const params = {};
    const targetEventId = props.eventId || route.query.event_id;
    if (targetEventId) {
      params.event_id = targetEventId;
    }
    const res = await api.get('/orders', { params });
    orders.value = res.data;
  } catch (err) {
    console.error('Failed to fetch orders', err);
    showToast('Failed to fetch orders', 'error');
  } finally {
    loading.value = false;
  }
}

async function handlePayNow(orderId) {
  try {
    payingOrderId.value = orderId;
    const res = await api.post(`/orders/${orderId}/checkout-session`);
    window.location.href = res.data.checkout_url;
  } catch (err) {
    const errorMsg = err.response?.data?.error || 'Payment failed';
    showToast(errorMsg, 'error');
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

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.search-input {
  min-width: 320px;
  flex-grow: 1;
}

.table-container {
  background: #181c27;
  border-color: #2a3142;
}

.orders-table {
  background: transparent !important;
}

.clickable-row {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.clickable-row:hover {
  background-color: rgba(245, 158, 11, 0.08) !important;
}

.gap-1 {
  gap: 4px;
}

.gap-2 {
  gap: 8px;
}

.gap-4 {
  gap: 16px;
}
</style>
