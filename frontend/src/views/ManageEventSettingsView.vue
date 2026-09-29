<template>
  <div class="manage-layout">
    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <v-progress-circular indeterminate color="amber-darken-1" size="64"></v-progress-circular>
      <div class="text-body-1 text-grey mt-4">Loading event settings...</div>
    </div>

    <!-- Error State -->
    <v-alert v-else-if="error" type="error" variant="tonal" class="rounded-xl ma-6">
      {{ error }}
    </v-alert>

    <div v-else-if="event" class="manage-layout-body">
      <!-- Sidebar (Flush Left) -->
      <EventSidebar :event="event" active-item="settings" />

      <!-- Main Content Column (Right) -->
      <main class="manage-main-content">
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
                  placeholder="e.g. TAX-8842-TSB-2026" variant="outlined" density="comfortable"
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

        <!-- Section 3: Danger Zone -->
        <v-card theme="dark" class="rounded-xl border border-red-darken-4 pa-6 mb-8 bg-surface">
          <div class="d-flex align-center justify-space-between flex-wrap gap-4">
            <div>
              <div class="d-flex align-center gap-2 mb-1">
                <v-icon icon="mdi-alert-outline" color="red-lighten-2" size="22"></v-icon>
                <span class="text-h6 font-weight-bold text-red-lighten-2">Danger Zone</span>
              </div>
              <p class="text-caption text-grey mb-0" style="max-width: 600px;">
                Permanently delete this event and all associated showtimes, tickets, and configurations. This action is
                irreversible.
              </p>
            </div>

            <v-btn color="red-darken-1" variant="flat" size="default" prepend-icon="mdi-delete-outline"
              class="text-none font-weight-bold delete-event-btn" @click="openDeleteEventDialog">
              Delete Event
            </v-btn>
          </div>
        </v-card>
      </main>
    </div>

    <!-- Delete Event Confirmation Dialog -->
    <v-dialog v-model="deleteDialog.show" max-width="440" theme="dark">
      <v-card theme="dark" class="rounded-xl pa-6 border bg-surface">
        <div class="d-flex align-center gap-3 mb-4">
          <v-avatar color="red-darken-4" size="44">
            <v-icon icon="mdi-alert" color="red-lighten-2" size="24"></v-icon>
          </v-avatar>
          <div>
            <h3 class="text-h6 font-weight-bold text-white">Delete Event</h3>
            <span class="text-caption text-grey">This action cannot be undone.</span>
          </div>
        </div>

        <p class="text-body-2 text-grey-lighten-1 mb-6">
          Are you sure you want to delete <strong>{{ event?.title }}</strong>? All showtimes, seat reservations, and
          associated data will be permanently removed.
        </p>

        <div class="d-flex justify-end gap-3">
          <v-btn variant="text" class="text-none font-weight-bold" :disabled="deleteDialog.deleting"
            @click="deleteDialog.show = false">
            Cancel
          </v-btn>
          <v-btn color="red-darken-1" variant="flat" class="text-none font-weight-bold px-6"
            :loading="deleteDialog.deleting" @click="confirmDeleteEvent">
            Yes, Delete Event
          </v-btn>
        </div>
      </v-card>
    </v-dialog>

    <!-- Global Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3500" location="bottom right">
      {{ snackbar.text }}
      <template #actions>
        <v-btn variant="text" @click="snackbar.show = false">Close</v-btn>
      </template>
    </v-snackbar>
  </div>
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

const deleteDialog = ref({
  show: false,
  deleting: false,
});

function openDeleteEventDialog() {
  deleteDialog.value = {
    show: true,
    deleting: false,
  };
}

async function confirmDeleteEvent() {
  deleteDialog.value.deleting = true;
  try {
    await api.delete(`/events/${eventId}`);
    deleteDialog.value.show = false;
    showToast('Event deleted successfully', 'success');
    setTimeout(() => {
      router.push('/manage/events');
    }, 500);
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to delete event', 'error');
  } finally {
    deleteDialog.value.deleting = false;
  }
}

onMounted(() => {
  loadEvent();
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

.border-red-darken-4 {
  border-color: rgba(239, 68, 68, 0.35) !important;
}

.delete-event-btn {
  transition: all 0.2s ease;
}

.delete-event-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
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
