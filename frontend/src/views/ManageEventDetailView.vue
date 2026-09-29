<template>
  <div class="manage-layout">
    <!-- Loading Skeleton -->
    <v-skeleton-loader v-if="loading" type="article, table, actions" theme="dark"
      class="rounded-xl ma-6"></v-skeleton-loader>

    <!-- Error State -->
    <v-alert v-else-if="error" type="error" variant="tonal" class="rounded-xl ma-6">
      {{ error }}
    </v-alert>

    <div v-else-if="event" class="manage-layout-body">
      <!-- Sidebar (Flush Left) -->
      <EventSidebar :event="event" active-item="overview" @open-seat-map="openTheaterSeatMapSelector" />

      <!-- Main Content Column (Right) -->
      <main class="manage-main-content">
        <div class="d-flex flex-column gap-6">
          <!-- Event Header Card -->
          <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="3">
            <div class="d-flex align-start gap-4">
              <v-avatar color="amber-darken-3" size="52" rounded="lg">
                <v-icon icon="mdi-filmstrip" color="white" size="30"></v-icon>
              </v-avatar>
              <div>
                <div class="d-flex align-center gap-2 mb-1">
                  <span class="text-caption text-grey font-weight-mono text-uppercase">EVENT REFERENCE</span>
                  <span class="text-caption font-weight-mono text-amber-darken-1 font-weight-bold">#{{ event.id
                  }}</span>
                </div>
                <h1 class="text-h4 font-weight-bold text-white mb-2">
                  {{ event.title }}
                </h1>
                <div class="d-flex align-center gap-2 flex-wrap">
                  <v-chip size="small" color="indigo-lighten-2" variant="tonal" class="font-weight-bold">
                    <v-icon start icon="mdi-ticket-percent-outline" size="14"></v-icon>
                    Limit: {{ event.max_seats_per_booking || 6 }} seats / order
                  </v-chip>
                  <v-chip size="small" color="teal-lighten-2" variant="tonal" class="font-weight-bold">
                    <v-icon start icon="mdi-account-lock-outline" size="14"></v-icon>
                    Personal Limit: {{ event.max_seats_per_user || 10 }} seats / customer
                  </v-chip>
                  <v-btn
                    variant="tonal"
                    size="small"
                    color="amber-darken-1"
                    prepend-icon="mdi-calendar-clock"
                    class="font-weight-bold text-none ml-2"
                    :to="`/manage/events/${event.id}/showtimes`"
                  >
                    {{ eventShowtimes.length }} {{ eventShowtimes.length === 1 ?
                      'Showtime session' : 'Showtime sessions' }}
                    <v-icon end icon="mdi-chevron-right" size="14"></v-icon>
                  </v-btn>
                </div>
              </div>
            </div>
          </v-card>

          <!-- Embedded Event Analytics Dashboard Section -->
          <div>
            <div class="d-flex justify-space-between align-center mb-3">
              <h2 class="text-h6 font-weight-bold text-white d-flex align-center gap-2">
                <v-icon icon="mdi-chart-box-outline" color="amber-darken-1" size="24"></v-icon>
                Event Analytics & Financial Performance
              </h2>
              <v-btn prepend-icon="mdi-refresh" variant="text" size="small" color="grey-lighten-1"
                :loading="loadingMetrics" @click="fetchEventMetrics">
                Refresh Metrics
              </v-btn>
            </div>

            <!-- Metric KPI Cards Grid -->
            <v-row dense class="mb-2">
              <!-- 1. Gross Revenue -->
              <v-col cols="12" sm="6" md="4" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Gross Revenue</span>
                    <v-avatar color="indigo-darken-2" variant="tonal" size="32">
                      <v-icon icon="mdi-cash-multiple" size="18"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-white font-weight-mono">
                    ${{ ((metrics.gross_sales_cents || 0) / 100).toFixed(2) }}
                  </div>
                  <div class="text-caption text-grey mt-1">Total ticket sales revenue</div>
                </v-card>
              </v-col>

              <!-- 2. Total Refunds -->
              <v-col cols="12" sm="6" md="4" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Total Refunded</span>
                    <v-avatar color="red-darken-3" variant="tonal" size="32">
                      <v-icon icon="mdi-cash-refund" size="18"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-red-accent-2 font-weight-mono">
                    ${{ ((metrics.total_refunds_cents || 0) / 100).toFixed(2) }}
                  </div>
                  <div class="text-caption text-grey mt-1">
                    {{ metrics.refunded_tickets || 0 }} {{ (metrics.refunded_tickets === 1) ? 'ticket' : 'tickets' }}
                    refunded
                  </div>
                </v-card>
              </v-col>

              <!-- 3. Net Profit -->
              <v-col cols="12" sm="6" md="4" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100 highlight-card">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-amber-lighten-2 font-weight-bold text-uppercase">Net Profit</span>
                    <v-avatar color="amber-darken-1" variant="tonal" size="32">
                      <v-icon icon="mdi-trending-up" size="18"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-amber-darken-1 font-weight-mono">
                    ${{ ((metrics.net_profit_cents || 0) / 100).toFixed(2) }}
                  </div>
                  <div class="text-caption text-grey mt-1">Gross sales minus refunds</div>
                </v-card>
              </v-col>

              <!-- 4. Total Tickets Created -->
              <v-col cols="12" sm="6" md="4" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Tickets Issued</span>
                    <v-avatar color="teal-darken-2" variant="tonal" size="32">
                      <v-icon icon="mdi-ticket-confirmation" size="18"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-white font-weight-mono">
                    {{ metrics.total_tickets_created || 0 }}
                  </div>
                  <div class="text-caption text-grey mt-1">
                    <span class="text-success font-weight-bold">{{ metrics.active_tickets || 0 }} active</span> •
                    <span class="text-grey">{{ metrics.used_tickets || 0 }} scanned</span>
                  </div>
                </v-card>
              </v-col>

              <!-- 5. Checked-In Passes -->
              <v-col cols="12" sm="6" md="4" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Checked-In</span>
                    <v-avatar color="blue-darken-2" variant="tonal" size="32">
                      <v-icon icon="mdi-qrcode-scan" size="18"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-white font-weight-mono">
                    {{ metrics.used_tickets || 0 }}
                  </div>
                  <div class="text-caption text-grey mt-1">Scanned at gate entrance</div>
                </v-card>
              </v-col>

              <!-- 6. Attendance Rate -->
              <v-col cols="12" sm="6" md="4" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Attendance Rate</span>
                    <v-avatar color="purple-darken-2" variant="tonal" size="32">
                      <v-icon icon="mdi-account-check-outline" size="18"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-white font-weight-mono">
                    {{ metrics.attendance_rate_percent || 0 }}%
                  </div>
                  <v-progress-linear :model-value="metrics.attendance_rate_percent || 0" color="amber-darken-1"
                    bg-color="grey-darken-3" height="6" rounded class="mt-2"></v-progress-linear>
                </v-card>
              </v-col>
            </v-row>
          </div>
        </div>
      </main>
    </div>

    <!-- Theater Selector Dialog for Seat Map Editor -->
    <v-dialog v-model="theaterSelectorDialog.show" max-width="450">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold">Select Theater Hall</v-card-title>
        <v-card-text>
          <div class="text-caption text-grey mb-4">Choose a theater to open the interactive Seat Map Editor.</div>
          <v-select v-model="theaterSelectorDialog.theaterId" label="Theater" :items="theaters" item-title="name"
            item-value="id" variant="outlined" density="comfortable"></v-select>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="theaterSelectorDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn color="indigo-darken-1" variant="flat" class="font-weight-bold text-none"
            :disabled="!theaterSelectorDialog.theaterId" @click="goToSeatMapEditor">
            Open Editor
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialog.show" max-width="420">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold text-red-lighten-2">Confirm Deletion</v-card-title>
        <v-card-text>
          Are you sure you want to delete <strong>{{ deleteDialog.itemName }}</strong>?
          <div class="text-caption text-grey mt-2">This action cannot be undone.</div>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="deleteDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn color="red-darken-1" variant="flat" class="font-weight-bold text-none" :loading="deleteDialog.deleting"
            @click="confirmDelete">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import api from '../api/client';
import EventSidebar from '../components/EventSidebar.vue';

const props = defineProps({
  id: { type: String, required: true },
});

const route = useRoute();
const router = useRouter();

const event = ref(null);
const allShowtimes = ref([]);
const theaters = ref([]);
const loading = ref(true);
const error = ref(null);

const loadingMetrics = ref(false);
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

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

const eventShowtimes = computed(() => {
  if (!event.value) return [];
  return allShowtimes.value
    .filter((s) => s.event_id === event.value.id)
    .sort((a, b) => new Date(a.starts_at) - new Date(b.starts_at));
});

const theaterSelectorDialog = ref({
  show: false,
  theaterId: null,
});

const deleteDialog = ref({
  show: false,
  type: '', // 'event' | 'showtime'
  itemId: null,
  itemName: '',
  deleting: false,
});

function formatDate(isoStr) {
  if (!isoStr) return '—';
  const d = new Date(isoStr);
  return d.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  });
}

async function loadData() {
  const eventId = props.id || route.params.id;
  loading.value = true;
  error.value = null;

  try {
    const [eventsRes, showtimesRes, theatersRes] = await Promise.all([
      api.get('/events'),
      api.get('/showtimes'),
      api.get('/theaters'),
    ]);

    const found = eventsRes.data.find((e) => e.id === eventId);
    if (!found) {
      error.value = 'Event not found';
      return;
    }
    event.value = found;
    allShowtimes.value = showtimesRes.data;
    theaters.value = theatersRes.data;

    await fetchEventMetrics();
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load event details';
  } finally {
    loading.value = false;
  }
}

async function fetchEventMetrics() {
  const eventId = props.id || route.params.id;
  loadingMetrics.value = true;
  try {
    const res = await api.get('/analytics/dashboard', { params: { event_id: eventId } });
    metrics.value = res.data.metrics || {};
  } catch (err) {
    console.error('Failed to load event metrics', err);
  } finally {
    loadingMetrics.value = false;
  }
}

function openDeleteEventDialog() {
  deleteDialog.value = {
    show: true,
    type: 'event',
    itemId: event.value.id,
    itemName: event.value.title,
    deleting: false,
  };
}

function openTheaterSeatMapSelector() {
  const defaultTheaterId = eventShowtimes.value[0]?.theater_id || theaters.value[0]?.id || null;
  theaterSelectorDialog.value = {
    show: true,
    theaterId: defaultTheaterId,
  };
}

function goToSeatMapEditor() {
  if (theaterSelectorDialog.value.theaterId) {
    theaterSelectorDialog.value.show = false;
    router.push(`/manage/theaters/${theaterSelectorDialog.value.theaterId}/seat-map`);
  }
}

async function confirmDelete() {
  deleteDialog.value.deleting = true;
  try {
    if (deleteDialog.value.type === 'event') {
      await api.delete(`/events/${deleteDialog.value.itemId}`);
      showToast('Event deleted successfully', 'success');
      deleteDialog.value.show = false;
      router.push('/manage/events');
    }
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to delete item', 'error');
  } finally {
    deleteDialog.value.deleting = false;
  }
}

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.manage-layout {
  min-height: calc(100vh - 56px);
  width: 100%;
}

.manage-layout-body {
  display: flex;
  flex-direction: column;
  min-height: calc(100vh - 56px);
  width: 100%;
}

@media (min-width: 960px) {
  .manage-layout-body {
    flex-direction: row;
  }
}

.manage-main-content {
  flex: 1;
  min-width: 0;
  padding: 24px 32px 48px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.kpi-card {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.kpi-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35);
}

.highlight-card {
  border: 1px solid rgba(245, 158, 11, 0.3) !important;
  background: linear-gradient(180deg, #1e2330 0%, #181c27 100%) !important;
}

.gap-1 {
  gap: 4px;
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

.gap-6 {
  gap: 24px;
}
</style>
