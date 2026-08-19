<template>
  <v-container class="py-8 max-w-7xl">
    <!-- Header -->
    <div class="d-flex flex-wrap justify-space-between align-center gap-4 mb-6">
      <div>
        <h1 class="text-h4 font-weight-bold text-white mb-1 d-flex align-center gap-2">
          <v-icon icon="mdi-chart-box-outline" color="amber-darken-1" size="32"></v-icon>
          Event Analytics & Dashboard
        </h1>
        <div class="text-caption text-grey">
          Real-time metrics on gross revenue, refunds, net profit, ticket check-in rate, and capacity utilization.
        </div>
      </div>

      <!-- Controls / Event Filter Dropdown -->
      <div class="d-flex align-center gap-3 flex-wrap">
        <v-select
          v-model="selectedEventId"
          :items="eventOptions"
          item-title="title"
          item-value="id"
          label="Filter by Event"
          variant="outlined"
          density="compact"
          hide-details
          theme="dark"
          style="min-width: 240px;"
          @update:model-value="fetchDashboardData"
        ></v-select>

        <v-btn
          prepend-icon="mdi-refresh"
          variant="outlined"
          size="small"
          :loading="loading"
          @click="fetchDashboardData"
        >
          Refresh
        </v-btn>
      </div>
    </div>

    <!-- Loading Skeleton -->
    <v-row v-if="loading && !metrics">
      <v-col v-for="i in 6" :key="i" cols="12" sm="6" md="4">
        <v-skeleton-loader type="card" theme="dark" class="rounded-xl"></v-skeleton-loader>
      </v-col>
    </v-row>

    <!-- Error State -->
    <v-alert v-else-if="error" type="error" variant="tonal" class="rounded-xl mb-6">
      {{ error }}
    </v-alert>

    <div v-else>
      <!-- KPI Metric Cards Grid -->
      <v-row class="mb-6" dense>
        <!-- 1. Gross Revenue -->
        <v-col cols="12" sm="6" md="4" class="pa-2">
          <v-card theme="dark" class="kpi-card pa-5 rounded-xl border bg-surface h-100">
            <div class="d-flex justify-space-between align-center mb-2">
              <span class="text-caption text-grey font-weight-bold text-uppercase">Gross Revenue</span>
              <v-avatar color="indigo-darken-2" variant="tonal" size="36">
                <v-icon icon="mdi-cash-multiple" size="20"></v-icon>
              </v-avatar>
            </div>
            <div class="text-h4 font-weight-black text-white font-weight-mono">
              ${{ ((metrics.gross_sales_cents || 0) / 100).toFixed(2) }}
            </div>
            <div class="text-caption text-grey mt-1">Total revenue from all ticket sales</div>
          </v-card>
        </v-col>

        <!-- 2. Total Refunds -->
        <v-col cols="12" sm="6" md="4" class="pa-2">
          <v-card theme="dark" class="kpi-card pa-5 rounded-xl border bg-surface h-100">
            <div class="d-flex justify-space-between align-center mb-2">
              <span class="text-caption text-grey font-weight-bold text-uppercase">Total Refunded</span>
              <v-avatar color="red-darken-3" variant="tonal" size="36">
                <v-icon icon="mdi-cash-refund" size="20"></v-icon>
              </v-avatar>
            </div>
            <div class="text-h4 font-weight-black text-red-accent-2 font-weight-mono">
              ${{ ((metrics.total_refunds_cents || 0) / 100).toFixed(2) }}
            </div>
            <div class="text-caption text-grey mt-1">
              {{ metrics.refunded_tickets || 0 }} {{ (metrics.refunded_tickets === 1) ? 'ticket' : 'tickets' }} refunded
            </div>
          </v-card>
        </v-col>

        <!-- 3. Net Profit -->
        <v-col cols="12" sm="6" md="4" class="pa-2">
          <v-card theme="dark" class="kpi-card pa-5 rounded-xl border bg-surface h-100 highlight-card">
            <div class="d-flex justify-space-between align-center mb-2">
              <span class="text-caption text-amber-lighten-2 font-weight-bold text-uppercase">Net Profit</span>
              <v-avatar color="amber-darken-1" variant="tonal" size="36">
                <v-icon icon="mdi-trending-up" size="20"></v-icon>
              </v-avatar>
            </div>
            <div class="text-h4 font-weight-black text-amber-darken-1 font-weight-mono">
              ${{ ((metrics.net_profit_cents || 0) / 100).toFixed(2) }}
            </div>
            <div class="text-caption text-grey mt-1">Gross sales minus total refunds</div>
          </v-card>
        </v-col>

        <!-- 4. Tickets Created / Active -->
        <v-col cols="12" sm="6" md="4" class="pa-2">
          <v-card theme="dark" class="kpi-card pa-5 rounded-xl border bg-surface h-100">
            <div class="d-flex justify-space-between align-center mb-2">
              <span class="text-caption text-grey font-weight-bold text-uppercase">Total Tickets Issued</span>
              <v-avatar color="teal-darken-2" variant="tonal" size="36">
                <v-icon icon="mdi-ticket-confirmation" size="20"></v-icon>
              </v-avatar>
            </div>
            <div class="text-h4 font-weight-black text-white font-weight-mono">
              {{ metrics.total_tickets_created || 0 }}
            </div>
            <div class="text-caption text-grey mt-1">
              <span class="text-success font-weight-bold">{{ metrics.active_tickets || 0 }} active</span> •
              <span class="text-grey">{{ metrics.used_tickets || 0 }} scanned</span>
            </div>
          </v-card>
        </v-col>

        <!-- 5. Checked-In / Scanned Passes -->
        <v-col cols="12" sm="6" md="4" class="pa-2">
          <v-card theme="dark" class="kpi-card pa-5 rounded-xl border bg-surface h-100">
            <div class="d-flex justify-space-between align-center mb-2">
              <span class="text-caption text-grey font-weight-bold text-uppercase">Tickets Checked-In</span>
              <v-avatar color="blue-darken-2" variant="tonal" size="36">
                <v-icon icon="mdi-qrcode-scan" size="20"></v-icon>
              </v-avatar>
            </div>
            <div class="text-h4 font-weight-black text-white font-weight-mono">
              {{ metrics.used_tickets || 0 }}
            </div>
            <div class="text-caption text-grey mt-1">Scanned at gate entrance</div>
          </v-card>
        </v-col>

        <!-- 6. Attendance Rate -->
        <v-col cols="12" sm="6" md="4" class="pa-2">
          <v-card theme="dark" class="kpi-card pa-5 rounded-xl border bg-surface h-100">
            <div class="d-flex justify-space-between align-center mb-2">
              <span class="text-caption text-grey font-weight-bold text-uppercase">Attendance Rate</span>
              <v-avatar color="purple-darken-2" variant="tonal" size="36">
                <v-icon icon="mdi-account-check-outline" size="20"></v-icon>
              </v-avatar>
            </div>
            <div class="text-h4 font-weight-black text-white font-weight-mono">
              {{ metrics.attendance_rate_percent || 0 }}%
            </div>
            <v-progress-linear
              :model-value="metrics.attendance_rate_percent || 0"
              color="amber-darken-1"
              bg-color="grey-darken-3"
              height="6"
              rounded
              class="mt-2"
            ></v-progress-linear>
          </v-card>
        </v-col>
      </v-row>

      <!-- Showtime Performance Breakdown Table -->
      <v-card theme="dark" class="pa-6 rounded-xl border bg-surface mb-8" elevation="3">
        <div class="d-flex justify-space-between align-center mb-4">
          <div>
            <h2 class="text-h6 font-weight-bold text-white mb-1">Showtimes Performance Breakdown</h2>
            <span class="text-caption text-grey">Individual showtime occupancy, attendance, and net revenue.</span>
          </div>
          <v-chip color="grey" variant="outlined" size="small" class="font-weight-bold">
            {{ showtimes.length }} Showtimes
          </v-chip>
        </div>

        <div v-if="showtimes.length === 0" class="text-center pa-8 text-grey">
          No showtimes found for the selected event.
        </div>

        <div v-else class="table-container rounded-lg border overflow-hidden">
          <v-table theme="dark">
            <thead>
              <tr>
                <th class="text-left font-weight-bold">Showtime & Venue</th>
                <th class="text-left font-weight-bold">Capacity & Sold</th>
                <th class="text-left font-weight-bold">Checked-In</th>
                <th class="text-left font-weight-bold">Refunded</th>
                <th class="text-right font-weight-bold">Gross</th>
                <th class="text-right font-weight-bold">Refunds</th>
                <th class="text-right font-weight-bold">Net Profit</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="s in showtimes" :key="s.showtime_id">
                <!-- Showtime & Venue -->
                <td class="py-4">
                  <div class="font-weight-bold text-white text-body-2">{{ s.event_title }}</div>
                  <div class="text-caption text-amber-darken-1 font-weight-medium">
                    <v-icon icon="mdi-calendar-clock" size="14" class="mr-1"></v-icon>
                    {{ formatDate(s.starts_at) }}
                  </div>
                  <div class="text-caption text-grey">
                    <v-icon icon="mdi-map-marker" size="14" class="mr-1"></v-icon>
                    {{ s.theater_name }}
                  </div>
                </td>

                <!-- Capacity & Sold -->
                <td>
                  <div class="d-flex align-center gap-2">
                    <span class="font-weight-bold text-white text-body-2">{{ s.sold_seats }}</span>
                    <span class="text-caption text-grey">/ {{ s.total_seats }} seats</span>
                    <v-chip size="x-small" color="teal" variant="flat" class="font-weight-bold ml-1">
                      {{ s.total_seats > 0 ? Math.round((s.sold_seats / s.total_seats) * 100) : 0 }}%
                    </v-chip>
                  </div>
                  <v-progress-linear
                    :model-value="s.total_seats > 0 ? (s.sold_seats / s.total_seats) * 100 : 0"
                    color="teal"
                    bg-color="grey-darken-3"
                    height="4"
                    rounded
                    class="mt-1"
                    style="max-width: 140px;"
                  ></v-progress-linear>
                </td>

                <!-- Checked-In -->
                <td>
                  <div class="font-weight-bold text-white">
                    {{ s.used_tickets }}
                    <span class="text-caption text-grey font-weight-normal">
                      ({{ s.sold_seats > 0 ? Math.round((s.used_tickets / s.sold_seats) * 100) : 0 }}%)
                    </span>
                  </div>
                </td>

                <!-- Refunded -->
                <td>
                  <span v-if="s.refunded_tickets > 0" class="text-red-accent-2 font-weight-bold">
                    {{ s.refunded_tickets }}
                  </span>
                  <span v-else class="text-grey">0</span>
                </td>

                <!-- Gross -->
                <td class="text-right font-weight-mono text-white">
                  ${{ (s.gross_cents / 100).toFixed(2) }}
                </td>

                <!-- Refunds -->
                <td class="text-right font-weight-mono text-red-accent-2">
                  <span v-if="s.refunded_cents > 0">-${{ (s.refunded_cents / 100).toFixed(2) }}</span>
                  <span v-else class="text-grey">$0.00</span>
                </td>

                <!-- Net -->
                <td class="text-right font-weight-mono font-weight-bold text-amber-darken-1">
                  ${{ (s.net_cents / 100).toFixed(2) }}
                </td>
              </tr>
            </tbody>
          </v-table>
        </div>
      </v-card>
    </div>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import api from '../api/client';

const route = useRoute();
const loading = ref(true);
const error = ref(null);
const metrics = ref({
  gross_sales_cents: 0,
  total_refunds_cents: 0,
  net_profit_cents: 0,
  total_tickets_created: 0,
  active_tickets: 0,
  used_tickets: 0,
  refunded_tickets: 0,
  attendance_rate_percent: 0,
});
const showtimes = ref([]);
const events = ref([]);
const selectedEventId = ref(route.query.event_id || 'all');
const eventOptions = ref([{ id: 'all', title: 'All Events (Overview)' }]);

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

async function fetchEventsList() {
  try {
    const res = await api.get('/events');
    events.value = res.data || [];
    eventOptions.value = [
      { id: 'all', title: 'All Events (Overview)' },
      ...events.value.map((e) => ({ id: e.id, title: e.title })),
    ];
  } catch (err) {
    console.error('Failed to load events list', err);
  }
}

async function fetchDashboardData() {
  loading.value = true;
  error.value = null;
  try {
    const params = {};
    if (selectedEventId.value && selectedEventId.value !== 'all') {
      params.event_id = selectedEventId.value;
    }
    const res = await api.get('/analytics/dashboard', { params });
    metrics.value = res.data.metrics || {};
    showtimes.value = res.data.showtimes || [];
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load analytics dashboard data';
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  await fetchEventsList();
  if (route.query.event_id) {
    selectedEventId.value = route.query.event_id;
  }
  await fetchDashboardData();
});
</script>

<style scoped>
.max-w-7xl {
  max-width: 1280px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.kpi-card {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.kpi-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

.highlight-card {
  border: 1px solid rgba(245, 158, 11, 0.3) !important;
  background: linear-gradient(180deg, #1e2330 0%, #181c27 100%) !important;
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
