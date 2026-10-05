<template>
  <div class="report-wrapper py-6">
    <!-- Non-printable Top Action Bar -->
    <v-container class="no-print mb-4 max-w-5xl">
      <div class="d-flex justify-space-between align-center flex-wrap gap-3">
        <div class="d-flex align-center gap-2">
          <v-btn variant="text" prepend-icon="mdi-arrow-left" :to="`/manage/events/${eventId}/financial-reports`"
            class="text-none font-weight-bold">
            Back to Report Archive
          </v-btn>
        </div>

        <div class="d-flex align-center gap-3">
          <v-btn variant="outlined" prepend-icon="mdi-refresh" size="small" :loading="loading" @click="loadReportData"
            class="text-none font-weight-bold">
            Refresh
          </v-btn>
          <v-btn color="amber-darken-1" variant="flat" prepend-icon="mdi-printer" size="small"
            class="text-none font-weight-bold" @click="printReport">
            Print (A4)
          </v-btn>
          <v-btn v-if="reportIdParam" color="red-lighten-2" variant="tonal" prepend-icon="mdi-file-pdf-box" size="small"
            class="text-none font-weight-bold" :href="`/api/reports/${reportIdParam}/pdf`" target="_blank">
            Download PDF
          </v-btn>
        </div>
      </div>
    </v-container>

    <!-- Loading State -->
    <v-container v-if="loading" class="no-print max-w-5xl text-center py-12">
      <v-progress-circular indeterminate color="amber-darken-1" size="56"></v-progress-circular>
      <div class="text-body-1 text-grey mt-4">Loading Financial Statement & Tax Report Snapshot...</div>
    </v-container>

    <!-- Error State -->
    <v-container v-else-if="error" class="no-print max-w-5xl">
      <v-alert type="error" variant="tonal" class="rounded-xl">
        {{ error }}
      </v-alert>
    </v-container>

    <!-- A4 Paper Document Container -->
    <div v-else class="a4-document-container">
      <div class="a4-sheet" id="printable-report">
        <!-- Header -->
        <div class="report-header pb-4 mb-5 border-bottom">
          <div class="d-flex justify-space-between align-start">
            <div>
              <div class="company-logo-text mb-1">
                DREAM<span class="text-amber">CINEMA</span>
              </div>
              <div class="text-small text-muted">Box Office & Ticket Settlement Platform</div>
              <div class="text-small text-muted">Tax Identification / Reg. No: TAX-8842-TSB-2026</div>
            </div>
            <div class="text-right">
              <h1 class="report-title text-uppercase font-bold">Financial & Tax Statement</h1>
              <div class="report-id font-mono">Statement Ref: {{ statementRef }}</div>
              <div class="text-small text-muted">Generated on: {{ formatFullDate(reportGeneratedAt) }}</div>
            </div>
          </div>
        </div>

        <!-- Event & Accounting Specification Table -->
        <table class="report-table mb-5">
          <tbody>
            <tr>
              <th class="text-left bg-light-gray" style="width: 18%;">Event Title:</th>
              <td style="width: 32%;" class="font-bold">{{ event?.title }}</td>
              <th class="text-left bg-light-gray" style="width: 22%;">Accounting Currency:</th>
              <td style="width: 28%;" class="font-bold font-mono">{{ currency }}</td>
            </tr>
            <tr>
              <th class="text-left bg-light-gray">Event ID:</th>
              <td class="font-mono text-small">#{{ event?.id }}</td>
              <th class="text-left bg-light-gray">Total Sessions:</th>
              <td>{{ showtimes.length }} Scheduled Showtime(s)</td>
            </tr>
            <tr>
              <th class="text-left bg-light-gray">Organizer Account:</th>
              <td>{{ organizerEmail || 'Primary Cinema Administration' }}</td>
              <th class="text-left bg-light-gray">Period Coverage:</th>
              <td>{{ eventDateRange }}</td>
            </tr>
          </tbody>
        </table>

        <!-- Executive Financial Summary Table for Tax Filing -->
        <div class="section-title mb-2">1. EXECUTIVE FINANCIAL & TAX SUMMARY</div>
        <table class="report-table mb-5">
          <thead>
            <tr>
              <th class="text-left">Revenue & Expense Breakdown Line Items</th>
              <th class="text-center" style="width: 140px;">Quantity / Units</th>
              <th class="text-right" style="width: 180px;">Amount ({{ currency }})</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <span class="font-bold">Gross Ticket Sales Revenue</span>
                <div class="text-small text-muted">Total value of all customer orders placed and paid</div>
              </td>
              <td class="text-center font-mono">{{ metrics.total_tickets_created || 0 }} Tickets Issued</td>
              <td class="text-right font-mono font-bold">{{ ((metrics.gross_sales_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
            <tr class="row-danger">
              <td>
                <span class="font-bold text-danger">Less: Customer Refunds & Inventory Releases</span>
                <div class="text-small text-muted">Returned seat bookings and reversed charges</div>
              </td>
              <td class="text-center font-mono text-danger">{{ metrics.refunded_tickets || 0 }} Tickets Refunded</td>
              <td class="text-right font-mono font-bold text-danger">-{{ ((metrics.total_refunds_cents || 0) /
                100).toFixed(2) }}</td>
            </tr>
            <tr class="row-highlight bg-light-gray">
              <td>
                <span class="font-bold">Net Operating Ticket Profit (Before Tax)</span>
                <div class="text-small text-muted">Gross sales revenue minus refunded transactions</div>
              </td>
              <td class="text-center font-mono">{{ metrics.active_tickets || 0 }} Active Tickets</td>
              <td class="text-right font-mono font-bold">{{ ((metrics.net_profit_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
            <tr>
              <td>
                <span class="font-bold text-muted">Statutory Tax Withholding (10.0%)</span>
                <div class="text-small text-muted">Estimated entertainment & business transaction tax liability</div>
              </td>
              <td class="text-center font-mono text-muted">Standard Rate</td>
              <td class="text-right font-mono font-bold text-muted">- {{ (((metrics.net_profit_cents || 0) * 0.10) /
                100).toFixed(2) }}</td>
            </tr>
            <tr class="row-total">
              <td class="font-bold text-large">
                FINAL SETTLEMENT PAYOUT
                <div class="text-small text-muted font-normal">Disbursable funds after statutory withholding</div>
              </td>
              <td class="text-center font-mono text-large font-bold">100% Cleared</td>
              <td class="text-right font-mono text-large font-bold text-primary">
                {{ (((metrics.net_profit_cents || 0) * 0.90) / 100).toFixed(2) }}
              </td>
            </tr>
          </tbody>
        </table>

        <!-- Showtime & Hall Attendance Section -->
        <div class="section-title mb-2">2. SHOWTIME SESSIONS & ATTENDANCE AUDIT</div>
        <table class="report-table mb-5">
          <thead>
            <tr>
              <th class="text-left">Session ID & Starts At</th>
              <th class="text-left">Theater Hall</th>
              <th class="text-center" style="width: 100px;">Total Seats</th>
              <th class="text-center" style="width: 100px;">Sold</th>
              <th class="text-center" style="width: 100px;">Scanned</th>
              <th class="text-right" style="width: 130px;">Net Revenue</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="st in showtimes" :key="st.showtime_id">
              <td>
                <div class="font-bold">{{ formatDate(st.starts_at) }}</div>
                <div class="text-small font-mono text-muted">#{{ st.showtime_id.substring(0, 8) }}</div>
              </td>
              <td>
                <div>{{ st.theater_name }}</div>
                <div class="text-small text-muted">Grand Horizon Hall</div>
              </td>
              <td class="text-center font-mono">{{ st.total_seats || 0 }}</td>
              <td class="text-center font-mono font-bold">{{ st.sold_seats || 0 }}</td>
              <td class="text-center font-mono text-success">{{ st.used_tickets || 0 }}</td>
              <td class="text-right font-mono font-bold">{{ ((st.net_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
            <tr v-if="showtimes.length === 0">
              <td colspan="6" class="text-center text-muted py-3">No showtimes recorded for this event.</td>
            </tr>
          </tbody>
          <tfoot v-if="showtimes.length > 0">
            <tr class="bg-light-gray font-bold">
              <td colspan="2">Consolidated Totals</td>
              <td class="text-center font-mono">{{ totalCapacity }}</td>
              <td class="text-center font-mono">{{ metrics.active_tickets || 0 }}</td>
              <td class="text-center font-mono text-success">{{ metrics.used_tickets || 0 }}</td>
              <td class="text-right font-mono">{{ ((metrics.net_profit_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
          </tfoot>
        </table>

        <!-- Admission Performance Metrics -->
        <div class="section-title mb-2">3. GATE ENTRY & ADMISSION VERIFICATION</div>
        <table class="report-table mb-5">
          <tbody>
            <tr>
              <th class="text-left bg-light-gray" style="width: 25%;">Total Tickets Issued:</th>
              <td style="width: 25%;" class="font-mono font-bold">{{ metrics.total_tickets_created || 0 }}</td>
              <th class="text-left bg-light-gray" style="width: 25%;">Active Valid Passes:</th>
              <td style="width: 25%;" class="font-mono font-bold">{{ metrics.active_tickets || 0 }}</td>
            </tr>
            <tr>
              <th class="text-left bg-light-gray">Gate Checked-In (Scanned):</th>
              <td class="font-mono font-bold">{{ metrics.used_tickets || 0 }}</td>
              <th class="text-left bg-light-gray">Overall Attendance Rate:</th>
              <td class="font-mono font-bold">{{ metrics.attendance_rate_percent || 0 }}%</td>
            </tr>
          </tbody>
        </table>

        <!-- Certification & Sign-off -->
        <div class="report-footer mt-6 pt-4 border-top">
          <div class="row d-flex justify-space-between">
            <div style="width: 45%;">
              <div class="text-small text-muted mb-4">Authorized By Platform Custodian:</div>
              <div class="signature-line pb-1 mb-1">
                <span class="signature-text">DreamCinema Digital Settlement System</span>
              </div>
              <div class="text-small text-muted">Automated Financial Audit Seal • ID: DC-TSB-SYS</div>
            </div>
            <div style="width: 45%;" class="text-right">
              <div class="text-small text-muted mb-4">Event Organizer / Tax Filer Verification:</div>
              <div class="signature-line pb-1 mb-1 text-right">
                <span class="signature-text">{{ organizerEmail || 'Event Authorized Representative' }}</span>
              </div>
              <div class="text-small text-muted">Confirmed on behalf of Event Production Entity</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import api from '../api/client';

const props = defineProps({
  id: { type: String, required: true },
  reportId: { type: String, default: null },
});

const route = useRoute();
const eventId = computed(() => props.id || route.params.id);
const reportIdParam = computed(() => props.reportId || route.params.reportId);

const loading = ref(true);
const error = ref(null);
const event = ref(null);
const showtimes = ref([]);
const organizerEmail = ref('');
const currency = ref('THB');
const reportGeneratedAt = ref(new Date());
const rawStatementRef = ref('');

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

function printReport() {
  window.print();
}

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

function formatFullDate(date) {
  const d = date instanceof Date ? date : new Date(date);
  return d.toLocaleString(undefined, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
}

const totalCapacity = computed(() => {
  return showtimes.value.reduce((sum, st) => sum + (st.total_seats || 0), 0);
});

const statementRef = computed(() => {
  if (rawStatementRef.value) return rawStatementRef.value;
  const d = reportGeneratedAt.value || new Date();
  const yyyy = d.getFullYear();
  const mm = String(d.getMonth() + 1).padStart(2, '0');
  const seq = event.value?.event_number != null ? event.value.event_number : 1;
  return `FIN-${yyyy}-${mm}-${seq}`;
});

const eventDateRange = computed(() => {
  if (showtimes.value.length === 0) return 'No showtimes scheduled';
  const sorted = [...showtimes.value].sort((a, b) => new Date(a.starts_at) - new Date(b.starts_at));
  const first = formatDate(sorted[0].starts_at);
  const last = formatDate(sorted[sorted.length - 1].starts_at);
  return first === last ? first : `${first} — ${last}`;
});

async function loadReportData() {
  loading.value = true;
  error.value = null;

  try {
    if (reportIdParam.value) {
      // Load immutable saved snapshot report
      const repRes = await api.get(`/reports/${reportIdParam.value}`);
      const rep = repRes.data;
      rawStatementRef.value = rep.statement_ref;
      reportGeneratedAt.value = new Date(rep.generated_at);

      const snap = rep.report_data || {};
      currency.value = snap.currency || 'THB';
      event.value = snap.event || { id: rep.event_id, title: 'Event' };
      organizerEmail.value = snap.organizer_email || rep.generated_by_email || '';
      metrics.value = snap.metrics || {
        gross_sales_cents: rep.gross_sales_cents,
        total_refunds_cents: rep.total_refunds_cents,
        net_profit_cents: rep.net_profit_cents,
        total_tickets_created: rep.tickets_issued,
        active_tickets: rep.tickets_issued - rep.tickets_refunded,
        used_tickets: rep.tickets_scanned,
        refunded_tickets: rep.tickets_refunded,
        attendance_rate_percent: rep.attendance_rate_percent,
      };
      showtimes.value = snap.showtimes || [];
    } else {
      // Fallback: load latest report for event or live data
      const reportsRes = await api.get(`/events/${eventId.value}/reports`);
      if (reportsRes.data.reports && reportsRes.data.reports.length > 0) {
        const latest = reportsRes.data.reports[0];
        rawStatementRef.value = latest.statement_ref;
        reportGeneratedAt.value = new Date(latest.generated_at);
        const snap = latest.report_data || {};
        currency.value = snap.currency || reportsRes.data.currency || 'THB';
        event.value = snap.event || { id: latest.event_id, title: 'Event' };
        organizerEmail.value = snap.organizer_email || latest.generated_by_email || '';
        metrics.value = snap.metrics || {};
        showtimes.value = snap.showtimes || [];
      } else {
        // If no reports exist yet, load live data
        const [eventsRes, analyticsRes] = await Promise.all([
          api.get('/events'),
          api.get('/analytics/dashboard', { params: { event_id: eventId.value } }),
        ]);

        const found = eventsRes.data.find((e) => e.id === eventId.value);
        if (!found) {
          error.value = 'Event not found';
          return;
        }
        event.value = found;
        currency.value = analyticsRes.data.currency || 'THB';
        metrics.value = analyticsRes.data.metrics || {};
        showtimes.value = analyticsRes.data.showtimes || [];
        reportGeneratedAt.value = new Date();
      }
    }
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to generate financial statement';
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadReportData();
});
</script>

<style scoped>
.report-wrapper {
  background-color: #0f131c;
  min-height: 100vh;
}

.max-w-5xl {
  max-width: 1050px;
}

/* A4 Document styling */
.a4-document-container {
  display: flex;
  justify-content: center;
  padding-bottom: 40px;
}

.a4-sheet {
  background: #ffffff;
  color: #1e293b;
  width: 210mm;
  min-height: 297mm;
  padding: 20mm 20mm;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-size: 11pt;
  line-height: 1.4;
  box-sizing: border-box;
}

/* Typography & Layout */
.company-logo-text {
  font-size: 18pt;
  font-weight: 900;
  letter-spacing: -0.5px;
  color: #0f172a;
}

.text-amber {
  color: #d97706;
}

.report-title {
  font-size: 14pt;
  color: #0f172a;
  margin: 0;
  letter-spacing: 0.5px;
}

.report-id {
  font-size: 9.5pt;
  color: #475569;
  font-weight: 600;
}

.section-title {
  font-size: 10.5pt;
  font-weight: 800;
  color: #0f172a;
  letter-spacing: 0.5px;
  border-left: 3px solid #d97706;
  padding-left: 6px;
}

.text-small {
  font-size: 8.5pt;
}

.text-large {
  font-size: 12pt;
}

.text-muted {
  color: #64748b;
}

.text-danger {
  color: #dc2626 !important;
}

.text-primary {
  color: #0f172a !important;
}

.font-bold {
  font-weight: 700;
}

.font-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.border-bottom {
  border-bottom: 2px solid #e2e8f0;
}

.border-top {
  border-top: 1.5px solid #cbd5e1;
}

.bg-light-gray {
  background-color: #f8fafc;
}

.border {
  border: 1px solid #e2e8f0;
}

.grid-2col {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.meta-row {
  margin-bottom: 4px;
}

.meta-label {
  font-size: 9pt;
  color: #64748b;
  width: 140px;
  display: inline-block;
}

.meta-val {
  font-size: 9.5pt;
  color: #0f172a;
}

/* Report Table */
.report-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 9pt;
}

.report-table th {
  background-color: #f1f5f9;
  color: #334155;
  font-weight: 700;
  padding: 8px 10px;
  border: 1px solid #cbd5e1;
  text-transform: uppercase;
  font-size: 8pt;
  letter-spacing: 0.5px;
}

.report-table td {
  padding: 8px 10px;
  border: 1px solid #e2e8f0;
  vertical-align: middle;
}

.row-danger {
  background-color: #fef2f2;
}

.row-highlight {
  background-color: #f8fafc;
  font-weight: 600;
}

.row-total {
  background-color: #f1f5f9;
  border-top: 2px solid #94a3b8;
}

.table-summary-footer td {
  background-color: #f1f5f9;
  border: 1px solid #cbd5e1;
  padding: 8px 10px;
}

/* Signatures */
.sign-box {
  width: 180px;
}

.sign-line {
  border-bottom: 1px solid #94a3b8;
  height: 45px;
}

.gap-2 {
  gap: 8px;
}

.gap-3 {
  gap: 12px;
}

/* Print CSS rules */
@media print {
  body {
    background: #ffffff !important;
    color: #000000 !important;
  }

  .report-wrapper {
    background: transparent !important;
    padding: 0 !important;
  }

  .no-print {
    display: none !important;
  }

  .a4-document-container {
    padding: 0 !important;
  }

  .a4-sheet {
    box-shadow: none !important;
    padding: 10mm 15mm !important;
    width: 100% !important;
    min-height: auto !important;
  }

  @page {
    size: A4 portrait;
    margin: 10mm;
  }
}
</style>
