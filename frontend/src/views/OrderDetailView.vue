<template>
  <v-container class="py-8 max-w-4xl">
    <!-- Back Button & Actions -->
    <div class="d-flex justify-space-between align-center mb-4">
      <v-btn variant="text" prepend-icon="mdi-arrow-left" to="/orders" class="text-none font-weight-bold">
        Back to Orders
      </v-btn>

      <!-- Elevated Refund Action -->
      <v-btn
        v-if="canRefundOrder"
        color="red-accent-2"
        variant="tonal"
        size="small"
        prepend-icon="mdi-cash-refund"
        class="text-none font-weight-bold"
        @click="openRefundDialog()"
      >
        Refund Tickets ({{ activeTickets.length }})
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

      <!-- Seats List -->
      <div class="mb-6">
        <div class="d-flex justify-space-between align-center mb-3">
          <h2 class="text-h6 font-weight-bold text-white">Order Items ({{ order.seats?.length || 0 }})</h2>
          <div v-if="order.refunded_amount_cents > 0" class="text-caption text-red-accent-2 font-weight-bold">
            Refunded: ${{ (order.refunded_amount_cents / 100).toFixed(2) }}
          </div>
        </div>
        <v-table theme="dark" class="rounded-lg border">
          <thead>
            <tr>
              <th class="text-left font-weight-bold">Seat</th>
              <th class="text-left font-weight-bold">Zone Tier</th>
              <th v-if="hasTickets" class="text-left font-weight-bold" style="min-width: 240px;">
                Ticket & QR Pass
              </th>
              <th class="text-right font-weight-bold">Price</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in order.seats" :key="s.seat_booking_id">
              <td class="font-weight-bold text-white py-4 align-middle">
                <v-chip size="small" :color="getZoneColor(s)" variant="flat" class="mr-2 font-weight-bold">
                  {{ s.row_label }}{{ s.seat_number }}
                </v-chip>
                Row {{ s.row_label }}, Seat {{ s.seat_number }}
              </td>
              <td class="align-middle">
                <span class="text-body-2 text-grey-lighten-1">{{ s.zone_name }}</span>
              </td>
              <td v-if="hasTickets" class="py-3 align-middle">
                <div v-if="getTicket(s.seat_booking_id)" class="d-flex align-center gap-3">
                  <!-- QR wrapper -->
                  <div
                    class="qr-cell-wrapper pa-1 rounded bg-white elevation-2 d-flex align-center justify-center"
                    :class="{ 'opacity-40': getTicket(s.seat_booking_id).status === 'refunded' }"
                  >
                    <qrcode-vue :value="getTicket(s.seat_booking_id).code" :size="64" level="H" />
                  </div>
                  <div class="d-flex flex-column flex-grow-1">
                    <div class="d-flex align-center gap-2 mb-1">
                      <span class="text-caption text-grey text-uppercase font-weight-bold">Code</span>
                      <span
                        class="text-subtitle-2 font-weight-black font-weight-mono letter-spacing-code"
                        :class="getTicket(s.seat_booking_id).status === 'refunded' ? 'text-grey text-decoration-line-through' : 'text-amber-darken-1'"
                      >
                        {{ getTicket(s.seat_booking_id).code }}
                      </span>
                    </div>
                    <div class="d-flex align-center gap-2 flex-wrap">
                      <v-chip size="x-small" :color="getTicketStatusColor(getTicket(s.seat_booking_id).status)" variant="flat" class="font-weight-bold text-uppercase">
                        {{ getTicket(s.seat_booking_id).status }}
                      </v-chip>
                      <span v-if="getTicket(s.seat_booking_id).status === 'used'" class="text-caption text-grey text-truncate">
                        Scanned {{ formatDate(getTicket(s.seat_booking_id).used_at) }}
                      </span>
                      <span v-else-if="getTicket(s.seat_booking_id).status === 'refunded'" class="text-caption text-red-accent-2 text-truncate">
                        Refunded {{ formatDate(getTicket(s.seat_booking_id).refunded_at) }}
                      </span>
                      <!-- Individual ticket refund button for staff/organizers -->
                      <v-btn
                        v-else-if="canRefundOrder && getTicket(s.seat_booking_id).status === 'active'"
                        size="x-small"
                        color="red-accent-2"
                        variant="text"
                        class="text-none font-weight-bold px-1"
                        @click="openRefundDialog(getTicket(s.seat_booking_id).id)"
                      >
                        Refund
                      </v-btn>
                    </div>
                  </div>
                </div>
                <span v-else class="text-caption text-grey">Generating ticket...</span>
              </td>
              <td class="text-right font-weight-mono font-weight-bold text-white align-middle">
                <span :class="{ 'text-decoration-line-through text-grey': getTicket(s.seat_booking_id)?.status === 'refunded' }">
                  ${{ (s.price_cents / 100).toFixed(2) }}
                </span>
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
        <div v-if="order.refunded_amount_cents > 0" class="d-flex justify-space-between text-body-2 text-red-accent-2 font-weight-bold">
          <span>Total Refunded:</span>
          <span class="font-weight-mono">-${{ (order.refunded_amount_cents / 100).toFixed(2) }}</span>
        </div>
        <div class="d-flex justify-space-between text-body-2 text-grey">
          <span>Booking & Processing Fee:</span>
          <span class="text-success font-weight-bold">$0.00 (Free)</span>
        </div>
        <v-divider class="my-2"></v-divider>
        <div class="d-flex justify-space-between align-center">
          <span class="text-h6 font-weight-bold text-white">Net Total:</span>
          <span class="text-h4 font-weight-black text-amber-darken-1 font-weight-mono">
            ${{ ((order.total_amount_cents - (order.refunded_amount_cents || 0)) / 100).toFixed(2) }}
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

    <!-- Refund Dialog -->
    <v-dialog v-model="refundDialog.show" max-width="520" theme="dark">
      <v-card theme="dark" class="rounded-xl pa-6 border bg-surface">
        <div class="d-flex align-center gap-3 mb-4">
          <v-avatar color="red-accent-2" variant="tonal" size="44">
            <v-icon icon="mdi-cash-refund" size="24"></v-icon>
          </v-avatar>
          <div>
            <h3 class="text-h6 font-weight-bold text-white">Refund Tickets</h3>
            <span class="text-caption text-grey">Select the active tickets to refund and release back to inventory.</span>
          </div>
        </div>

        <v-divider class="mb-4"></v-divider>

        <!-- Ticket Selection Checkboxes -->
        <div class="mb-4">
          <div class="text-caption text-grey font-weight-bold text-uppercase mb-2">Select Tickets to Refund</div>
          <div class="d-flex flex-column gap-2 max-h-60 overflow-y-auto">
            <v-card
              v-for="t in activeTickets"
              :key="t.id"
              variant="outlined"
              class="pa-3 rounded-lg border bg-surface-variant cursor-pointer"
              @click="toggleTicketSelection(t.id)"
            >
              <div class="d-flex justify-space-between align-center">
                <div class="d-flex align-center gap-3">
                  <v-checkbox-btn
                    :model-value="refundDialog.selectedTicketIds.includes(t.id)"
                    color="red-accent-2"
                    hide-details
                    density="compact"
                    @click.stop="toggleTicketSelection(t.id)"
                  ></v-checkbox-btn>
                  <div>
                    <div class="font-weight-bold text-white text-body-2">
                      Row {{ t.row_label }}, Seat {{ t.seat_number }}
                    </div>
                    <div class="text-caption text-grey">
                      Code: <strong class="font-weight-mono text-amber-darken-1">{{ t.code }}</strong> • {{ t.zone_name }}
                    </div>
                  </div>
                </div>
                <div class="font-weight-mono font-weight-bold text-white">
                  ${{ (t.price_cents / 100).toFixed(2) }}
                </div>
              </div>
            </v-card>
          </div>
        </div>

        <!-- Refund Reason -->
        <v-text-field
          v-model="refundDialog.reason"
          label="Refund Reason (Optional)"
          placeholder="e.g., Customer requested cancellation"
          variant="outlined"
          density="compact"
          class="mb-4"
          hide-details
        ></v-text-field>

        <!-- Calculated Refund Total -->
        <div class="pa-4 rounded-lg bg-surface-variant border mb-6 d-flex justify-space-between align-center">
          <span class="text-body-2 text-grey">Total Refund Amount:</span>
          <span class="text-h6 font-weight-bold text-red-accent-2 font-weight-mono">
            ${{ (calculatedRefundAmount / 100).toFixed(2) }}
          </span>
        </div>

        <div class="d-flex justify-end gap-3">
          <v-btn variant="text" class="text-none font-weight-bold" :disabled="refundDialog.submitting" @click="refundDialog.show = false">
            Cancel
          </v-btn>
          <v-btn
            color="red-accent-2"
            variant="flat"
            class="text-none font-weight-bold px-6"
            :loading="refundDialog.submitting"
            :disabled="refundDialog.selectedTicketIds.length === 0"
            @click="handleRefundSubmit"
          >
            Confirm Refund ({{ refundDialog.selectedTicketIds.length }})
          </v-btn>
        </div>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </v-container>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import QrcodeVue from 'qrcode.vue';
import api from '../api/client';
import { useAuthStore } from '../stores/auth';

const props = defineProps({
  id: { type: String, required: true },
});

const route = useRoute();
const authStore = useAuthStore();
const order = ref(null);
const tickets = ref([]);
const loading = ref(true);
const error = ref(null);
const paying = ref(false);

const refundDialog = ref({
  show: false,
  selectedTicketIds: [],
  reason: '',
  submitting: false,
});

const hasTickets = computed(() => {
  if (!order.value) return false;
  return ['paid', 'partially_refunded', 'refunded'].includes(order.value.status);
});

const ticketMap = computed(() => {
  const map = new Map();
  for (const t of tickets.value) {
    map.set(t.seat_booking_id, t);
  }
  return map;
});

function getTicket(seatBookingId) {
  return ticketMap.value.get(seatBookingId);
}

const activeTickets = computed(() => {
  return tickets.value.filter((t) => t.status === 'active');
});

const canRefundOrder = computed(() => {
  if (!order.value || !authStore.isElevated) return false;
  return (
    (order.value.status === 'paid' || order.value.status === 'partially_refunded') &&
    activeTickets.value.length > 0
  );
});

const calculatedRefundAmount = computed(() => {
  return tickets.value
    .filter((t) => refundDialog.value.selectedTicketIds.includes(t.id))
    .reduce((sum, t) => sum + (t.price_cents || 0), 0);
});

function toggleTicketSelection(ticketId) {
  const idx = refundDialog.value.selectedTicketIds.indexOf(ticketId);
  if (idx > -1) {
    refundDialog.value.selectedTicketIds.splice(idx, 1);
  } else {
    refundDialog.value.selectedTicketIds.push(ticketId);
  }
}

function openRefundDialog(specificTicketId = null) {
  if (specificTicketId) {
    refundDialog.value.selectedTicketIds = [specificTicketId];
  } else {
    refundDialog.value.selectedTicketIds = activeTickets.value.map((t) => t.id);
  }
  refundDialog.value.reason = '';
  refundDialog.value.show = true;
}

async function handleRefundSubmit() {
  if (refundDialog.value.selectedTicketIds.length === 0 || !order.value) return;
  refundDialog.value.submitting = true;
  try {
    const res = await api.post(`/orders/${order.value.id}/refund`, {
      ticket_ids: refundDialog.value.selectedTicketIds,
      reason: refundDialog.value.reason || undefined,
    });
    showToast(`Refund processed successfully ($${(res.data.amount_cents / 100).toFixed(2)})`, 'success');
    refundDialog.value.show = false;
    await fetchOrderDetail();
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Failed to process refund';
    showToast(errMsg, 'error');
  } finally {
    refundDialog.value.submitting = false;
  }
}

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
    case 'partially_refunded': return 'deep-purple-accent-2';
    case 'refunded': return 'grey-darken-1';
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
    case 'refunded': return 'deep-purple-accent-2';
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

    if (['paid', 'partially_refunded', 'refunded'].includes(order.value.status)) {
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

.qr-cell-wrapper {
  background: #ffffff !important;
  width: 72px;
  height: 72px;
  min-width: 72px;
}

.letter-spacing-code {
  letter-spacing: 2px;
}

.align-middle {
  vertical-align: middle !important;
}

.opacity-40 {
  opacity: 0.4;
  filter: grayscale(1);
}

.gap-2 {
  gap: 8px;
}

.gap-3 {
  gap: 12px;
}

.gap-4 {
  gap: 16px;
}
</style>
