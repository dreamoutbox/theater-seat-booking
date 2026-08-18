<template>
  <v-container class="py-6">
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
              <v-list-item
                v-for="s in allShowtimes"
                :key="s.id"
                :to="`/booking/${s.id}`"
                :active="s.id === props.showtimeId"
              >
                <v-list-item-title class="text-caption">
                  {{ s.event_title }} — {{ formatDate(s.starts_at) }}
                </v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </div>
      </div>
    </v-card>

    <!-- Seat Map Component -->
    <SeatMap
      v-if="currentShowtime"
      ref="seatMapRef"
      :theater-id="currentShowtime.theater_id"
      :showtime-id="props.showtimeId"
      @checkout="handleOpenCheckout"
    />

    <!-- Checkout Modal Dialog -->
    <v-dialog v-model="checkoutDialog" max-width="500" persistent>
      <v-card theme="dark" class="rounded-xl pa-2">
        <v-card-title class="d-flex justify-space-between align-center text-h6 font-weight-bold">
          <span>Confirm Booking & Payment</span>
          <v-btn icon="mdi-close" variant="text" size="small" @click="checkoutDialog = false"></v-btn>
        </v-card-title>

        <v-card-text>
          <div class="mb-4">
            <div class="text-caption text-grey mb-1">EVENT & SHOWTIME</div>
            <div class="font-weight-bold text-body-1">{{ currentShowtime?.event_title }}</div>
            <div class="text-caption text-grey">{{ currentShowtime?.theater_name }} • {{ formatDate(currentShowtime?.starts_at) }}</div>
          </div>

          <v-divider class="my-3"></v-divider>

          <div class="mb-4">
            <div class="text-caption text-grey mb-2">SELECTED HELD SEATS</div>
            <div class="d-flex flex-column gap-2">
              <div
                v-for="seat in heldSeatsForCheckout"
                :key="seat.id"
                class="d-flex justify-space-between align-center pa-2 rounded bg-grey-darken-4"
              >
                <div>
                  <span class="font-weight-bold mr-2">Seat {{ seat.row_label }}{{ seat.seat_number }}</span>
                  <v-chip size="x-small" variant="tonal" color="amber-lighten-2">{{ seat.zone_name }}</v-chip>
                </div>
                <span class="font-weight-mono font-weight-bold">${{ (seat.price_cents / 100).toFixed(2) }}</span>
              </div>
            </div>
          </div>

          <v-divider class="my-3"></v-divider>

          <div class="d-flex justify-space-between align-center mb-4">
            <span class="text-h6 font-weight-bold">Total Due:</span>
            <span class="text-h5 font-weight-black text-amber-darken-1 font-weight-mono">
              ${{ (totalCheckoutCents / 100).toFixed(2) }}
            </span>
          </div>

          <v-alert type="info" variant="tonal" density="compact" class="text-caption mb-2">
            Stripe payment processing will mark your held seats as confirmed.
          </v-alert>
        </v-card-text>

        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="checkoutDialog = false" :disabled="processingOrder">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="amber-darken-1"
            variant="flat"
            class="text-none font-weight-bold px-6"
            :loading="processingOrder"
            prepend-icon="mdi-credit-card-check"
            @click="handleCreateAndPayOrder"
          >
            Pay & Confirm Booking
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="4000">
      {{ snackbar.text }}
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

const checkoutDialog = ref(false);
const heldSeatsForCheckout = ref([]);
const processingOrder = ref(false);

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

const totalCheckoutCents = computed(() => {
  return heldSeatsForCheckout.value.reduce((sum, s) => sum + s.price_cents, 0);
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
  });
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

function handleOpenCheckout(myHeldSeats) {
  if (!myHeldSeats || myHeldSeats.length === 0) return;
  heldSeatsForCheckout.value = myHeldSeats;
  checkoutDialog.value = true;
}

async function handleCreateAndPayOrder() {
  processingOrder.value = true;
  try {
    // 1. Fetch the seat_booking ids corresponding to these held seats
    // We get them from the active holds query
    const resSeats = await api.get(`/theaters/${currentShowtime.value.theater_id}/seats?showtime_id=${props.showtimeId}`);
    const myHeld = resSeats.data.filter((s) => s.is_mine && s.status === 'held');
    
    if (myHeld.length === 0) {
      showToast('Holds expired. Please select seats again.', 'error');
      checkoutDialog.value = false;
      if (seatMapRef.value) seatMapRef.value.fetchSeats();
      return;
    }

    // 2. Create order: POST /api/orders
    const orderRes = await api.post('/orders', {
      seat_booking_ids: myHeld.map((s) => s.id),
    });

    const orderId = orderRes.data.order_id;
    showToast('Order created! Processing payment...', 'info');

    // 3. Confirm payment via mock endpoint / webhook simulation
    await api.post(`/orders/${orderId}/confirm-mock`);

    showToast('Payment successful! Your seats are confirmed.', 'success');
    checkoutDialog.value = false;
    router.push('/orders');
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Order processing failed';
    showToast(errMsg, 'error');
  } finally {
    processingOrder.value = false;
  }
}

onMounted(() => {
  loadShowtimeData();
});

watch(() => props.showtimeId, () => {
  loadShowtimeData();
});
</script>

<style scoped>
.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.gap-2 {
  gap: 8px;
}

.gap-4 {
  gap: 16px;
}
</style>
