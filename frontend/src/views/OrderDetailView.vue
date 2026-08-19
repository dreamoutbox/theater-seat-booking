<template>
  <v-container class="py-8 max-w-4xl">
    <!-- Back Button -->
    <div class="mb-4">
      <v-btn variant="text" prepend-icon="mdi-arrow-left" to="/orders" class="text-none font-weight-bold">
        Back to Orders
      </v-btn>
    </div>

    <!-- Loading Skeleton -->
    <v-skeleton-loader v-if="loading" type="article, table, actions" theme="dark"
      class="rounded-xl"></v-skeleton-loader>

    <!-- Error State -->
    <v-alert v-else-if="error" type="error" variant="tonal" class="rounded-xl mb-6">
      {{ error }}
    </v-alert>

    <!-- Order Detail Card -->
    <v-card v-else-if="order" theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="4">
      <!-- Order Header -->
      <div class="d-flex flex-wrap justify-space-between align-start gap-4 mb-6">
        <div>
          <div class="d-flex align-center gap-2 mb-2">
            <span class="text-caption text-grey font-weight-mono text-uppercase">Order Reference</span>
            <span class="text-caption font-weight-mono text-amber-darken-1 font-weight-bold">#{{ order.id }}</span>
          </div>
          <div class="d-flex align-center gap-5 flex-wrap mb-2">
            <h1 class="text-h4 font-weight-bold text-white mb-0 mr-5">
              {{ order.event_title }}
            </h1>
            <v-chip size="small" :color="getStatusColor(order.status)" variant="flat"
              class="font-weight-bold text-uppercase">
              {{ formatStatus(order.status) }}
            </v-chip>
          </div>
          <div class="d-flex align-center text-body-2 text-grey">
            <v-icon icon="mdi-map-marker" size="16" class="mr-1"></v-icon>
            <span>{{ order.theater_name }}</span>
            <span class="mx-2">•</span>
            <v-icon icon="mdi-calendar-clock" size="16" class="mr-1"></v-icon>
            <span>{{ formatDate(order.starts_at) }}</span>
          </div>
        </div>

        <div class="d-flex flex-column align-end">
          <span class="text-caption text-grey">Placed on {{ formatDate(order.created_at) }}</span>
        </div>
      </div>

      <v-divider class="mb-6"></v-divider>

      <!-- Customer / Buyer Info -->
      <div class="row-info mb-6 pa-4 rounded-lg bg-surface-variant border">
        <div class="d-flex flex-wrap justify-space-between align-center gap-4">
          <div>
            <div class="text-caption text-grey font-weight-bold text-uppercase mb-1">Account / Buyer</div>
            <div class="text-body-1 font-weight-medium text-white">{{ order.user_email || 'You' }}</div>
          </div>
          <div>
            <div class="text-caption text-grey font-weight-bold text-uppercase mb-1">Stripe Reference</div>
            <div class="text-caption font-weight-mono text-grey-lighten-1">
              {{ order.stripe_payment_intent_id || 'N/A' }}
            </div>
          </div>
        </div>
      </div>

      <!-- Tickets & QR Pass (Visible when Paid) -->
      <div v-if="order.status === 'paid'" class="mb-8">
        <div class="d-flex align-center justify-space-between mb-4">
          <div>
            <h2 class="text-h5 font-weight-bold text-white mb-1 d-flex align-center gap-2">
              <v-icon icon="mdi-qrcode-scan" color="amber-darken-1" size="28"></v-icon>
              Digital Tickets & QR Entry Passes
            </h2>
            <div class="text-caption text-grey">Show this 6-digit code or QR code at the entrance for gatekeeper verification.</div>
          </div>
          <v-chip color="emerald" variant="tonal" size="small" class="font-weight-bold">
            {{ tickets.length }} {{ tickets.length === 1 ? 'Pass' : 'Passes' }} Ready
          </v-chip>
        </div>

        <v-row dense>
          <v-col v-for="t in tickets" :key="t.id" cols="12" sm="6" md="6" class="pa-2">
            <v-card theme="dark" class="ticket-pass-card rounded-xl pa-4 border bg-surface-variant h-100 d-flex flex-column justify-space-between">
              <!-- Pass Header -->
              <div class="d-flex justify-space-between align-start mb-3">
                <div>
                  <div class="text-caption text-grey text-uppercase font-weight-bold">Seat Assignment</div>
                  <div class="text-h6 font-weight-black text-white">
                    Row {{ t.row_label }}, Seat {{ t.seat_number }}
                  </div>
                  <v-chip size="x-small" :color="getZoneColor(t)" variant="flat" class="font-weight-bold mt-1">
                    {{ t.zone_name }} • ${{ (t.price_cents / 100).toFixed(2) }}
                  </v-chip>
                </div>
                <v-chip size="small" :color="getTicketStatusColor(t.status)" variant="flat" class="font-weight-bold text-uppercase">
                  {{ t.status }}
                </v-chip>
              </div>

              <!-- Pass QR Center -->
              <div class="qr-box my-3 pa-3 rounded-lg d-flex flex-column align-center justify-center bg-grey-darken-4 border">
                <div class="qr-wrapper pa-2 rounded-lg bg-white elevation-2">
                  <qrcode-vue :value="t.code" :size="130" level="H" />
                </div>
                <div class="text-caption text-grey mt-2 font-weight-medium">Ticket Code</div>
                <div class="text-h5 font-weight-black font-weight-mono text-amber-darken-1 letter-spacing-wide">
                  {{ t.code }}
                </div>
              </div>

              <!-- Pass Footer -->
              <div class="text-caption text-grey text-center pt-2 border-top">
                <template v-if="t.status === 'used'">
                  <span class="text-grey font-weight-bold">✓ Scanned & Used at {{ formatDate(t.used_at) }}</span>
                </template>
                <template v-else>
                  <span>Valid for event admission</span>
                </template>
              </div>
            </v-card>
          </v-col>
        </v-row>
      </div>

      <!-- Seats List -->
      <div class="mb-6">
        <h2 class="text-h6 font-weight-bold text-white mb-3">Order Items ({{ order.seats?.length || 0 }})</h2>
        <v-table theme="dark" class="rounded-lg border">
          <thead>
            <tr>
              <th class="text-left font-weight-bold">Seat</th>
              <th class="text-left font-weight-bold">Zone Tier</th>
              <th class="text-right font-weight-bold">Price</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in order.seats" :key="s.seat_booking_id">
              <td class="font-weight-bold text-white py-3">
                <v-chip size="small" :color="getZoneColor(s)" variant="flat" class="mr-2 font-weight-bold">
                  {{ s.row_label }}{{ s.seat_number }}
                </v-chip>
                Row {{ s.row_label }}, Seat {{ s.seat_number }}
              </td>
              <td>
                <span class="text-body-2 text-grey-lighten-1">{{ s.zone_name }}</span>
              </td>
              <td class="text-right font-weight-mono font-weight-bold text-white">
                ${{ (s.price_cents / 100).toFixed(2) }}
              </td>
            </tr>
          </tbody>
        </v-table>
      </div>

      <v-divider class="mb-6"></v-divider>

      <!-- Financial Summary -->
      <div class="d-flex flex-column gap-2 max-w-sm ml-auto mb-6">
        <div class="d-flex justify-space-between text-body-2 text-grey">
          <span>Seats Subtotal:</span>
          <span class="font-weight-mono text-white">${{ (order.total_amount_cents / 100).toFixed(2) }}</span>
        </div>
        <div class="d-flex justify-space-between text-body-2 text-grey">
          <span>Booking & Processing Fee:</span>
          <span class="text-success font-weight-bold">$0.00 (Free)</span>
        </div>
        <v-divider class="my-2"></v-divider>
        <div class="d-flex justify-space-between align-center">
          <span class="text-h6 font-weight-bold text-white">Total Amount:</span>
          <span class="text-h4 font-weight-black text-amber-darken-1 font-weight-mono">
            ${{ (order.total_amount_cents / 100).toFixed(2) }}
          </span>
        </div>
      </div>

      <!-- Pending Payment Action -->
      <div v-if="order.status === 'pending_payment'"
        class="pa-4 rounded-xl border bg-surface-variant d-flex justify-space-between align-center">
        <div>
          <div class="font-weight-bold text-white mb-1">Awaiting Payment</div>
          <div class="text-caption text-grey">Complete checkout to permanently confirm your reserved seats.</div>
        </div>
        <v-btn color="amber-darken-1" variant="flat" size="large" prepend-icon="mdi-credit-card-check"
          class="text-none font-weight-bold px-6" :loading="paying" @click="handlePayMock">
          Pay Now (${{ (order.total_amount_cents / 100).toFixed(2) }})
        </v-btn>
      </div>
    </v-card>

    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import QrcodeVue from 'qrcode.vue';
import api from '../api/client';

const props = defineProps({
  id: { type: String, required: true },
});

const route = useRoute();
const order = ref(null);
const tickets = ref([]);
const loading = ref(true);
const error = ref(null);
const paying = ref(false);

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
    weekday: 'short',
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
    case 'pending_payment': return 'warning';
    case 'cancelled': return 'grey';
    case 'expired': return 'red-accent-3';
    default: return 'info';
  }
}

function getTicketStatusColor(status) {
  switch (status) {
    case 'active': return 'success';
    case 'used': return 'grey-darken-1';
    case 'cancelled': return 'red';
    case 'expired': return 'amber-darken-2';
    default: return 'info';
  }
}

function formatStatus(status) {
  if (!status) return '';
  return status.replace('_', ' ');
}

function getZoneColor(seatOrName) {
  if (!seatOrName) return '#14b8a6';
  if (typeof seatOrName === 'object') {
    if (seatOrName.zone_color) return seatOrName.zone_color;
    seatOrName = seatOrName.zone_name;
  }
  if (!seatOrName) return '#14b8a6';
  if (seatOrName.toLowerCase().includes('vip')) return '#f59e0b';
  if (seatOrName.toLowerCase().includes('premium')) return '#6366f1';
  return '#14b8a6';
}

async function fetchOrderDetail() {
  const orderId = props.id || route.params.id;
  loading.value = true;
  error.value = null;
  try {
    const res = await api.get(`/orders/${orderId}`);
    order.value = res.data;

    if (order.value.status === 'paid') {
      await fetchTickets(orderId);
    }
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load order details';
  } finally {
    loading.value = false;
  }
}

async function fetchTickets(orderId) {
  try {
    const res = await api.get(`/orders/${orderId}/tickets`);
    tickets.value = res.data || [];
  } catch (err) {
    console.error('Failed to load tickets', err);
  }
}

async function handlePayMock() {
  if (!order.value) return;
  paying.value = true;
  try {
    await api.post(`/orders/${order.value.id}/confirm-mock`);
    showToast('Payment successful! Tickets generated.', 'success');
    await fetchOrderDetail();
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Payment failed';
    showToast(errMsg, 'error');
  } finally {
    paying.value = false;
  }
}

onMounted(() => {
  fetchOrderDetail();
});
</script>

<style scoped>
.max-w-4xl {
  max-width: 900px;
}

.max-w-sm {
  max-width: 350px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.ticket-pass-card {
  border: 1px solid #333d52 !important;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.ticket-pass-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.4);
}

.letter-spacing-wide {
  letter-spacing: 4px;
}

.border-top {
  border-top: 1px dashed #2a3142;
}

.gap-2 {
  gap: 8px;
}

.gap-4 {
  gap: 16px;
}
</style>
