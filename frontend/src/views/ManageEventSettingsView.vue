<template>
  <v-container class="py-8" style="max-width: 900px;">
    <!-- Top Bar with Back Link -->
    <div class="d-flex align-center justify-space-between mb-6">
      <div class="d-flex align-center gap-3">
        <v-btn
          icon="mdi-arrow-left"
          variant="text"
          color="amber-darken-1"
          :to="`/manage/events/${eventId}`"
          title="Back to Event Detail"
        ></v-btn>
        <div>
          <h1 class="text-h4 font-weight-black text-white">Event Settings & Tax Info</h1>
          <p class="text-subtitle-2 text-grey mb-0">
            Configure event policy limits and business tax identity for invoicing & reporting
          </p>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <v-progress-circular indeterminate color="amber-darken-1" size="64"></v-progress-circular>
      <div class="text-body-1 text-grey mt-4">Loading event settings...</div>
    </div>

    <!-- Error State -->
    <v-alert v-else-if="error" type="error" variant="tonal" class="rounded-xl mb-6">
      {{ error }}
    </v-alert>

    <v-form v-else @submit.prevent="saveSettings">
      <!-- Section 1: General Event Settings -->
      <v-card theme="dark" class="rounded-xl border pa-6 mb-6 bg-surface">
        <div class="d-flex align-center gap-2 mb-4">
          <v-icon icon="mdi-tune" color="amber-darken-1"></v-icon>
          <span class="text-h6 font-weight-bold text-white">General Event Configuration</span>
        </div>

        <v-text-field
          v-model="form.title"
          label="Event Title *"
          variant="outlined"
          density="comfortable"
          required
          :rules="[v => !!v || 'Event title is required']"
          class="mb-3"
        ></v-text-field>

        <v-row dense>
          <v-col cols="12" sm="6">
            <v-text-field
              v-model.number="form.max_seats_per_booking"
              label="Max Seats Per Order / Checkout *"
              type="number"
              min="1"
              max="50"
              variant="outlined"
              density="comfortable"
              hint="Max seats held and purchased in a single checkout (Default: 6)"
              persistent-hint
              class="mb-3"
            ></v-text-field>
          </v-col>
          <v-col cols="12" sm="6">
            <v-text-field
              v-model.number="form.max_seats_per_user"
              label="Max Seats Per Customer / Account *"
              type="number"
              min="1"
              max="100"
              variant="outlined"
              density="comfortable"
              hint="Total cumulative seats a customer can book for this event (Default: 10)"
              persistent-hint
              class="mb-3"
            ></v-text-field>
          </v-col>
        </v-row>
      </v-card>

      <!-- Section 2: Organizer Tax & Invoicing Identity -->
      <v-card theme="dark" class="rounded-xl border pa-6 mb-6 bg-surface">
        <div class="d-flex align-center gap-2 mb-2">
          <v-icon icon="mdi-domain" color="teal-accent-3"></v-icon>
          <span class="text-h6 font-weight-bold text-white">Organizer Tax & Billing Identity</span>
        </div>
        <p class="text-caption text-grey mb-4">
          These details are displayed on customer tax invoices, digital receipts, and A4 financial settlement reports.
        </p>

        <v-row dense>
          <v-col cols="12" sm="8">
            <v-text-field
              v-model="form.tax_business_name"
              label="Legal Business / Company Name"
              placeholder="e.g. DreamCinema Live Productions Ltd."
              variant="outlined"
              density="comfortable"
              prepend-inner-icon="mdi-office-building"
              class="mb-3"
            ></v-text-field>
          </v-col>
          <v-col cols="12" sm="4">
            <v-text-field
              v-model="form.tax_id"
              label="Tax ID / VAT Registration Number"
              placeholder="e.g. TAX-8842-TSB-2026"
              variant="outlined"
              density="comfortable"
              prepend-inner-icon="mdi-card-account-details-outline"
              class="mb-3"
            ></v-text-field>
          </v-col>
        </v-row>

        <v-textarea
          v-model="form.tax_address"
          label="Registered Tax / Corporate Address"
          placeholder="e.g. 100 Theater Way, Suite 400, Los Angeles, CA 90028"
          variant="outlined"
          density="comfortable"
          rows="2"
          prepend-inner-icon="mdi-map-marker-outline"
          class="mb-3"
        ></v-textarea>

        <v-row dense>
          <v-col cols="12" sm="6">
            <v-text-field
              v-model="form.tax_email"
              label="Tax & Billing Contact Email"
              placeholder="billing@dreamcinema.com"
              type="email"
              variant="outlined"
              density="comfortable"
              prepend-inner-icon="mdi-email-outline"
              class="mb-3"
            ></v-text-field>
          </v-col>
          <v-col cols="12" sm="6">
            <v-text-field
              v-model="form.tax_phone"
              label="Tax & Billing Contact Phone"
              placeholder="+1 (555) 019-2834"
              variant="outlined"
              density="comfortable"
              prepend-inner-icon="mdi-phone-outline"
              class="mb-3"
            ></v-text-field>
          </v-col>
        </v-row>
      </v-card>

      <!-- Bottom Save Action -->
      <div class="d-flex justify-end gap-3">
        <v-btn
          variant="outlined"
          color="grey"
          class="text-none font-weight-bold px-6"
          :to="`/manage/events/${eventId}`"
        >
          Cancel
        </v-btn>
        <v-btn
          color="amber-darken-1"
          variant="flat"
          type="submit"
          class="text-none font-weight-bold px-8"
          :loading="saving"
          prepend-icon="mdi-content-save"
        >
          Save Settings & Tax Info
        </v-btn>
      </div>
    </v-form>

    <!-- Global Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3500" location="bottom right">
      {{ snackbar.text }}
      <template #actions>
        <v-btn variant="text" @click="snackbar.show = false">Close</v-btn>
      </template>
    </v-snackbar>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import api from '../api/client';

const props = defineProps({
  id: { type: String, required: false },
});

const route = useRoute();
const router = useRouter();
const eventId = props.id || route.params.id;

const loading = ref(true);
const saving = ref(false);
const error = ref(null);

const form = ref({
  title: '',
  max_seats_per_booking: 6,
  max_seats_per_user: 10,
  tax_business_name: '',
  tax_id: '',
  tax_address: '',
  tax_email: '',
  tax_phone: '',
});

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

async function loadEvent() {
  loading.value = true;
  error.value = null;
  try {
    const res = await api.get('/events');
    const found = res.data.find((e) => e.id === eventId);
    if (!found) {
      error.value = 'Event not found';
      return;
    }
    form.value = {
      title: found.title || '',
      max_seats_per_booking: found.max_seats_per_booking || 6,
      max_seats_per_user: found.max_seats_per_user || 10,
      tax_business_name: found.tax_business_name || '',
      tax_id: found.tax_id || '',
      tax_address: found.tax_address || '',
      tax_email: found.tax_email || '',
      tax_phone: found.tax_phone || '',
    };
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load event';
  } finally {
    loading.value = false;
  }
}

async function saveSettings() {
  if (!form.value.title.trim()) {
    showToast('Event title is required', 'error');
    return;
  }

  saving.value = true;
  try {
    await api.put(`/events/${eventId}`, {
      title: form.value.title.trim(),
      max_seats_per_booking: Number(form.value.max_seats_per_booking) || 6,
      max_seats_per_user: Number(form.value.max_seats_per_user) || 10,
      tax_business_name: form.value.tax_business_name?.trim() || null,
      tax_id: form.value.tax_id?.trim() || null,
      tax_address: form.value.tax_address?.trim() || null,
      tax_email: form.value.tax_email?.trim() || null,
      tax_phone: form.value.tax_phone?.trim() || null,
    });
    showToast('Event settings & tax data saved successfully!', 'success');
    setTimeout(() => {
      router.push(`/manage/events/${eventId}`);
    }, 800);
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to update event settings', 'error');
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  loadEvent();
});
</script>
