<template>
  <v-container fluid class="py-6 px-6">
    <!-- Showtime Top Bar -->
    <v-card v-if="currentShowtime" class="pa-4 mb-6 rounded-xl border bg-surface" elevation="2">
      <div class="d-flex flex-wrap justify-space-between align-center gap-4">
        <div>
          <div class="d-flex align-center gap-2 mb-1">
            <v-btn icon="mdi-arrow-left" variant="text" size="small" to="/"></v-btn>
            <h1 class="text-h5 font-weight-bold text-white mb-0">
              {{ currentShowtime.event_title }}
            </h1>
          </div>
          <div class="d-flex align-center text-caption text-grey ml-8">
            <v-icon icon="mdi-map-marker" size="14" class="mr-1"></v-icon>
            <span>{{ currentShowtime.theater_name }}</span>
            <span class="mx-2">•</span>
            <v-icon icon="mdi-clock-outline" size="14" class="mr-1"></v-icon>
            <span>{{ formatDate(currentShowtime.starts_at) }}</span>
          </div>
        </div>

        <div class="d-flex align-center gap-2">
          <!-- Showtime Switcher Menu -->
          <v-menu>
            <template #activator="{ props }">
              <v-btn v-bind="props" variant="outlined" size="small" prepend-icon="mdi-swap-horizontal">
                Switch Showtime
              </v-btn>
            </template>
            <v-list density="compact" bg-color="#1e222d">
              <v-list-item v-for="s in allShowtimes" :key="s.id" :to="`/booking/${s.id}`"
                :active="s.id === props.showtimeId">
                <v-list-item-title class="text-caption">
                  {{ s.event_title }} — {{ formatDate(s.starts_at) }}
                </v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </div>
      </div>
    </v-card>

    <!-- Main Grid: Left = SeatMap, Right = Reservation Summary & Checkout -->
    <v-row v-if="currentShowtime">
      <!-- Left Column: Seat Map (Fills Width Screen) -->
      <v-col cols="12" md="8" lg="9" xl="9">
        <SeatMap ref="seatMapRef" :theater-id="currentShowtime.theater_id" :showtime-id="props.showtimeId"
          @held-seats-changed="onHeldSeatsChanged" />
      </v-col>

      <!-- Right Column: Order Summary & Selected Held Seats -->
      <v-col cols="12" md="4" lg="3" xl="3">
        <v-card theme="dark" class="rounded-xl pa-5 border bg-surface h-100 d-flex flex-column justify-space-between"
          elevation="4">
          <div>
            <div class="d-flex justify-space-between align-center mb-3">
              <h2 class="text-h6 font-weight-bold text-white mb-0">Your Reservation</h2>
              <!-- Timer Badge -->
              <v-chip v-if="myHeldSeats.length > 0 && countdownSeconds > 0" size="small" color="warning" variant="flat"
                class="font-weight-bold font-weight-mono">
                <v-icon start icon="mdi-timer-outline" size="14"></v-icon>
                {{ formattedCountdown }}
              </v-chip>
            </div>

            <div class="text-caption text-grey mb-4">
              {{ currentShowtime.event_title }} • {{ formatDate(currentShowtime.starts_at) }}
            </div>

            <v-divider class="mb-4"></v-divider>

            <!-- SELECTED HELD SEATS -->
            <div class="mb-4">
              <div class="d-flex justify-space-between align-center mb-2">
                <span class="text-caption text-grey font-weight-bold text-uppercase">
                  SELECTED HELD SEATS ({{ myHeldSeats.length }})
                </span>
                <v-btn v-if="myHeldSeats.length > 1 && !pendingOrder" variant="text" color="red-lighten-2"
                  size="x-small" class="text-none px-1" :disabled="cancellingSeatId !== null"
                  @click="handleCancelAllSeats">
                  Cancel All
                </v-btn>
              </div>

              <!-- Empty State -->
              <div v-if="myHeldSeats.length === 0"
                class="text-center pa-6 rounded-lg bg-surface-variant border border-dashed">
                <v-icon icon="mdi-seat-outline" size="36" color="grey" class="mb-2"></v-icon>
                <div class="text-caption text-grey">No seats selected yet</div>
                <div class="text-caption text-grey-darken-1 mt-1">Click on any available seat on the map to hold it.
                </div>
              </div>

              <!-- List of Selected Held Seats -->
              <div v-else class="d-flex flex-column gap-2">
                <!-- Pending order banner -->
                <v-alert v-if="pendingOrder" type="warning" variant="tonal" density="compact" icon="mdi-lock-outline"
                  class="mb-2 text-caption">
                  Payment in progress. Seats are locked.
                  <v-btn :to="`/orders/${pendingOrder.id}`" size="x-small" variant="text" color="amber-darken-1"
                    class="ml-1 text-none font-weight-bold">View Order →</v-btn>
                </v-alert>

                <div v-for="seat in myHeldSeats" :key="seat.id"
                  class="d-flex justify-space-between align-center pa-3 rounded-lg bg-surface-variant border">
                  <div class="d-flex align-center gap-2">
                    <v-chip size="x-small" :color="getZoneColor(seat)" variant="flat" class="font-weight-bold">
                      {{ seat.row_label }}{{ seat.seat_number }}
                    </v-chip>
                    <div>
                      <div class="text-caption font-weight-bold text-white">
                        Row {{ seat.row_label }}, Seat {{ seat.seat_number }}
                      </div>
                      <div class="text-caption text-grey">{{ seat.zone_name }}</div>
                    </div>
                  </div>

                  <div class="d-flex align-center gap-2">
                    <span class="font-weight-mono font-weight-bold text-body-2 text-white">
                      ${{ (seat.price_cents / 100).toFixed(2) }}
                    </span>
                    <!-- Cancel button hidden when order is pending -->
                    <v-btn v-if="!pendingOrder" icon="mdi-close" size="x-small" color="red-lighten-2" variant="text"
                      title="Cancel booking for this seat" :loading="cancellingSeatId === seat.id"
                      @click="handleCancelSeat(seat)"></v-btn>
                    <v-icon v-else icon="mdi-lock" size="16" color="warning"
                      title="Locked — payment in progress"></v-icon>
                  </div>
                </div>
              </div>
            </div>

            <!-- Price Breakdown -->
            <template v-if="myHeldSeats.length > 0">
              <v-divider class="my-4"></v-divider>

              <div class="d-flex justify-space-between align-center mb-2 text-caption text-grey">
                <span>Seats Subtotal ({{ myHeldSeats.length }}):</span>
                <span class="font-weight-mono">${{ (totalCheckoutCents / 100).toFixed(2) }}</span>
              </div>
              <div class="d-flex justify-space-between align-center mb-3 text-caption text-grey">
                <span>Processing & Fees:</span>
                <span class="text-success font-weight-bold">$0.00 (Included)</span>
              </div>

              <v-divider class="my-3"></v-divider>

              <div class="d-flex justify-space-between align-center mb-4">
                <span class="text-body-1 font-weight-bold text-white">Total Due:</span>
                <span class="text-h5 font-weight-black text-amber-darken-1 font-weight-mono">
                  ${{ (totalCheckoutCents / 100).toFixed(2) }}
                </span>
              </div>
            </template>
          </div>

          <!-- Actions -->
          <div class="mt-4">
            <!-- When order is pending: show "Complete Payment" button -->
            <v-btn v-if="pendingOrder" color="amber-darken-1" variant="flat" block size="large"
              class="text-none font-weight-bold" prepend-icon="mdi-credit-card-check"
              :to="`/orders/${pendingOrder.id}`">
              Complete Payment
            </v-btn>
            <v-btn v-else color="amber-darken-1" variant="flat" block size="large" class="text-none font-weight-bold"
              :disabled="myHeldSeats.length === 0" prepend-icon="mdi-cart-arrow-right" @click="goToCheckout">
              {{ myHeldSeats.length === 0 ? 'Select Seats to Book' : `Proceed to Checkout ($${(totalCheckoutCents /
                100).toFixed(2)})` }}
            </v-btn>
            <div class="text-caption text-grey text-center mt-2">
              🔒 Instant lock reservation & transactional guarantee
            </div>
          </div>
        </v-card>
      </v-col>
    </v-row>

    <!-- Global Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="4000" location="bottom right">
      {{ snackbar.text }}
      <template #actions>
        <v-btn variant="text" @click="snackbar.show = false">Close</v-btn>
      </template>
    </v-snackbar>
  </v-container>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import api from '../api/client';
import SeatMap from '../components/SeatMap.vue';

const props = defineProps({
  showtimeId: { type: String, required: true },
});

const router = useRouter();
const currentShowtime = ref(null);
const allShowtimes = ref([]);
const seatMapRef = ref(null);

const myHeldSeats = ref([]);
const countdownSeconds = ref(0);
const cancellingSeatId = ref(null);
const processingOrder = ref(false);
const pendingOrder = ref(null); // non-null when user has a pending_payment order for this showtime

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

const totalCheckoutCents = computed(() => {
  return myHeldSeats.value.reduce((sum, s) => sum + s.price_cents, 0);
});

const formattedCountdown = computed(() => {
  const m = Math.floor(countdownSeconds.value / 60);
  const s = countdownSeconds.value % 60;
  return `${m}:${s < 10 ? '0' : ''}${s}`;
});

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

function onHeldSeatsChanged(payload) {
  if (Array.isArray(payload)) {
    myHeldSeats.value = payload;
  } else if (payload && Array.isArray(payload.heldSeats)) {
    myHeldSeats.value = payload.heldSeats;
    countdownSeconds.value = payload.countdownSeconds || 0;
  } else {
    myHeldSeats.value = [];
  }
}

async function loadShowtimeData() {
  try {
    const res = await api.get('/showtimes');
    allShowtimes.value = res.data;
    currentShowtime.value = allShowtimes.value.find((s) => s.id === props.showtimeId);
  } catch (err) {
    console.error('Failed to load showtime', err);
  }
}

async function checkPendingOrder() {
  try {
    const res = await api.get('/orders');
    const orders = res.data || [];
    // Find any pending_payment order for this showtime
    const found = orders.find(
      (o) => o.showtime_id === props.showtimeId && o.status === 'pending_payment'
    );
    pendingOrder.value = found || null;
  } catch (err) {
    // Not critical — silently ignore
    pendingOrder.value = null;
  }
}

// Cancel individual held seat
async function handleCancelSeat(seat) {
  cancellingSeatId.value = seat.id;
  try {
    await api.post('/bookings/release', {
      showtime_id: props.showtimeId,
      seat_id: seat.id,
    });
    showToast(`Cancelled hold for Seat ${seat.row_label}${seat.seat_number}`, 'info');
    if (seatMapRef.value) {
      await seatMapRef.value.fetchSeats();
    }
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Failed to cancel hold';
    showToast(errMsg, 'error');
  } finally {
    cancellingSeatId.value = null;
  }
}

// Cancel all held seats
async function handleCancelAllSeats() {
  const seatsToCancel = [...myHeldSeats.value];
  cancellingSeatId.value = 'all';
  try {
    for (const s of seatsToCancel) {
      await api.post('/bookings/release', {
        showtime_id: props.showtimeId,
        seat_id: s.id,
      });
    }
    showToast('All seat holds cancelled', 'info');
    if (seatMapRef.value) {
      await seatMapRef.value.fetchSeats();
    }
  } catch (err) {
    showToast('Failed to cancel all seat holds', 'error');
  } finally {
    cancellingSeatId.value = null;
  }
}

function goToCheckout() {
  if (myHeldSeats.value.length === 0) return;
  router.push(`/checkout/${props.showtimeId}`);
}

onMounted(() => {
  loadShowtimeData();
  checkPendingOrder();
});

watch(() => props.showtimeId, () => {
  loadShowtimeData();
  checkPendingOrder();
});
</script>

<style scoped>
.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.gap-2 {
  gap: 8px;
}

.gap-4 {
  gap: 16px;
}
</style>
