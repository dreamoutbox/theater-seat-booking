<template>
  <v-container class="py-8" style="max-width: 1100px;">
    <!-- Top Bar with Back Link -->
    <div class="d-flex align-center justify-space-between mb-6">
      <div class="d-flex align-center gap-3">
        <v-btn icon="mdi-arrow-left" variant="text" color="amber-darken-1" :to="`/showtimes/${showtimeId}/booking`"
          title="Back to Seat Selection"></v-btn>
        <div>
          <h1 class="text-h4 font-weight-black text-white">Checkout & Ticket Registration</h1>
          <p class="text-subtitle-2 text-grey mb-0">
            Provide attendee details for each ticket pass and optional tax invoice info
          </p>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <v-progress-circular indeterminate color="amber-darken-1" size="64"></v-progress-circular>
      <div class="text-body-1 text-grey mt-4">Loading your reserved seats...</div>
    </div>

    <!-- Empty State / No Held Seats -->
    <v-card v-else-if="heldSeats.length === 0" theme="dark" class="rounded-xl border pa-12 text-center bg-surface">
      <v-icon icon="mdi-seat-passenger" size="64" color="grey" class="mb-4"></v-icon>
      <h2 class="text-h5 font-weight-bold text-white mb-2">No Active Seat Holds Found</h2>
      <p class="text-grey mb-6" style="max-width: 500px; margin: 0 auto;">
        You don't have any seats held for this showtime, or your seat hold expired. Please return to the seat map to
        pick your seats.
      </p>
      <v-btn color="amber-darken-1" variant="flat" size="large" class="text-none font-weight-bold"
        prepend-icon="mdi-seat-outline" :to="`/showtimes/${showtimeId}/booking`">
        Return to Seat Map
      </v-btn>
    </v-card>

    <!-- Main Checkout Form & Summary Grid -->
    <v-row v-else>
      <!-- Left Column: Forms -->
      <v-col cols="12" md="7">
        <v-form @submit.prevent="handleCompletePayment">
          <!-- Showtime Information Banner -->
          <v-card theme="dark" class="rounded-xl border pa-4 mb-6 bg-surface-variant">
            <div class="d-flex align-center gap-3">
              <v-icon icon="mdi-movie-open" color="amber-darken-1" size="32"></v-icon>
              <div>
                <div class="text-h6 font-weight-bold text-white">{{ showtime?.event_title }}</div>
                <div class="text-caption text-grey">
                  {{ showtime?.theater_name }} • {{ formatDate(showtime?.starts_at) }}
                </div>
              </div>
            </div>
          </v-card>

          <!-- Step 1: Attendee Details for each seat -->
          <v-card theme="dark" class="rounded-xl border pa-6 mb-6 bg-surface">
            <div class="d-flex align-center gap-2 mb-2">
              <v-icon icon="mdi-account-group" color="amber-darken-1"></v-icon>
              <span class="text-h6 font-weight-bold text-white">Ticket Attendee Details</span>
            </div>
            <p class="text-caption text-grey mb-4">
              Enter the name and contact phone for the person using each ticket pass at entry gate.
            </p>

            <div v-for="(seat, idx) in heldSeats" :key="seat.id" class="pa-4 rounded-lg bg-surface-variant border mb-4">
              <div class="d-flex justify-space-between align-center mb-3">
                <div class="d-flex align-center gap-2">
                  <v-chip size="small" :color="seat.zone_color || 'amber-darken-1'" variant="flat"
                    class="font-weight-bold">
                    Row {{ seat.row_label }}, Seat {{ seat.seat_number }}
                  </v-chip>
                  <span class="text-caption text-grey font-weight-bold">{{ seat.zone_name }}</span>
                </div>
                <span class="text-body-2 font-weight-bold font-weight-mono text-white">
                  {{ (seat.price_cents / 100).toFixed(2) }}
                </span>
              </div>

              <v-row dense>
                <v-col cols="12" sm="6">
                  <v-text-field v-model="attendees[seat.id].name" :label="`Attendee ${idx + 1} Full Name *`"
                    placeholder="e.g. John Doe" variant="outlined" density="comfortable" required
                    :rules="[v => !!v || 'Attendee name is required']" prepend-inner-icon="mdi-account"></v-text-field>
                </v-col>
                <v-col cols="12" sm="6">
                  <v-text-field v-model="attendees[seat.id].phone" :label="`Attendee ${idx + 1} Mobile Phone *`"
                    placeholder="e.g. +1 555-019-2834" variant="outlined" density="comfortable" required
                    :rules="[v => !!v || 'Attendee phone is required']" prepend-inner-icon="mdi-phone"></v-text-field>
                </v-col>
              </v-row>
            </div>
          </v-card>

          <!-- Step 2: Customer Tax & Billing Information -->
          <v-card theme="dark" class="rounded-xl border pa-6 mb-6 bg-surface">
            <div class="d-flex align-center justify-space-between mb-2">
              <div class="d-flex align-center gap-2">
                <v-icon icon="mdi-file-document-outline" color="teal-accent-3"></v-icon>
                <span class="text-h6 font-weight-bold text-white">Tax Invoice & Billing Information</span>
              </div>
              <v-checkbox v-model="includeTaxInfo" label="Request Tax Invoice" color="amber-darken-1" hide-details
                density="compact" class="font-weight-bold"></v-checkbox>
            </div>
            <p class="text-caption text-grey mb-4">
              Optional corporate or personal tax identification to be printed on your official tax invoice.
            </p>

            <v-expand-transition>
              <div v-if="includeTaxInfo" class="pt-2">
                <v-row dense>
                  <v-col cols="12" sm="7">
                    <v-text-field v-model="billingForm.billing_name" label="Company / Individual Tax Name *"
                      placeholder="e.g. Acme Corporation or Jane Doe" variant="outlined" density="comfortable"
                      prepend-inner-icon="mdi-domain" class="mb-3"></v-text-field>
                  </v-col>
                  <v-col cols="12" sm="5">
                    <v-text-field v-model="billingForm.tax_id" label="Tax ID / VAT Registration No. *"
                      placeholder="e.g. US-987654321" variant="outlined" density="comfortable"
                      prepend-inner-icon="mdi-card-account-details-outline" class="mb-3"></v-text-field>
                  </v-col>
                </v-row>

                <v-textarea v-model="billingForm.billing_address" label="Registered Billing Address"
                  placeholder="e.g. 500 5th Ave, Suite 1200, New York, NY 10110" variant="outlined"
                  density="comfortable" rows="2" prepend-inner-icon="mdi-map-marker-outline" class="mb-3"></v-textarea>
              </div>
            </v-expand-transition>
          </v-card>
        </v-form>
      </v-col>

      <!-- Right Column: Order Summary & Pay Button -->
      <v-col cols="12" md="5">
        <v-card theme="dark" class="rounded-xl border pa-6 bg-surface position-sticky" style="top: 24px;">
          <h2 class="text-h6 font-weight-bold text-white mb-4">Order Summary</h2>

          <!-- Line items -->
          <div class="d-flex flex-column gap-2 mb-4">
            <div v-for="seat in heldSeats" :key="seat.id"
              class="d-flex justify-space-between align-center py-2 border-bottom">
              <div>
                <span class="text-body-2 font-weight-bold text-white">
                  Seat {{ seat.row_label }}{{ seat.seat_number }}
                </span>
                <span class="text-caption text-grey ml-2">({{ seat.zone_name }})</span>
                <div v-if="attendees[seat.id]?.name" class="text-caption text-grey-lighten-1">
                  👤 {{ attendees[seat.id].name }}
                </div>
              </div>
              <span class="text-body-2 font-weight-mono font-weight-bold text-white">
                {{ (seat.price_cents / 100).toFixed(2) }}
              </span>
            </div>
          </div>

          <!-- Price Calculations -->
          <div class="d-flex justify-space-between align-center mb-2 text-caption text-grey">
            <span>Seats Subtotal ({{ heldSeats.length }}):</span>
            <span class="font-weight-mono">{{ (totalCents / 100).toFixed(2) }}</span>
          </div>
          <div class="d-flex justify-space-between align-center mb-3 text-caption text-grey">
            <span>Merchant & Processing Fee:</span>
            <span class="text-success font-weight-bold">0.00 (Included)</span>
          </div>

          <v-divider class="my-4"></v-divider>

          <div class="d-flex justify-space-between align-center mb-6">
            <span class="text-h6 font-weight-bold text-white">Total Due:</span>
            <span class="text-h4 font-weight-black text-amber-darken-1 font-weight-mono">
              {{ (totalCents / 100).toFixed(2) }}
            </span>
          </div>

          <!-- Pay & Confirm Button -->
          <v-btn color="amber-darken-1" variant="flat" block size="x-large" class="text-none font-weight-bold mb-3"
            :loading="paying" prepend-icon="mdi-credit-card-check" @click="handleCompletePayment">
            Pay & Confirm ({{ (totalCents / 100).toFixed(2) }})
          </v-btn>

          <div class="text-caption text-grey text-center">
            🔒 Instant 256-bit encrypted checkout & QR pass generation
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
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import api from '../api/client';
import { useAuthStore } from '../stores/auth';

const props = defineProps({
  showtimeId: { type: String, required: false },
});

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const showtimeId = props.showtimeId || route.params.showtimeId;

const loading = ref(true);
const paying = ref(false);
const showtime = ref(null);
const heldSeats = ref([]);
const attendees = ref({});

const includeTaxInfo = ref(false);
const billingForm = ref({
  billing_name: '',
  tax_id: '',
  billing_address: '',
});

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
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

const totalCents = computed(() => {
  return heldSeats.value.reduce((sum, s) => sum + (s.price_cents || 0), 0);
});

async function loadCheckoutData() {
  loading.value = true;
  try {
    const [seatsRes, showtimesRes] = await Promise.all([
      api.get(`/showtimes/${showtimeId}/seats`),
      api.get('/showtimes'),
    ]);

    const foundSt = showtimesRes.data.find((s) => s.id === showtimeId);
    showtime.value = foundSt || null;

    const myHolds = (seatsRes.data || []).filter(
      (s) => s.status === 'held' && (s.is_mine === true || s.my_hold === true)
    );
    heldSeats.value = myHolds;

    // Initialize attendee fields for each held seat
    const userEmail = authStore.user?.email || '';
    const defaultName = userEmail ? userEmail.split('@')[0] : '';
    const map = {};
    for (const seat of myHolds) {
      map[seat.id] = {
        name: attendees.value[seat.id]?.name || defaultName,
        phone: attendees.value[seat.id]?.phone || '',
      };
    }
    attendees.value = map;
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to load checkout details', 'error');
  } finally {
    loading.value = false;
  }
}

async function handleCompletePayment() {
  if (heldSeats.value.length === 0) {
    showToast('No held seats to purchase', 'warning');
    return;
  }

  // Validate attendee inputs
  for (const seat of heldSeats.value) {
    const att = attendees.value[seat.id];
    if (!att?.name?.trim()) {
      showToast(`Please enter Attendee Name for Row ${seat.row_label}, Seat ${seat.seat_number}`, 'warning');
      return;
    }
    if (!att?.phone?.trim()) {
      showToast(`Please enter Attendee Phone for Row ${seat.row_label}, Seat ${seat.seat_number}`, 'warning');
      return;
    }
  }

  if (includeTaxInfo.value && !billingForm.value.billing_name.trim()) {
    showToast('Please provide Company / Individual Name for the tax invoice', 'warning');
    return;
  }

  paying.value = true;

  try {
    // 1. Build seats payload with attendee details
    const seatItems = heldSeats.value.map((seat) => ({
      seat_booking_id: seat.seat_booking_id || seat.booking_id || seat.id,
      attendee_name: attendees.value[seat.id]?.name?.trim() || null,
      attendee_phone: attendees.value[seat.id]?.phone?.trim() || null,
    }));

    // 2. Create Order
    const orderRes = await api.post('/orders', {
      seats: seatItems,
      billing_name: includeTaxInfo.value ? billingForm.value.billing_name.trim() : null,
      tax_id: includeTaxInfo.value ? billingForm.value.tax_id.trim() : null,
      billing_address: includeTaxInfo.value ? billingForm.value.billing_address.trim() : null,
    });

    const orderId = orderRes.data.order_id;

    // 3. Create Stripe Checkout Session → redirect to Stripe-hosted payment page
    const sessionRes = await api.post(`/orders/${orderId}/checkout-session`);
    const checkoutUrl = sessionRes.data.checkout_url;

    showToast('Redirecting to secure payment...', 'info');
    // Redirect to Stripe Checkout (or mock URL)
    window.location.href = checkoutUrl;
  } catch (err) {
    showToast(err.response?.data?.error || 'Checkout failed. Please try again.', 'error');
    paying.value = false;
  }
}

onMounted(() => {
  loadCheckoutData();
});
</script>

<style scoped>
.gap-2 {
  gap: 8px;
}

.gap-3 {
  gap: 12px;
}

.border-bottom {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}
</style>
