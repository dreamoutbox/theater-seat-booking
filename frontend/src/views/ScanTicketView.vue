<template>
  <v-container class="py-8 max-w-3xl">
    <div class="d-flex align-center justify-space-between mb-6 flex-wrap gap-4">
      <div>
        <h1 class="text-h4 font-weight-bold text-white mb-1 d-flex align-center gap-2">
          <v-icon icon="mdi-qrcode-scan" color="amber-darken-1" size="36"></v-icon>
          Gatekeeper Ticket Scanner
        </h1>
        <div class="text-caption text-grey">Scan customer QR pass or enter 6-digit ticket code to verify and mark admission.</div>
      </div>

      <!-- Quick Event Filter / Scoping Badge -->
      <div v-if="assignedEvents.length > 0" class="d-flex align-center gap-2">
        <v-select
          v-model="selectedEventId"
          :items="assignedEvents"
          item-title="title"
          item-value="id"
          label="Filter Event Scoping"
          density="compact"
          variant="outlined"
          hide-details
          class="min-w-200"
          prepend-inner-icon="mdi-filmstrip"
        ></v-select>
      </div>
    </div>

    <!-- Scanner Input Card -->
    <v-card theme="dark" class="rounded-xl pa-6 border bg-surface mb-6" elevation="4">
      <form @submit.prevent="handleScan">
        <div class="text-subtitle-1 font-weight-bold text-white mb-2">
          Enter or Scan 6-Digit Code
        </div>
        <div class="d-flex gap-3 align-center">
          <v-text-field
            v-model="ticketCode"
            placeholder="e.g. A8X9K2"
            maxlength="6"
            variant="outlined"
            density="comfortable"
            hide-details
            class="font-weight-mono text-h5 uppercase-input"
            prepend-inner-icon="mdi-ticket-confirmation-outline"
            autofocus
            :disabled="scanning"
            @input="ticketCode = ticketCode.toUpperCase()"
          ></v-text-field>

          <v-btn
            color="amber-darken-1"
            size="large"
            variant="flat"
            type="submit"
            prepend-icon="mdi-check-circle"
            class="text-none font-weight-bold px-6"
            :loading="scanning"
            :disabled="!ticketCode || ticketCode.trim().length < 6"
          >
            Verify
          </v-btn>
        </div>
      </form>
    </v-card>

    <!-- Scan Result Card (Alerts) -->
    <v-fade-transition>
      <div v-if="lastResult" class="mb-6">
        <!-- Success State -->
        <v-card
          v-if="lastResult.success"
          theme="dark"
          class="rounded-xl pa-6 border border-emerald bg-emerald-darken-4 elevation-4"
        >
          <div class="d-flex align-start gap-4">
            <v-avatar color="emerald-darken-1" size="56">
              <v-icon icon="mdi-check-bold" size="36" color="white"></v-icon>
            </v-avatar>
            <div class="flex-grow-1">
              <div class="d-flex align-center justify-space-between mb-1">
                <span class="text-caption text-emerald-lighten-3 font-weight-bold text-uppercase">
                  ADMISSION GRANTED • VERIFIED
                </span>
                <v-chip color="emerald" variant="flat" size="small" class="font-weight-bold font-weight-mono">
                  #{{ lastResult.ticket.code }}
                </v-chip>
              </div>
              <h2 class="text-h5 font-weight-black text-white mb-2">
                Row {{ lastResult.ticket.row_label }}, Seat {{ lastResult.ticket.seat_number }}
              </h2>
              <div class="text-body-2 text-grey-lighten-2 mb-1">
                <strong>Event:</strong> {{ lastResult.ticket.event_title }}
              </div>
              <div class="text-body-2 text-grey-lighten-2 mb-1">
                <strong>Theater:</strong> {{ lastResult.ticket.theater_name }} • {{ formatDate(lastResult.ticket.starts_at) }}
              </div>
              <div class="text-body-2 text-grey-lighten-2">
                <strong>Zone:</strong> {{ lastResult.ticket.zone_name }} • ${{ (lastResult.ticket.price_cents / 100).toFixed(2) }}
              </div>
            </div>
          </div>
        </v-card>

        <!-- Error / Already Used State -->
        <v-card
          v-else
          theme="dark"
          class="rounded-xl pa-6 border border-red bg-red-darken-4 elevation-4"
        >
          <div class="d-flex align-start gap-4">
            <v-avatar color="red-darken-1" size="56">
              <v-icon icon="mdi-close-octagon" size="36" color="white"></v-icon>
            </v-avatar>
            <div class="flex-grow-1">
              <div class="d-flex align-center justify-space-between mb-1">
                <span class="text-caption text-red-lighten-3 font-weight-bold text-uppercase">
                  ADMISSION DENIED
                </span>
                <v-chip color="red" variant="flat" size="small" class="font-weight-bold font-weight-mono">
                  #{{ lastScannedCode }}
                </v-chip>
              </div>
              <h2 class="text-h5 font-weight-black text-white mb-2">
                {{ lastResult.error }}
              </h2>
              <div v-if="lastResult.ticket" class="text-body-2 text-grey-lighten-2">
                <strong>Seat:</strong> Row {{ lastResult.ticket.row_label }}, Seat {{ lastResult.ticket.seat_number }}
                • <strong>Event:</strong> {{ lastResult.ticket.event_title }}
              </div>
            </div>
          </div>
        </v-card>
      </div>
    </v-fade-transition>

    <!-- Recent Session Scan History -->
    <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="2">
      <div class="d-flex justify-space-between align-center mb-4">
        <div>
          <h2 class="text-h6 font-weight-bold text-white mb-0">Session Scan History</h2>
          <div class="text-caption text-grey">{{ scanHistory.length }} tickets scanned in this session</div>
        </div>
        <v-btn
          v-if="scanHistory.length > 0"
          variant="text"
          size="small"
          color="grey"
          class="text-none"
          @click="scanHistory = []"
        >
          Clear History
        </v-btn>
      </div>

      <div v-if="scanHistory.length === 0" class="text-center pa-6 text-grey text-caption">
        No scan records in current session yet. Enter a code above to begin.
      </div>

      <v-list v-else density="compact" bg-color="transparent" class="pa-0">
        <v-list-item
          v-for="(item, idx) in scanHistory"
          :key="idx"
          class="rounded-lg mb-2 border bg-surface-variant pa-3"
        >
          <template #prepend>
            <v-icon
              :icon="item.success ? 'mdi-check-circle' : 'mdi-alert-circle'"
              :color="item.success ? 'emerald' : 'red'"
              class="mr-3"
              size="24"
            ></v-icon>
          </template>

          <v-list-item-title class="text-body-2 font-weight-bold text-white d-flex align-center gap-2">
            <span class="font-weight-mono text-amber-darken-1">{{ item.code }}</span>
            <span v-if="item.ticket">— Row {{ item.ticket.row_label }}, Seat {{ item.ticket.seat_number }}</span>
          </v-list-item-title>

          <v-list-item-subtitle class="text-caption text-grey mt-1">
            {{ item.message || item.error }} • {{ formatTime(item.timestamp) }}
          </v-list-item-subtitle>
        </v-list-item>
      </v-list>
    </v-card>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import api from '../api/client';

const ticketCode = ref('');
const scanning = ref(false);
const lastResult = ref(null);
const lastScannedCode = ref('');
const scanHistory = ref([]);
const assignedEvents = ref([]);
const selectedEventId = ref(null);

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

function formatTime(d) {
  if (!d) return '';
  return new Date(d).toLocaleTimeString();
}

async function loadAssignedEvents() {
  try {
    const res = await api.get('/gatekeeper/events');
    assignedEvents.value = res.data || [];
    if (assignedEvents.value.length > 0) {
      selectedEventId.value = assignedEvents.value[0].id;
    }
  } catch (err) {
    console.error('Failed to load assigned events', err);
  }
}

async function handleScan() {
  const code = ticketCode.value.trim().toUpperCase();
  if (code.length < 6) return;

  scanning.value = true;
  lastScannedCode.value = code;

  try {
    const res = await api.post('/tickets/scan', { code });
    lastResult.value = {
      success: true,
      message: res.data.message,
      ticket: res.data.ticket,
    };
    scanHistory.value.unshift({
      code,
      success: true,
      message: res.data.message,
      ticket: res.data.ticket,
      timestamp: new Date(),
    });
    ticketCode.value = '';
  } catch (err) {
    const errorMsg = err.response?.data?.error || 'Invalid ticket code or scan error';
    const ticketData = err.response?.data?.ticket || null;
    lastResult.value = {
      success: false,
      error: errorMsg,
      ticket: ticketData,
    };
    scanHistory.value.unshift({
      code,
      success: false,
      error: errorMsg,
      ticket: ticketData,
      timestamp: new Date(),
    });
  } finally {
    scanning.value = false;
  }
}

onMounted(() => {
  loadAssignedEvents();
});
</script>

<style scoped>
.max-w-3xl {
  max-width: 800px;
}

.min-w-200 {
  min-width: 220px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.bg-emerald-darken-4 {
  background: #064e3b !important;
}

.border-emerald {
  border-color: #10b981 !important;
}

.bg-red-darken-4 {
  background: #450a0a !important;
}

.border-red {
  border-color: #ef4444 !important;
}

.uppercase-input input {
  text-transform: uppercase !important;
  letter-spacing: 4px;
  font-weight: 800;
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
