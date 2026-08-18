<template>
  <div class="seat-map-container pa-4">
    <!-- Cinema Screen visual -->
    <div class="screen-container text-center mb-8">
      <div class="screen-curve mx-auto mb-2"></div>
      <span class="text-caption text-uppercase tracking-widest text-grey-darken-1 font-weight-bold">
        Cinema Screen
      </span>
    </div>

    <!-- Seating Legend -->
    <div class="d-flex flex-wrap justify-center align-center gap-4 mb-6 text-caption">
      <div class="d-flex align-center mr-4">
        <span class="legend-box zone-vip mr-1"></span>
        <span>VIP ($25.00)</span>
      </div>
      <div class="d-flex align-center mr-4">
        <span class="legend-box zone-premium mr-1"></span>
        <span>Premium ($18.00)</span>
      </div>
      <div class="d-flex align-center mr-4">
        <span class="legend-box zone-standard mr-1"></span>
        <span>Standard ($12.00)</span>
      </div>
      <div class="d-flex align-center mr-4">
        <span class="legend-box seat-held-mine mr-1"></span>
        <span>Your Hold (Click to cancel)</span>
      </div>
      <div class="d-flex align-center mr-4">
        <span class="legend-box seat-processing mr-1"></span>
        <span>In Queue</span>
      </div>
      <div class="d-flex align-center">
        <span class="legend-box seat-taken mr-1"></span>
        <span>Unavailable</span>
      </div>
    </div>

    <!-- In-flight Booking Queue Alert -->
    <v-alert
      v-if="inFlightBookings.length > 0"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-6 mx-auto max-w-xl"
    >
      <div class="d-flex align-center">
        <v-progress-circular indeterminate size="18" width="2" color="info" class="mr-3"></v-progress-circular>
        <span>Processing seat reservation in queue... (Per-seat advisory lock ordering)</span>
      </div>
    </v-alert>

    <!-- Seating Grid -->
    <div class="seating-grid mx-auto">
      <div v-for="row in groupedRows" :key="row.label" class="seat-row d-flex align-center justify-center mb-3">
        <!-- Row Label -->
        <span class="row-label mr-4 text-body-2 font-weight-bold text-grey">{{ row.label }}</span>

        <!-- Seats in Row -->
        <div class="d-flex gap-2">
          <template v-for="slot in row.slots" :key="`${row.label}-${slot.seatNumber}`">
            <button
              v-if="slot.seat"
              class="seat-btn"
              :class="getSeatClasses(slot.seat)"
              :disabled="isSeatDisabled(slot.seat)"
              @click="handleSeatClick(slot.seat)"
              :title="getSeatTooltip(slot.seat)"
            >
              <v-progress-circular
                v-if="isSeatProcessing(slot.seat.id)"
                indeterminate
                size="14"
                width="2"
                color="white"
              ></v-progress-circular>
              <v-icon
                v-else-if="slot.seat.status === 'confirmed' || (slot.seat.status === 'held' && !slot.seat.is_mine)"
                icon="mdi-lock"
                size="12"
              ></v-icon>
              <template v-else-if="slot.seat.is_mine && slot.seat.status === 'held'">
                <v-icon
                  icon="mdi-check"
                  size="14"
                  class="seat-icon-check"
                ></v-icon>
                <v-icon
                  icon="mdi-close"
                  size="14"
                  class="seat-icon-cancel"
                ></v-icon>
              </template>
              <span v-else class="seat-num">{{ slot.seat.seat_number }}</span>
            </button>
            <div v-else class="seat-spacer"></div>
          </template>
        </div>

        <span class="row-label ml-4 text-body-2 font-weight-bold text-grey">{{ row.label }}</span>
      </div>
    </div>

    <!-- Notification Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="4000" location="bottom right">
      {{ snackbar.text }}
      <template #actions>
        <v-btn variant="text" @click="snackbar.show = false">Close</v-btn>
      </template>
    </v-snackbar>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import api from '../api/client';
import { useAuthStore } from '../stores/auth';
import { useRouter } from 'vue-router';

const props = defineProps({
  theaterId: { type: String, required: true },
  showtimeId: { type: String, required: true },
});

const emit = defineEmits(['seats-updated', 'held-seats-changed']);

const authStore = useAuthStore();
const router = useRouter();

const seats = ref([]);
const loading = ref(false);
const inFlightBookings = ref([]); // List of active booking_uuids being polled
const processingSeatIds = ref(new Set());
const countdownSeconds = ref(0);
let countdownInterval = null;

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

// Compute maximum seat number across the theater to preserve grid alignment
const maxSeatCol = computed(() => {
  let max = 0;
  for (const s of seats.value) {
    if (s.seat_number > max) max = s.seat_number;
  }
  return max || 14;
});

// Group seats by row with fixed slot positions
const groupedRows = computed(() => {
  const map = {};
  for (const s of seats.value) {
    if (!map[s.row_label]) {
      map[s.row_label] = {};
    }
    map[s.row_label][s.seat_number] = s;
  }

  const sortedLabels = Object.keys(map).sort((a, b) => a.localeCompare(b));
  return sortedLabels.map((label) => {
    const slots = [];
    for (let num = 1; num <= maxSeatCol.value; num++) {
      slots.push({
        seatNumber: num,
        seat: map[label][num] || null,
      });
    }
    return { label, slots };
  });
});

const myHeldSeats = computed(() => {
  return seats.value.filter((s) => s.is_mine && s.status === 'held');
});

function isSeatProcessing(seatId) {
  return processingSeatIds.value.has(seatId);
}

function isSeatDisabled(seat) {
  if (isSeatProcessing(seat.id)) return true;
  if (seat.status === 'confirmed') return true;
  if (seat.status === 'held' && !seat.is_mine) return true;
  return false;
}

function getSeatClasses(seat) {
  const classes = [];
  if (isSeatProcessing(seat.id)) {
    classes.push('seat-processing');
  } else if (seat.is_mine && seat.status === 'held') {
    classes.push('seat-held-mine');
  } else if (seat.status === 'confirmed') {
    classes.push('seat-confirmed');
  } else if (seat.status === 'held' && !seat.is_mine) {
    classes.push('seat-held-other');
  } else {
    // Available with zone colors
    if (seat.zone_name.includes('VIP')) classes.push('zone-vip');
    else if (seat.zone_name.includes('Premium')) classes.push('zone-premium');
    else classes.push('zone-standard');
  }
  return classes.join(' ');
}

function getSeatTooltip(seat) {
  if (isSeatProcessing(seat.id)) return 'Enqueued in booking queue...';
  if (seat.is_mine && seat.status === 'held') return `Held by you (${seat.row_label}${seat.seat_number}) - Click to cancel hold`;
  if (seat.status === 'confirmed') return `Seat ${seat.row_label}${seat.seat_number} is Sold Out`;
  if (seat.status === 'held') return `Seat ${seat.row_label}${seat.seat_number} is currently held by another customer`;
  return `${seat.zone_name}: Row ${seat.row_label}, Seat ${seat.seat_number} ($${(seat.price_cents / 100).toFixed(2)})`;
}

// Fetch seats from server
async function fetchSeats() {
  if (!props.theaterId || !props.showtimeId) return;
  try {
    loading.value = true;
    const res = await api.get(`/theaters/${props.theaterId}/seats?showtime_id=${props.showtimeId}`);
    seats.value = res.data;
    emit('seats-updated', seats.value);
    emit('held-seats-changed', {
      heldSeats: myHeldSeats.value,
      countdownSeconds: countdownSeconds.value,
    });

    // Calculate nearest expiry for countdown
    updateCountdown();
  } catch (err) {
    console.error('Failed to load seats', err);
  } finally {
    loading.value = false;
  }
}

function updateCountdown() {
  const heldWithExpiry = seats.value.filter((s) => s.is_mine && s.held_expires_at);
  if (heldWithExpiry.length === 0) {
    countdownSeconds.value = 0;
    emit('held-seats-changed', {
      heldSeats: myHeldSeats.value,
      countdownSeconds: 0,
    });
    return;
  }

  // Find min expires_at
  const earliestExpiry = Math.min(...heldWithExpiry.map((s) => new Date(s.held_expires_at).getTime()));
  const now = Date.now();
  const diffSecs = Math.max(0, Math.floor((earliestExpiry - now) / 1000));
  countdownSeconds.value = diffSecs;
  emit('held-seats-changed', {
    heldSeats: myHeldSeats.value,
    countdownSeconds: diffSecs,
  });
}

// Handle Cancel Hold
async function handleCancelHold(seat) {
  processingSeatIds.value.add(seat.id);
  try {
    await api.post('/bookings/release', {
      showtime_id: props.showtimeId,
      seat_id: seat.id,
    });
    showToast(`Hold for Seat ${seat.row_label}${seat.seat_number} cancelled`, 'info');
    await fetchSeats();
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Failed to cancel hold';
    showToast(errMsg, 'error');
  } finally {
    processingSeatIds.value.delete(seat.id);
  }
}

// Handle Seat Click: Enqueue Booking Request or Cancel existing Hold
async function handleSeatClick(seat) {
  if (!authStore.isAuthenticated) {
    showToast('Please login to book seats', 'warning');
    router.push({ name: 'Login', query: { redirect: router.currentRoute.value.fullPath } });
    return;
  }

  if (seat.is_mine && seat.status === 'held') {
    // Click on held seat cancels hold
    await handleCancelHold(seat);
    return;
  }

  if (seat.status !== 'available') return;

  // Add to processing set
  processingSeatIds.value.add(seat.id);

  try {
    const res = await api.post('/bookings', {
      showtime_id: props.showtimeId,
      seat_id: seat.id,
    });

    const bookingUuid = res.data.booking_uuid;
    inFlightBookings.value.push(bookingUuid);
    showToast(`Request for Seat ${seat.row_label}${seat.seat_number} enqueued...`, 'info');

    // Poll this specific booking_uuid until resolved (§12 locked decision)
    pollBookingStatus(bookingUuid, seat);
  } catch (err) {
    processingSeatIds.value.delete(seat.id);
    const errText = err.response?.data?.error || 'Failed to enqueue booking';
    showToast(errText, 'error');
  }
}

async function pollBookingStatus(uuid, seat) {
  const maxAttempts = 30;
  let attempts = 0;

  const interval = setInterval(async () => {
    attempts++;
    try {
      const res = await api.get(`/bookings/${uuid}`);
      const status = res.data.status;

      if (status === 'success') {
        clearInterval(interval);
        inFlightBookings.value = inFlightBookings.value.filter((id) => id !== uuid);
        processingSeatIds.value.delete(seat.id);
        showToast(`Seat ${seat.row_label}${seat.seat_number} held successfully!`, 'success');
        // Trigger seat map refresh only on own booking resolve
        await fetchSeats();
      } else if (status === 'failed') {
        clearInterval(interval);
        inFlightBookings.value = inFlightBookings.value.filter((id) => id !== uuid);
        processingSeatIds.value.delete(seat.id);
        const reason = res.data.failure_reason === 'seat_taken' ? 'Seat was taken by another user' : (res.data.failure_reason || 'Booking failed');
        showToast(`Seat ${seat.row_label}${seat.seat_number}: ${reason}`, 'error');
        // Trigger seat map refresh
        await fetchSeats();
      } else if (attempts >= maxAttempts) {
        clearInterval(interval);
        inFlightBookings.value = inFlightBookings.value.filter((id) => id !== uuid);
        processingSeatIds.value.delete(seat.id);
        showToast(`Seat ${seat.row_label}${seat.seat_number} request timed out`, 'error');
        await fetchSeats();
      }
    } catch (err) {
      clearInterval(interval);
      inFlightBookings.value = inFlightBookings.value.filter((id) => id !== uuid);
      processingSeatIds.value.delete(seat.id);
      console.error('Error polling booking status', err);
    }
  }, 400);
}

onMounted(() => {
  fetchSeats();
  countdownInterval = setInterval(() => {
    if (countdownSeconds.value > 0) {
      countdownSeconds.value--;
      emit('held-seats-changed', {
        heldSeats: myHeldSeats.value,
        countdownSeconds: countdownSeconds.value,
      });
      if (countdownSeconds.value === 0) {
        // Expiry reached, refresh seats
        fetchSeats();
      }
    }
  }, 1000);
});

onUnmounted(() => {
  if (countdownInterval) clearInterval(countdownInterval);
});

watch(() => [props.theaterId, props.showtimeId], () => {
  fetchSeats();
});

defineExpose({
  fetchSeats,
  handleCancelHold,
});
</script>

<style scoped>
.seat-map-container {
  background: #181c27;
  border-radius: 12px;
  border: 1px solid #2a3142;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.screen-container {
  perspective: 400px;
}

.screen-curve {
  width: 70%;
  height: 14px;
  background: linear-gradient(180deg, #f59e0b 0%, rgba(245, 158, 11, 0.1) 100%);
  border-radius: 50% / 100% 100% 0 0;
  box-shadow: 0 4px 20px rgba(245, 158, 11, 0.4);
}

.tracking-widest {
  letter-spacing: 0.25em;
}

.legend-box {
  display: inline-block;
  width: 16px;
  height: 16px;
  border-radius: 4px;
}

.seat-btn {
  width: 38px;
  height: 38px;
  border-radius: 8px;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: #fff;
}

.seat-btn:hover:not(:disabled) {
  transform: translateY(-2px) scale(1.08);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
}

/* Zone Styles */
.zone-vip {
  background: linear-gradient(135deg, #d97706, #b45309);
  color: #fff;
  box-shadow: 0 2px 6px rgba(217, 119, 6, 0.3);
}

.zone-premium {
  background: linear-gradient(135deg, #4f46e5, #3730a3);
  color: #fff;
  box-shadow: 0 2px 6px rgba(79, 70, 229, 0.3);
}

.zone-standard {
  background: linear-gradient(135deg, #0d9488, #0f766e);
  color: #fff;
  box-shadow: 0 2px 6px rgba(13, 148, 136, 0.3);
}

/* Status Styles */
.seat-held-mine {
  background: #10b981 !important;
  color: #fff !important;
  box-shadow: 0 0 12px #10b981;
  animation: pulse-border 2s infinite;
}

.seat-held-mine .seat-icon-check {
  display: inline-flex;
}

.seat-held-mine .seat-icon-cancel {
  display: none;
}

.seat-held-mine:hover {
  background: #ef4444 !important;
  box-shadow: 0 0 14px rgba(239, 68, 68, 0.8) !important;
  animation: none !important;
}

.seat-held-mine:hover .seat-icon-check {
  display: none;
}

.seat-held-mine:hover .seat-icon-cancel {
  display: inline-flex;
}

.seat-processing {
  background: #3b82f6 !important;
  color: #fff !important;
  opacity: 0.8;
  animation: pulse-opac 1s infinite alternate;
}

.seat-held-other,
.seat-confirmed,
.seat-taken {
  background: #2b3245 !important;
  color: #64748b !important;
  cursor: not-allowed;
  opacity: 0.5;
}

.seat-spacer {
  width: 32px;
  height: 32px;
  visibility: hidden;
}

.gap-2 {
  gap: 8px;
}

.gap-4 {
  gap: 16px;
}

.max-w-xl {
  max-width: 600px;
}

@keyframes pulse-border {
  0% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7); }
  70% { box-shadow: 0 0 0 8px rgba(16, 185, 129, 0); }
  100% { box-shadow: 0 0 0 0 rgba(16, 185, 129, 0); }
}

@keyframes pulse-opac {
  from { opacity: 0.6; }
  to { opacity: 1; }
}
</style>
