<template>
  <div class="seat-map-container pa-4">


    <!-- Seating Legend (Dynamic from theater zones) -->
    <div class="d-flex flex-wrap justify-center align-center gap-3 mb-5 text-caption mt-8">
      <div v-for="zone in dynamicZones" :key="zone.id" class="d-flex align-center gap-1">
        <div class="legend-box" :style="{
          borderColor: zone.color,
          background: hexToRgba(zone.color, 0.25),
        }"></div>
        <span class="text-grey-lighten-1">{{ zone.name }} (${{ (zone.price_cents / 100).toFixed(2) }})</span>
      </div>
      <div class="d-flex align-center gap-1">
        <div class="legend-box seat-held-mine-legend"></div>
        <span class="text-amber-darken-1 font-weight-bold">Your Hold (Click to cancel)</span>
      </div>
      <div class="d-flex align-center gap-1">
        <div class="legend-box seat-processing"></div>
        <span class="text-grey-lighten-1">In Queue</span>
      </div>
      <div class="d-flex align-center gap-1">
        <div class="legend-box seat-taken"></div>
        <span class="text-grey-lighten-1">Unavailable</span>
      </div>
    </div>

    <!-- Cinema Screen visual (Matches Editor) -->
    <div class="screen-wrapper mb-6">
      <div class="cinema-screen">
        <div class="screen-text">CINEMA SCREEN</div>
      </div>
    </div>


    <!-- In-flight Booking Queue Alert -->
    <v-alert v-if="inFlightBookings.length > 0" type="info" variant="tonal" density="compact"
      class="mb-4 mx-auto max-w-xl rounded-lg">
      <div class="d-flex align-center">
        <v-progress-circular indeterminate size="16" width="2" color="info" class="mr-3"></v-progress-circular>
        <span class="text-caption">Processing seat reservation in queue...</span>
      </div>
    </v-alert>

    <!-- Interactive Seating Grid Matrix (Matching Seat Map Editor layout) -->
    <div class="seat-canvas-container">
      <!-- Grid Header Column Numbers -->
      <div class="d-flex align-center gap-1 mb-1 justify-center">
        <div class="row-header-placeholder"></div>
        <div class="d-flex align-center gap-1 seat-row-items">
          <div v-for="col in maxSeatCol" :key="col"
            class="col-header-number text-caption text-grey font-weight-bold font-weight-mono">
            {{ col }}
          </div>
        </div>
        <div class="row-header-placeholder"></div>
      </div>

      <!-- Rows of Grid Slots -->
      <div class="d-flex flex-column gap-1 align-center">
        <div v-for="row in groupedRows" :key="row.label" class="seat-row d-flex align-center gap-1">
          <!-- Row Left Header -->
          <div class="row-header-btn">
            {{ row.label }}
          </div>

          <!-- Slots in this row -->
          <div class="d-flex align-center gap-1 seat-row-items">
            <template v-for="slot in row.slots" :key="`${row.label}-${slot.seatNumber}`">
              <!-- Active Seat -->
              <button v-if="slot.seat" type="button" class="seat-node" :class="getSeatClasses(slot.seat)"
                :disabled="isSeatDisabled(slot.seat)" :style="getSeatStyle(slot.seat)"
                @click="handleSeatClick(slot.seat)" :title="getSeatTooltip(slot.seat)">
                <!-- Loading Spinner -->
                <v-progress-circular v-if="isSeatProcessing(slot.seat.id)" indeterminate size="14" width="2"
                  color="white"></v-progress-circular>

                <!-- Sold / Locked Icon -->
                <v-icon
                  v-else-if="slot.seat.status === 'confirmed' || (slot.seat.status === 'held' && !slot.seat.is_mine)"
                  icon="mdi-lock" size="12" color="grey"></v-icon>

                <!-- My Held Seat: Checkmark or Cancel X on hover -->
                <template v-else-if="slot.seat.is_mine && slot.seat.status === 'held'">
                  <v-icon icon="mdi-check" size="13" class="seat-icon-check" color="amber-darken-1"></v-icon>
                  <v-icon icon="mdi-close" size="13" class="seat-icon-cancel" color="white"></v-icon>
                </template>

                <!-- Seat Number -->
                <span v-else class="seat-number-label">{{ slot.seat.seat_number }}</span>
              </button>

              <!-- Walkway / Empty Slot Spacer -->
              <div v-else class="empty-grid-slot">
                <span class="empty-slot-number">{{ slot.seatNumber }}</span>
              </div>
            </template>
          </div>

          <!-- Row Right Header -->
          <div class="row-header-label-right text-caption text-grey font-weight-bold font-weight-mono">
            {{ row.label }}
          </div>
        </div>
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
const inFlightBookings = ref([]);
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

// Convert 0-based integer to row label: 0 -> A, 25 -> Z, 26 -> AA, etc.
function indexToRowLabel(idx) {
  let label = '';
  idx = idx + 1;
  while (idx > 0) {
    const rem = (idx - 1) % 26;
    label = String.fromCharCode(65 + rem) + label;
    idx = Math.floor((idx - 1) / 26);
  }
  return label;
}

// Convert row label to 0-based integer
function rowLabelToIndex(label) {
  if (!label) return 0;
  label = label.toUpperCase().trim();
  let index = 0;
  for (let i = 0; i < label.length; i++) {
    const code = label.charCodeAt(i);
    if (code >= 65 && code <= 90) {
      index = index * 26 + (code - 64);
    }
  }
  return Math.max(0, index - 1);
}

// Compute maximum seat number across the theater to preserve grid alignment
const maxSeatCol = computed(() => {
  let max = 0;
  for (const s of seats.value) {
    if (s.seat_number > max) max = s.seat_number;
  }
  return Math.max(max, 14);
});

// Compute maximum row index
const maxRowIdx = computed(() => {
  let maxIdx = 7; // At least 'H'
  for (const s of seats.value) {
    const idx = rowLabelToIndex(s.row_label);
    if (idx > maxIdx) maxIdx = idx;
  }
  return maxIdx;
});

// Group seats by 2D matrix of rows and slots (matches editor)
const groupedRows = computed(() => {
  const map = new Map();
  for (const s of seats.value) {
    map.set(`${s.row_label}-${s.seat_number}`, s);
  }

  const rows = [];
  for (let r = 0; r <= maxRowIdx.value; r++) {
    const rowLabel = indexToRowLabel(r);
    const slots = [];
    for (let num = 1; num <= maxSeatCol.value; num++) {
      const seat = map.get(`${rowLabel}-${num}`) || null;
      slots.push({
        seatNumber: num,
        seat,
      });
    }
    rows.push({ label: rowLabel, slots });
  }
  return rows;
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

const dynamicZones = computed(() => {
  const map = new Map();
  for (const s of seats.value) {
    if (s.zone_id && !map.has(s.zone_id)) {
      map.set(s.zone_id, {
        id: s.zone_id,
        name: s.zone_name,
        price_cents: s.price_cents,
        color: s.zone_color || getZoneColorHex(s.zone_name),
      });
    }
  }
  return Array.from(map.values()).sort((a, b) => b.price_cents - a.price_cents);
});

function hexToRgba(hex, alpha = 0.2) {
  if (!hex || typeof hex !== 'string') return `rgba(20, 184, 166, ${alpha})`;
  let c = hex.replace('#', '').trim();
  if (c.length === 3) c = c.split('').map(x => x + x).join('');
  if (c.length !== 6) return `rgba(20, 184, 166, ${alpha})`;
  const num = parseInt(c, 16);
  if (isNaN(num)) return `rgba(20, 184, 166, ${alpha})`;
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function getZoneColorHex(name) {
  if (!name) return '#94a3b8';
  if (name.toLowerCase().includes('vip')) return '#f59e0b';
  if (name.toLowerCase().includes('premium')) return '#6366f1';
  return '#14b8a6';
}

function getSeatStyle(seat) {
  const color = seat.zone_color || getZoneColorHex(seat.zone_name);
  if (seat.is_mine && seat.status === 'held') {
    return {
      borderColor: '#f59e0b',
      background: 'rgba(245, 158, 11, 0.35)',
    };
  }
  if (seat.status === 'confirmed' || (seat.status === 'held' && !seat.is_mine)) {
    return {
      borderColor: '#374151',
      background: '#1f2937',
    };
  }
  return {
    borderColor: color,
    background: hexToRgba(color, 0.18),
  };
}

function getSeatClasses(seat) {
  const classes = [];
  if (isSeatProcessing(seat.id)) {
    classes.push('seat-processing');
  } else if (seat.is_mine && seat.status === 'held') {
    classes.push('seat-held-mine');
  } else if (seat.status === 'held') {
    classes.push('seat-held-other');
  } else if (seat.status === 'confirmed') {
    classes.push('seat-confirmed');
  } else {
    classes.push('seat-available');
  }
  return classes;
}

function getSeatTooltip(seat) {
  const price = (seat.price_cents / 100).toFixed(2);
  if (seat.is_mine && seat.status === 'held') {
    return `Held by you (${seat.row_label}${seat.seat_number}) - Click to cancel hold`;
  }
  if (seat.status === 'confirmed') {
    return `Seat ${seat.row_label}${seat.seat_number} - Sold / Unavailable`;
  }
  if (seat.status === 'held') {
    return `Seat ${seat.row_label}${seat.seat_number} - Currently reserved by another customer`;
  }
  return `Row ${seat.row_label}, Seat ${seat.seat_number} • ${seat.zone_name} ($${price})`;
}

async function fetchSeats() {
  if (!props.showtimeId) return;
  try {
    const res = await api.get(`/showtimes/${props.showtimeId}/seats`);
    seats.value = res.data;
    emit('seats-updated', seats.value);
    emit('held-seats-changed', myHeldSeats.value);
    updateCountdown();
  } catch (err) {
    showToast('Failed to load theater seat layout', 'error');
  }
}

async function handleSeatClick(seat) {
  if (!authStore.isAuthenticated) {
    showToast('Please sign in or select a customer profile to reserve seats', 'warning');
    return;
  }

  // Release hold if clicking own held seat
  if (seat.is_mine && seat.status === 'held') {
    try {
      processingSeatIds.value.add(seat.id);
      await api.post('/release-hold', {
        showtime_id: props.showtimeId,
        seat_id: seat.id,
      });
      showToast(`Hold released for seat ${seat.row_label}${seat.seat_number}`, 'info');
      await fetchSeats();
    } catch (err) {
      showToast(err.response?.data?.error || 'Failed to cancel hold', 'error');
    } finally {
      processingSeatIds.value.delete(seat.id);
    }
    return;
  }

  if (seat.status !== 'available') return;

  try {
    processingSeatIds.value.add(seat.id);
    const res = await api.post('/hold', {
      showtime_id: props.showtimeId,
      seat_id: seat.id,
    });

    const bookingUuid = res.data.booking_uuid;
    inFlightBookings.value.push(bookingUuid);
    pollBookingStatus(bookingUuid, seat.id);
  } catch (err) {
    processingSeatIds.value.delete(seat.id);
    showToast(err.response?.data?.error || 'Failed to queue seat hold', 'error');
  }
}

async function pollBookingStatus(bookingUuid, seatId) {
  const maxAttempts = 20;
  let attempts = 0;

  const interval = setInterval(async () => {
    attempts++;
    try {
      const res = await api.get(`/booking/status/${bookingUuid}`);
      const data = res.data;

      if (data.status === 'success') {
        clearInterval(interval);
        inFlightBookings.value = inFlightBookings.value.filter((id) => id !== bookingUuid);
        processingSeatIds.value.delete(seatId);
        showToast('Seat hold confirmed! Complete checkout within 10 minutes.', 'success');
        await fetchSeats();
      } else if (data.status === 'failed') {
        clearInterval(interval);
        inFlightBookings.value = inFlightBookings.value.filter((id) => id !== bookingUuid);
        processingSeatIds.value.delete(seatId);
        showToast(`Could not hold seat: ${data.failure_reason || 'Already taken'}`, 'error');
        await fetchSeats();
      } else if (attempts >= maxAttempts) {
        clearInterval(interval);
        inFlightBookings.value = inFlightBookings.value.filter((id) => id !== bookingUuid);
        processingSeatIds.value.delete(seatId);
        showToast('Queue processing timed out. Please try again.', 'warning');
        await fetchSeats();
      }
    } catch (err) {
      clearInterval(interval);
      inFlightBookings.value = inFlightBookings.value.filter((id) => id !== bookingUuid);
      processingSeatIds.value.delete(seatId);
    }
  }, 1000);
}

function updateCountdown() {
  const held = myHeldSeats.value;
  if (held.length === 0) {
    countdownSeconds.value = 0;
    if (countdownInterval) clearInterval(countdownInterval);
    return;
  }

  const earliestExpire = held.reduce((earliest, s) => {
    if (!s.held_expires_at) return earliest;
    const exp = new Date(s.held_expires_at).getTime();
    return earliest ? Math.min(earliest, exp) : exp;
  }, null);

  if (!earliestExpire) return;

  const diffSeconds = Math.max(0, Math.floor((earliestExpire - Date.now()) / 1000));
  countdownSeconds.value = diffSeconds;

  if (countdownInterval) clearInterval(countdownInterval);
  countdownInterval = setInterval(() => {
    if (countdownSeconds.value > 0) {
      countdownSeconds.value--;
    } else {
      clearInterval(countdownInterval);
      fetchSeats();
    }
  }, 1000);
}

watch(
  () => [props.showtimeId, props.theaterId],
  () => {
    fetchSeats();
  }
);

onMounted(() => {
  fetchSeats();
});

onUnmounted(() => {
  if (countdownInterval) clearInterval(countdownInterval);
});

defineExpose({
  fetchSeats,
});
</script>

<style scoped>
.seat-map-container {
  width: 100%;
  background: #181c27;
  border-radius: 12px;
  border: 1px solid #2a3142;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  overflow-x: auto;
}

.screen-wrapper {
  perspective: 400px;
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
}

.cinema-screen {
  height: 20px;
  background: linear-gradient(180deg, #f59e0b 0%, rgba(245, 158, 11, 0.1) 100%);
  transform: rotateX(-20deg);
  border-radius: 50% 50% 0 0;
  box-shadow: 0 8px 20px rgba(245, 158, 11, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
}

.screen-text {
  font-size: 9px;
  letter-spacing: 4px;
  color: #fff;
  font-weight: 900;
  opacity: 0.9;
}

.legend-box {
  width: 12px;
  height: 12px;
  border-radius: 3px;
}

.zone-vip {
  background: rgba(245, 158, 11, 0.25);
  border: 1.5px solid #f59e0b;
}

.zone-premium {
  background: rgba(99, 102, 241, 0.25);
  border: 1.5px solid #6366f1;
}

.zone-standard {
  background: rgba(20, 184, 166, 0.25);
  border: 1.5px solid #14b8a6;
}

.seat-held-mine-legend {
  background: #f59e0b;
  border: 1.5px solid #f59e0b;
}

.seat-canvas-container {
  position: relative;
  min-height: 480px;
  padding: 16px;
  background: #131722;
  border-radius: 12px;
  border: 1px dashed #2a3142;
  overflow-x: auto;
}

.seat-row {
  flex-wrap: nowrap;
  min-width: max-content;
}

.row-header-placeholder {
  min-width: 32px;
}

.col-header-number {
  min-width: 32px;
  width: 32px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  opacity: 0.6;
}

.row-header-btn {
  min-width: 32px;
  height: 32px;
  padding: 0 2px;
  border-radius: 6px;
  border: 1px solid #3b4256;
  background: #1e2330;
  color: #94a3b8;
  font-size: 11px;
  font-weight: bold;
  display: flex;
  align-items: center;
  justify-content: center;
  position: sticky;
  left: 0;
  z-index: 2;
}

.row-header-label-right {
  min-width: 32px;
  text-align: center;
}

.seat-node {
  position: relative;
  min-width: 32px;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1.5px solid #4b5563;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform 0.1s ease, border-color 0.15s ease, background 0.15s ease;
  flex-shrink: 0;
  padding: 0;
}

.seat-available:hover {
  transform: scale(1.15);
  z-index: 5;
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.4);
}

.seat-held-mine {
  border-color: #f59e0b !important;
  border-width: 2px !important;
  box-shadow: 0 0 12px rgba(245, 158, 11, 0.6) !important;
  transform: scale(1.08);
  animation: pulse-hold 2s infinite ease-in-out;
}

@keyframes pulse-hold {

  0%,
  100% {
    box-shadow: 0 0 8px rgba(245, 158, 11, 0.5);
  }

  50% {
    box-shadow: 0 0 16px rgba(245, 158, 11, 0.9);
  }
}

.seat-held-mine .seat-icon-check {
  display: inline-flex;
}

.seat-held-mine .seat-icon-cancel {
  display: none;
}

.seat-held-mine:hover {
  background: #ef4444 !important;
  border-color: #ef4444 !important;
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
  border-color: #3b82f6 !important;
  color: #fff !important;
  opacity: 0.8;
  animation: pulse-opac 1s infinite alternate;
}

@keyframes pulse-opac {
  from {
    opacity: 0.6;
  }

  to {
    opacity: 1;
  }
}

.seat-held-other,
.seat-confirmed,
.seat-taken {
  background: #1f2937 !important;
  border-color: #374151 !important;
  color: #64748b !important;
  cursor: not-allowed;
  opacity: 0.5;
}

.empty-grid-slot {
  position: relative;
  min-width: 32px;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1px dashed rgba(255, 255, 255, 0.05);
  background: rgba(255, 255, 255, 0.01);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.empty-slot-number {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.08);
  font-family: monospace;
}

.seat-number-label {
  font-size: 10px;
  font-weight: 700;
  color: #fff;
}

.gap-1 {
  gap: 4px;
}

.gap-3 {
  gap: 12px;
}

.max-w-xl {
  max-width: 600px;
}
</style>
