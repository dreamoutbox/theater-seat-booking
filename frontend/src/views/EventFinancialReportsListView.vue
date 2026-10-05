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
      <EventSidebar :event="event" active-item="financial-report" />

      <!-- Main Content Column (Right) -->
      <main class="manage-main-content">
        <div class="d-flex flex-column gap-6">
          <!-- Header & Generator Card -->
          <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="3">
            <div class="d-flex flex-wrap justify-space-between align-start gap-4 mb-4">
              <div class="d-flex align-start gap-4">
                <v-avatar color="amber-darken-3" size="52" rounded="lg">
                  <v-icon icon="mdi-file-document-outline" color="white" size="30"></v-icon>
                </v-avatar>
                <div>
                  <div class="d-flex align-center gap-2 mb-1">
                    <span class="text-caption text-grey font-weight-mono text-uppercase">FINANCIAL STATEMENTS & TAX
                      FILINGS</span>
                    <span class="text-caption font-weight-mono text-amber-darken-1 font-weight-bold">#{{ event.id
                      }}</span>
                  </div>
                  <h1 class="text-h4 font-weight-bold text-white mb-2">
                    {{ event.title }}
                  </h1>
                  <p class="text-caption text-grey mb-0" style="max-width: 680px;">
                    Generated financial and tax statement snapshots for legal accounting, tax filings, and settlement
                    records. Click "Generate Report Snapshot" to capture new accounting data.
                  </p>
                </div>
              </div>

              <!-- Action / Cooldown Button -->
              <div class="d-flex align-center gap-2">
                <v-btn color="amber-darken-1" variant="flat" size="default" prepend-icon="mdi-calculator-variant"
                  class="text-none font-weight-bold" :loading="generating"
                  :disabled="!canGenerate && !authStore.isAdmin && !authStore.isStaff" @click="generateReport">
                  <span v-if="canGenerate || authStore.isAdmin || authStore.isStaff">Generate Report Snapshot</span>
                  <span v-else>Cooldown: {{ formattedCooldown }}</span>
                </v-btn>
              </div>
            </div>
          </v-card>

          <!-- Latest Report KPI Summary (if available) -->
          <div v-if="latestReport">
            <div class="d-flex justify-space-between align-center mb-3">
              <h2 class="text-h6 font-weight-bold text-white d-flex align-center gap-2">
                <v-icon icon="mdi-history" color="amber-darken-1" size="22"></v-icon>
                Latest Statement Summary (Ref: {{ latestReport.statement_ref }})
              </h2>
              <v-btn variant="outlined" color="red-lighten-2" size="small" prepend-icon="mdi-file-pdf-box"
                class="text-none font-weight-bold" :href="`/api/reports/${latestReport.id}/pdf`" target="_blank">
                View & Print PDF
              </v-btn>
            </div>

            <v-row dense>
              <!-- Gross Revenue -->
              <v-col cols="12" sm="6" md="3" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Gross Sales</span>
                    <v-avatar color="indigo-darken-2" variant="tonal" size="28">
                      <v-icon icon="mdi-cash-multiple" size="16"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-white font-weight-mono">
                    {{ ((latestReport.gross_sales_cents || 0) / 100).toFixed(2) }}
                  </div>
                  <div class="text-caption text-grey mt-1">Total customer purchases</div>
                </v-card>
              </v-col>

              <!-- Total Refunds -->
              <v-col cols="12" sm="6" md="3" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Refunded</span>
                    <v-avatar color="red-darken-3" variant="tonal" size="28">
                      <v-icon icon="mdi-cash-refund" size="16"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-red-accent-2 font-weight-mono">
                    {{ ((latestReport.total_refunds_cents || 0) / 100).toFixed(2) }}
                  </div>
                  <div class="text-caption text-grey mt-1">
                    {{ latestReport.tickets_refunded || 0 }} tickets refunded
                  </div>
                </v-card>
              </v-col>

              <!-- Net Settlement -->
              <v-col cols="12" sm="6" md="3" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100 highlight-card">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-amber-lighten-2 font-weight-bold text-uppercase">Net
                      Settlement</span>
                    <v-avatar color="amber-darken-1" variant="tonal" size="28">
                      <v-icon icon="mdi-check-decagram" size="16"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-amber-darken-1 font-weight-mono">
                    {{ ((latestReport.net_settlement_cents || 0) / 100).toFixed(2) }}
                  </div>
                  <div class="text-caption text-grey mt-1">After 10% tax withholding</div>
                </v-card>
              </v-col>

              <!-- Attendance Rate -->
              <v-col cols="12" sm="6" md="3" class="pa-2">
                <v-card theme="dark" class="kpi-card pa-4 rounded-xl border bg-surface h-100">
                  <div class="d-flex justify-space-between align-center mb-2">
                    <span class="text-caption text-grey font-weight-bold text-uppercase">Attendance Rate</span>
                    <v-avatar color="teal-darken-2" variant="tonal" size="28">
                      <v-icon icon="mdi-account-check-outline" size="16"></v-icon>
                    </v-avatar>
                  </div>
                  <div class="text-h5 font-weight-black text-white font-weight-mono">
                    {{ latestReport.attendance_rate_percent || 0 }}%
                  </div>
                  <div class="text-caption text-grey mt-1">
                    {{ latestReport.tickets_scanned || 0 }} of {{ latestReport.tickets_issued || 0 }} checked in
                  </div>
                </v-card>
              </v-col>
            </v-row>
          </div>

          <!-- Generated Reports Table Card -->
          <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="3">
            <div class="d-flex justify-space-between align-center mb-4">
              <div>
                <h2 class="text-h6 font-weight-bold text-white mb-1">Generated Statement Archive</h2>
                <span class="text-caption text-grey">All historical financial statements stored for this event</span>
              </div>
              <v-chip size="small" color="amber-darken-1" variant="tonal" class="font-weight-bold">
                {{ reports.length }} {{ reports.length === 1 ? 'Report Generated' : 'Reports Generated' }}
              </v-chip>
            </div>

            <!-- Empty State -->
            <div v-if="reports.length === 0"
              class="text-center pa-10 rounded-lg bg-surface-variant border border-dashed">
              <v-icon icon="mdi-file-document-clock-outline" size="56" color="grey" class="mb-3"></v-icon>
              <div class="text-h6 text-white font-weight-bold">No Financial Reports Generated Yet</div>
              <div class="text-caption text-grey mt-1" style="max-width: 480px; margin: 0 auto;">
                Generate your first official tax and financial statement snapshot. Point-in-time statements are securely
                stored for legal audit and tax filing purposes.
              </div>
              <v-btn color="amber-darken-1" variant="flat" prepend-icon="mdi-file-document-plus-outline"
                class="mt-5 text-none font-weight-bold" :loading="generating" :disabled="!canGenerate"
                @click="generateReport">
                Generate Initial Financial Statement
              </v-btn>
            </div>

            <!-- Reports Table -->
            <v-table v-else class="reports-table rounded-lg border">
              <thead>
                <tr>
                  <th class="text-left font-weight-bold">Statement Ref</th>
                  <th class="text-left font-weight-bold">Generated At</th>
                  <th class="text-left font-weight-bold">Generated By</th>
                  <th class="text-right font-weight-bold">Gross Revenue</th>
                  <th class="text-right font-weight-bold">Tax (10%)</th>
                  <th class="text-right font-weight-bold">Net Settlement</th>
                  <th class="text-center font-weight-bold">Attendance</th>
                  <th class="text-right font-weight-bold">Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="rep in reports" :key="rep.id">
                  <!-- Statement Ref -->
                  <td class="py-3 font-weight-bold">
                    <div class="d-flex align-center gap-2">
                      <v-icon icon="mdi-file-check-outline" size="18" color="amber-darken-1"></v-icon>
                      <span class="font-mono text-amber-lighten-2">{{ rep.statement_ref }}</span>
                    </div>
                  </td>

                  <!-- Generated Date -->
                  <td class="text-caption text-grey font-mono">
                    {{ formatDate(rep.generated_at) }}
                  </td>

                  <!-- Generated By -->
                  <td class="text-caption text-grey font-weight-medium">
                    {{ rep.generated_by_email || 'Administrator' }}
                  </td>

                  <!-- Gross Revenue -->
                  <td class="text-right font-mono font-weight-bold text-white">
                    {{ ((rep.gross_sales_cents || 0) / 100).toFixed(2) }}
                  </td>

                  <!-- Tax -->
                  <td class="text-right font-mono text-grey">
                    {{ ((rep.tax_withholding_cents || 0) / 100).toFixed(2) }}
                  </td>

                  <!-- Net Settlement -->
                  <td class="text-right font-mono font-weight-bold text-amber-darken-1">
                    {{ ((rep.net_settlement_cents || 0) / 100).toFixed(2) }}
                  </td>

                  <!-- Attendance -->
                  <td class="text-center font-mono text-caption">
                    <span class="text-success font-weight-bold">{{ rep.attendance_rate_percent || 0 }}%</span>
                    <div class="text-grey text-caption">({{ rep.tickets_scanned }}/{{ rep.tickets_issued }})</div>
                  </td>

                  <!-- Actions -->
                  <td class="text-right">
                    <div class="d-flex align-center justify-end gap-2">
                      <v-btn variant="outlined" size="small" color="red-lighten-2" prepend-icon="mdi-file-pdf-box"
                        class="text-none font-weight-bold" title="Open official PDF in new tab"
                        :href="`/api/reports/${rep.id}/pdf`" target="_blank">
                        PDF
                      </v-btn>
                    </div>
                  </td>
                </tr>
              </tbody>
            </v-table>
          </v-card>
        </div>
      </main>
    </div>

    <!-- Snackbar Notification -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="4000">
      {{ snackbar.text }}
    </v-snackbar>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import api from '../api/client';
import EventSidebar from '../components/EventSidebar.vue';

const props = defineProps({
  id: { type: String, required: true },
});

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const eventId = computed(() => props.id || route.params.id);

const event = ref(null);
const reports = ref([]);
const canGenerate = ref(true);
const cooldownRemainingSeconds = ref(0);
const latestReportAt = ref(null);
const loading = ref(true);
const generating = ref(false);
const error = ref(null);

let cooldownTimer = null;

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

const latestReport = computed(() => {
  return reports.value.length > 0 ? reports.value[0] : null;
});

const formattedCooldown = computed(() => {
  const sec = cooldownRemainingSeconds.value;
  if (sec <= 0) return '00:00';
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
});

function formatDate(isoStr) {
  if (!isoStr) return '—';
  const d = new Date(isoStr);
  return d.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  });
}

function startCooldownTicker() {
  if (cooldownTimer) clearInterval(cooldownTimer);

  cooldownTimer = setInterval(() => {
    if (cooldownRemainingSeconds.value > 0) {
      cooldownRemainingSeconds.value -= 1;
      if (cooldownRemainingSeconds.value <= 0) {
        canGenerate.value = true;
      }
    }
  }, 1000);
}

async function loadData() {
  loading.value = true;
  error.value = null;

  try {
    const [eventsRes, reportsRes] = await Promise.all([
      api.get('/events'),
      api.get(`/events/${eventId.value}/reports`),
    ]);

    const found = eventsRes.data.find((e) => e.id === eventId.value);
    if (!found) {
      error.value = 'Event not found';
      return;
    }
    event.value = found;

    reports.value = reportsRes.data.reports || [];
    latestReportAt.value = reportsRes.data.latest_report_at;

    if (authStore.isAdmin || authStore.isStaff) {
      canGenerate.value = true;
      cooldownRemainingSeconds.value = 0;
    } else {
      canGenerate.value = reportsRes.data.can_generate;
      cooldownRemainingSeconds.value = reportsRes.data.cooldown_remaining_seconds || 0;
      if (!canGenerate.value && cooldownRemainingSeconds.value > 0) {
        startCooldownTicker();
      }
    }
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load financial reports';
  } finally {
    loading.value = false;
  }
}

async function generateReport() {
  if (!canGenerate.value && !authStore.isAdmin && !authStore.isStaff) return;

  generating.value = true;
  try {
    const res = await api.post(`/events/${eventId.value}/reports/generate`);
    showToast(`Statement ${res.data.statement_ref} generated successfully!`, 'success');

    // Prepend new report to list
    reports.value.unshift(res.data);
    if (!authStore.isAdmin && !authStore.isStaff) {
      canGenerate.value = false;
      cooldownRemainingSeconds.value = 300; // 5 minutes
      startCooldownTicker();
    } else {
      canGenerate.value = true;
      cooldownRemainingSeconds.value = 0;
    }
  } catch (err) {
    const msg = err.response?.data?.error || 'Failed to generate report';
    showToast(msg, 'error');
    if (err.response?.status === 429) {
      canGenerate.value = false;
      cooldownRemainingSeconds.value = err.response?.data?.cooldown_remaining_seconds || 300;
      startCooldownTicker();
    }
  } finally {
    generating.value = false;
  }
}

function goToEventDetails() {
  router.push(`/manage/events/${eventId.value}`);
}

onMounted(() => {
  loadData();
});

onUnmounted(() => {
  if (cooldownTimer) clearInterval(cooldownTimer);
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

.kpi-card {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.kpi-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35);
}

.highlight-card {
  border: 1px solid rgba(245, 158, 11, 0.35) !important;
  background: linear-gradient(180deg, #1e2330 0%, #181c27 100%) !important;
}

.reports-table {
  background: #131722 !important;
  border-color: #2a3142 !important;
}

.generate-btn {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.generate-btn:not(:disabled):hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 15px rgba(245, 158, 11, 0.35);
}

.border-top {
  border-top: 1px solid #2a3142;
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
