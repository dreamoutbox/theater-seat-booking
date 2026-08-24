<template>
  <v-container fluid class="py-6 px-4 px-md-6 px-lg-8">
    <!-- Top Bar with Back Link -->
    <div class="d-flex align-center justify-space-between mb-4">
      <div class="d-flex align-center gap-3">
        <v-btn variant="text" prepend-icon="mdi-arrow-left" :to="`/manage/events/${eventId}`"
          class="text-none font-weight-bold">
          Back to Event Details
        </v-btn>
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

    <v-row v-else-if="event" class="mt-1">
      <!-- Sidebar Column (Left) -->
      <v-col cols="12" md="3" xl="2">
        <div class="sticky-sidebar">
          <EventSidebar :event="event" active-item="settings" @add-showtime="goToEventDetails"
            @open-seat-map="goToEventDetails" @delete-event="goToEventDetails" />
        </div>
      </v-col>

      <!-- Main Content Column (Right) -->
      <v-col cols="12" md="9" xl="10">
        <v-form @submit.prevent="saveSettings">
          <!-- Page Header Card -->
          <v-card theme="dark" class="rounded-xl pa-6 border bg-surface mb-6" elevation="3">
            <div class="d-flex align-start gap-4">
              <v-avatar color="blue-grey-darken-3" size="52" rounded="lg">
                <v-icon icon="mdi-cog-outline" color="white" size="30"></v-icon>
              </v-avatar>
              <div>
                <div class="d-flex align-center gap-2 mb-1">
                  <span class="text-caption text-grey font-weight-mono text-uppercase">EVENT SETTINGS & TAX
                    CONFIGURATION</span>
                  <span class="text-caption font-weight-mono text-amber-darken-1 font-weight-bold">#{{ event.id
                    }}</span>
                </div>
                <h1 class="text-h4 font-weight-bold text-white mb-1">
                  {{ form.title || event.title }}
                </h1>
                <p class="text-caption text-grey mb-0">
                  Configure event policy limits and business tax identity for invoicing & reporting
                </p>
              </div>
            </div>
          </v-card>

          <!-- Section 1: General Event Settings -->
          <v-card theme="dark" class="rounded-xl border pa-6 mb-6 bg-surface">
            <div class="d-flex align-center gap-2 mb-4">
              <v-icon icon="mdi-tune" color="amber-darken-1"></v-icon>
              <span class="text-h6 font-weight-bold text-white">General Event Configuration</span>
            </div>

            <v-text-field v-model="form.title" label="Event Title *" variant="outlined" density="comfortable" required
              :rules="[v => !!v || 'Event title is required']" class="mb-3"></v-text-field>

            <v-row dense>
              <v-col cols="12" sm="6">
                <v-text-field v-model.number="form.max_seats_per_booking" label="Max Seats Per Order / Checkout *"
                  type="number" min="1" max="50" variant="outlined" density="comfortable"
                  hint="Limit how many tickets one order transaction can contain" persistent-hint
                  class="mb-3"></v-text-field>
              </v-col>
              <v-col cols="12" sm="6">
                <v-text-field v-model.number="form.max_seats_per_user" label="Max Total Seats Per User Account *"
                  type="number" min="1" max="100" variant="outlined" density="comfortable"
                  hint="Global cap on active tickets owned by single customer across event" persistent-hint
                  class="mb-3"></v-text-field>
              </v-col>
            </v-row>
          </v-card>

          <!-- Section 2: Organizer Tax Identity -->
          <v-card theme="dark" class="rounded-xl border pa-6 mb-6 bg-surface">
            <div class="d-flex align-center gap-2 mb-2">
              <v-icon icon="mdi-domain" color="teal-accent-3"></v-icon>
              <span class="text-h6 font-weight-bold text-white">Organizer Tax & Business Invoicing Identity</span>
            </div>
            <p class="text-caption text-grey mb-4">
              This tax information will appear on official tax receipts given to customers and on financial settlement
              statements generated for tax filings.
            </p>

            <v-row dense>
              <v-col cols="12" sm="6">
                <v-text-field v-model="form.tax_business_name" label="Legal Business / Entity Name"
                  placeholder="e.g. DreamCinema Live Productions Ltd." variant="outlined" density="comfortable"
                  class="mb-3"></v-text-field>
              </v-col>
              <v-col cols="12" sm="6">
                <v-text-field v-model="form.tax_id" label="Tax ID / VAT / GST Registration Number"
                  placeholder="e.g. TAX-8842-TSB-2026 / US-EIN-991823" variant="outlined" density="comfortable"
                  class="mb-3"></v-text-field>
              </v-col>
            </v-row>

            <v-textarea v-model="form.tax_address" label="Registered Tax / Billing Address"
              placeholder="e.g. 742 Evergreen Terrace, Suite 400, Springfield, OR 97477" rows="2" variant="outlined"
              density="comfortable" class="mb-3"></v-textarea>

            <v-row dense>
              <v-col cols="12" sm="6">
                <v-text-field v-model="form.tax_email" label="Tax / Financial Inquiries Email"
                  placeholder="e.g. billing@dreamcinema.com" variant="outlined" density="comfortable"
                  class="mb-3"></v-text-field>
              </v-col>
              <v-col cols="12" sm="6">
                <v-text-field v-model="form.tax_phone" label="Tax / Business Phone" placeholder="e.g. +1 (555) 019-2834"
                  variant="outlined" density="comfortable" class="mb-3"></v-text-field>
              </v-col>
            </v-row>
          </v-card>

          <!-- Save Actions -->
          <div class="d-flex justify-end gap-3 pb-8">
            <v-btn variant="outlined" color="grey" size="large" :to="`/manage/events/${eventId}`"
              class="text-none font-weight-bold px-6">
              Cancel
            </v-btn>
            <v-btn color="amber-darken-1" variant="flat" size="large" type="submit"
              class="text-none font-weight-bold px-8" :loading="saving" prepend-icon="mdi-content-save">
              Save Settings & Tax Info
            </v-btn>
          </div>
        </v-form>
      </v-col>
    </v-row>

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
import EventSidebar from '../components/EventSidebar.vue';

const props = defineProps({
  id: { type: String, required: false },
});

const route = useRoute();
const router = useRouter();
const eventId = props.id || route.params.id;

const event = ref(null);
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

function goToEventDetails() {
  router.push(`/manage/events/${eventId}`);
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
    event.value = found;
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

<style scoped>
.sticky-sidebar {
  position: sticky;
  top: 80px;
  z-index: 10;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
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
