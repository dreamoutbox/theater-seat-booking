<template>
  <div class="report-wrapper py-6">
    <!-- Non-printable Top Action Bar -->
    <v-container class="no-print mb-4 max-w-5xl">
      <div class="d-flex justify-space-between align-center flex-wrap gap-3">
        <div class="d-flex align-center gap-2">
          <v-btn
            variant="text"
            prepend-icon="mdi-arrow-left"
            :to="`/manage/events/${props.id || route.params.id}`"
            class="text-none font-weight-bold"
          >
            Back to Event Details
          </v-btn>
        </div>

        <div class="d-flex align-center gap-3">
          <v-btn
            variant="outlined"
            prepend-icon="mdi-refresh"
            size="small"
            :loading="loading"
            @click="loadReportData"
            class="text-none font-weight-bold"
          >
            Refresh
          </v-btn>
          <v-btn
            color="amber-darken-1"
            variant="flat"
            prepend-icon="mdi-printer"
            size="small"
            class="text-none font-weight-bold"
            @click="printReport"
          >
            Print / Save as PDF (A4)
          </v-btn>
        </div>
      </div>
    </v-container>

    <!-- Loading State -->
    <v-container v-if="loading" class="no-print max-w-5xl text-center py-12">
      <v-progress-circular indeterminate color="amber-darken-1" size="56"></v-progress-circular>
      <div class="text-body-1 text-grey mt-4">Generating Financial Statement & Tax Report...</div>
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
              <td style="width: 28%;" class="font-bold font-mono">USD ($)</td>
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
              <th class="text-right" style="width: 180px;">Amount (USD)</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <span class="font-bold">Gross Ticket Sales Revenue</span>
                <div class="text-small text-muted">Total value of all customer orders placed and paid</div>
              </td>
              <td class="text-center font-mono">{{ metrics.total_tickets_created || 0 }} Tickets Issued</td>
              <td class="text-right font-mono font-bold">${{ ((metrics.gross_sales_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
            <tr class="row-danger">
              <td>
                <span class="font-bold text-danger">Less: Ticket Returns & Refunds</span>
                <div class="text-small text-muted">Refunded transactions reversed back to customers</div>
              </td>
              <td class="text-center font-mono text-danger">{{ metrics.refunded_tickets || 0 }} Refunded</td>
              <td class="text-right font-mono font-bold text-danger">-${{ ((metrics.total_refunds_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
            <tr class="row-highlight">
              <td>
                <span class="font-bold">Net Gross Ticket Sales (Taxable Base Revenue)</span>
                <div class="text-small text-muted">Gross sales revenue after deducting all refunds</div>
              </td>
              <td class="text-center font-mono font-bold">{{ (metrics.active_tickets || 0) + (metrics.used_tickets || 0) }} Net Valid</td>
              <td class="text-right font-mono font-bold text-large">${{ ((metrics.net_profit_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
            <tr>
              <td>
                <span>Payment Processing & Merchant Gateway Fees</span>
                <div class="text-small text-muted">Stripe card transaction processing (Included / Absorbed)</div>
              </td>
              <td class="text-center font-mono">—</td>
              <td class="text-right font-mono text-muted">$0.00</td>
            </tr>
            <tr class="row-total">
              <td>
                <span class="font-bold">NET ORGANIZER SETTLEMENT AMOUNT</span>
                <div class="text-small text-muted">Final net payable amount for tax reporting and bank disbursement</div>
              </td>
              <td class="text-center font-mono font-bold">100% Settled</td>
              <td class="text-right font-mono font-bold text-large text-primary">
                ${{ ((metrics.net_profit_cents || 0) / 100).toFixed(2) }}
              </td>
            </tr>
          </tbody>
        </table>

        <!-- Showtime-by-Showtime Financial Breakdown -->
        <div class="section-title mb-2">2. SHOWTIME SESSION BREAKDOWN & ATTENDANCE AUDIT</div>
        <table class="report-table mb-6">
          <thead>
            <tr>
              <th class="text-left">Session Date & Venue</th>
              <th class="text-center">Capacity</th>
              <th class="text-center">Sold</th>
              <th class="text-center">Checked-In</th>
              <th class="text-center">Refunded</th>
              <th class="text-right">Gross Sales</th>
              <th class="text-right">Refunds</th>
              <th class="text-right">Net Revenue</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="showtimes.length === 0">
              <td colspan="8" class="text-center py-3 text-muted">No sessions recorded for this event.</td>
            </tr>
            <tr v-for="st in showtimes" :key="st.showtime_id">
              <td>
                <div class="font-bold">{{ formatDate(st.starts_at) }}</div>
                <div class="text-small text-muted">{{ st.theater_name }}</div>
              </td>
              <td class="text-center font-mono">{{ st.total_seats }}</td>
              <td class="text-center font-mono font-bold">{{ st.sold_seats }}</td>
              <td class="text-center font-mono">
                {{ st.used_tickets }}
                <span class="text-small text-muted">({{ st.sold_seats > 0 ? Math.round((st.used_tickets / st.sold_seats) * 100) : 0 }}%)</span>
              </td>
              <td class="text-center font-mono" :class="{ 'text-danger font-bold': st.refunded_tickets > 0 }">
                {{ st.refunded_tickets }}
              </td>
              <td class="text-right font-mono">${{ (st.gross_cents / 100).toFixed(2) }}</td>
              <td class="text-right font-mono" :class="{ 'text-danger': st.refunded_cents > 0 }">
                {{ st.refunded_cents > 0 ? `-$${(st.refunded_cents / 100).toFixed(2)}` : '$0.00' }}
              </td>
              <td class="text-right font-mono font-bold">${{ (st.net_cents / 100).toFixed(2) }}</td>
            </tr>
          </tbody>
          <tfoot>
            <tr class="table-summary-footer">
              <td class="font-bold">TOTALS</td>
              <td class="text-center font-mono font-bold">{{ totalCapacity }}</td>
              <td class="text-center font-mono font-bold">{{ (metrics.active_tickets || 0) + (metrics.used_tickets || 0) }}</td>
              <td class="text-center font-mono font-bold">{{ metrics.used_tickets || 0 }} ({{ metrics.attendance_rate_percent || 0 }}%)</td>
              <td class="text-center font-mono font-bold text-danger">{{ metrics.refunded_tickets || 0 }}</td>
              <td class="text-right font-mono font-bold">${{ ((metrics.gross_sales_cents || 0) / 100).toFixed(2) }}</td>
              <td class="text-right font-mono font-bold text-danger">-${{ ((metrics.total_refunds_cents || 0) / 100).toFixed(2) }}</td>
              <td class="text-right font-mono font-bold text-primary">${{ ((metrics.net_profit_cents || 0) / 100).toFixed(2) }}</td>
            </tr>
          </tfoot>
        </table>

        <!-- Signature & Verification Footer -->
        <div class="report-signatures pt-4 border-top">
          <div class="d-flex justify-space-between align-end">
            <div class="sign-box text-center">
              <div class="sign-line mb-1"></div>
              <div class="text-small font-bold">Event Organizer / Tax Filer</div>
              <div class="text-small text-muted">Authorized Signature & Date</div>
            </div>
            <div class="qr-verify text-center">
              <div class="text-small font-bold text-muted">DreamCinema Automated Clearance</div>
              <div class="text-small text-muted">System Verified & Reconciled</div>
            </div>
            <div class="sign-box text-center">
              <div class="sign-line mb-1"></div>
              <div class="text-small font-bold">Cinema Financial Auditor</div>
              <div class="text-small text-muted">Official Verification & Stamp</div>
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
});

const route = useRoute();
const loading = ref(true);
const error = ref(null);
const event = ref(null);
const showtimes = ref([]);
const organizerEmail = ref('');
const reportGeneratedAt = ref(new Date());

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
  return date.toLocaleString(undefined, {
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
  const eventId = props.id || route.params.id;
  loading.value = true;
  error.value = null;
  reportGeneratedAt.value = new Date();

  try {
    const [eventsRes, analyticsRes] = await Promise.all([
      api.get('/events'),
      api.get('/analytics/dashboard', { params: { event_id: eventId } }),
    ]);

    const found = eventsRes.data.find((e) => e.id === eventId);
    if (!found) {
      error.value = 'Event not found';
      return;
    }
    event.value = found;
    metrics.value = analyticsRes.data.metrics || {};
    showtimes.value = analyticsRes.data.showtimes || [];
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
