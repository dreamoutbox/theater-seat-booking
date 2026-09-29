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
      <EventSidebar :event="event" active-item="gatekeepers" />

      <!-- Main Content Column (Right) -->
      <main class="manage-main-content">
        <div class="d-flex flex-column gap-6">
          <!-- Event Header Card -->
          <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="3">
            <div class="d-flex flex-wrap justify-space-between align-start gap-4">
              <div class="d-flex align-start gap-3">
                <v-avatar color="teal-darken-3" size="48" rounded="lg">
                  <v-icon icon="mdi-shield-account" color="white" size="28"></v-icon>
                </v-avatar>
                <div>
                  <div class="d-flex align-center gap-2 mb-1">
                    <span class="text-caption text-grey font-weight-mono text-uppercase">EVENT GATEKEEPERS</span>
                    <span class="text-caption font-weight-mono text-teal-accent-3 font-weight-bold">#{{ event?.id
                    }}</span>
                  </div>
                  <h1 class="text-h4 font-weight-bold text-white mb-1">
                    {{ event?.title }}
                  </h1>
                  <div class="d-flex align-center gap-2 flex-wrap mt-2">
                    <v-chip size="small" color="teal-lighten-2" variant="tonal" class="font-weight-bold">
                      <v-icon start icon="mdi-account-multiple-check" size="14"></v-icon>
                      {{ gatekeepers.length }}
                      {{ gatekeepers.length === 1 ? 'Gatekeeper Assigned' : 'Gatekeepers Assigned' }}
                    </v-chip>
                  </div>
                </div>
              </div>

              <div class="d-flex align-center gap-2 flex-wrap">
                <v-btn color="teal-accent-3" variant="flat" size="small" prepend-icon="mdi-account-plus"
                  class="text-none font-weight-bold text-black" @click="openAddGatekeeperDialog">
                  Add Gatekeeper
                </v-btn>
              </div>
            </div>
          </v-card>

          <!-- Gatekeepers Management Card -->
          <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="3">
            <div class="d-flex justify-space-between align-center mb-4 flex-wrap gap-2">
              <div>
                <h2 class="text-h6 font-weight-bold text-white mb-1 d-flex align-center gap-2">
                  <v-icon icon="mdi-shield-account-outline" color="teal-accent-3" size="24"></v-icon>
                  Assigned Scanner Staff
                </h2>
                <span class="text-caption text-grey">Staff authorized to scan QR tickets and verify admission at the
                  entrance</span>
              </div>
            </div>

            <!-- Empty State -->
            <div v-if="gatekeepers.length === 0"
              class="text-center pa-8 rounded-lg bg-surface-variant border border-dashed">
              <v-icon icon="mdi-shield-account" size="48" color="grey" class="mb-2"></v-icon>
              <div class="text-body-1 text-grey font-weight-medium">No gatekeepers assigned to this event yet</div>
              <div class="text-caption text-grey-darken-1 mt-1">Add scanner staff accounts to allow QR scanning at
                entrance.</div>
              <v-btn color="teal-accent-3" variant="flat" prepend-icon="mdi-account-plus"
                class="mt-4 text-none font-weight-bold text-black" @click="openAddGatekeeperDialog">
                Add Gatekeeper
              </v-btn>
            </div>

            <!-- Gatekeepers Table -->
            <v-table v-else class="gatekeepers-table rounded-lg border">
              <thead>
                <tr>
                  <th class="text-left font-weight-bold">Gatekeeper Email</th>
                  <th class="text-left font-weight-bold">Assigned Date & Time</th>
                  <th class="text-right font-weight-bold">Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="g in gatekeepers" :key="g.user_id">
                  <td class="font-weight-bold text-white py-3">
                    <v-icon icon="mdi-account-circle" color="teal-accent-3" size="20" class="mr-2"></v-icon>
                    {{ g.email }}
                  </td>
                  <td class="text-caption text-grey">
                    {{ formatDate(g.assigned_at) }}
                  </td>
                  <td class="text-right">
                    <v-btn icon="mdi-account-remove-outline" size="small" variant="text" color="red-lighten-2"
                      title="Remove Gatekeeper from Event" @click="openRemoveGatekeeperDialog(g)"></v-btn>
                  </td>
                </tr>
              </tbody>
            </v-table>
          </v-card>
        </div>
      </main>
    </div>

    <!-- Add Gatekeeper Dialog -->
    <v-dialog v-model="gatekeeperDialog.show" max-width="480">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold d-flex align-center gap-2">
          <v-icon icon="mdi-shield-account" color="teal-accent-3"></v-icon>
          Add Gatekeeper Scanner
        </v-card-title>
        <v-card-text>
          <div class="text-caption text-grey mb-3">
            Create or assign a gatekeeper account for admission QR scanning at this event.
          </div>
          <v-text-field v-model="gatekeeperDialog.email" label="Gatekeeper Email" type="email" variant="outlined"
            density="comfortable" placeholder="e.g. staff.scanner@theater.local" class="mb-3" autofocus></v-text-field>
          <v-text-field v-model="gatekeeperDialog.password" label="Temporary Password" type="password"
            variant="outlined" density="comfortable" placeholder="Min 6 characters"
            hint="Scanner staff can use this to login and access /scan" persistent-hint></v-text-field>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="gatekeeperDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn color="teal-accent-3" variant="flat" class="font-weight-bold text-none text-black"
            :loading="gatekeeperDialog.saving" @click="saveGatekeeper">
            Assign Gatekeeper
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialog.show" max-width="420">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold text-red-lighten-2">Confirm Removal</v-card-title>
        <v-card-text>
          Are you sure you want to remove gatekeeper <strong>{{ deleteDialog.itemName }}</strong> from this event?
          <div class="text-caption text-grey mt-2">They will no longer be authorized to scan tickets for this event.
          </div>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="deleteDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn color="red-darken-1" variant="flat" class="font-weight-bold text-none" :loading="deleteDialog.deleting"
            @click="confirmDelete">
            Remove
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
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import api from '../api/client';
import EventSidebar from '../components/EventSidebar.vue';

const props = defineProps({
  id: { type: String, required: true },
});

const route = useRoute();
const router = useRouter();
const event = ref(null);
const gatekeepers = ref([]);
const loading = ref(true);
const error = ref(null);

function goToEventDetails() {
  router.push(`/manage/events/${props.id || route.params.id}`);
}

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

const gatekeeperDialog = ref({
  show: false,
  email: '',
  password: '',
  saving: false,
});

const deleteDialog = ref({
  show: false,
  userId: null,
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
    const eventsRes = await api.get('/events');
    const found = eventsRes.data.find((e) => e.id === eventId);
    if (!found) {
      error.value = 'Event not found';
      return;
    }
    event.value = found;
    await fetchGatekeepers(eventId);
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load event details';
  } finally {
    loading.value = false;
  }
}

async function fetchGatekeepers(eventId) {
  try {
    const res = await api.get(`/events/${eventId}/gatekeepers`);
    gatekeepers.value = res.data || [];
  } catch (err) {
    console.error('Failed to load event gatekeepers', err);
  }
}

function openAddGatekeeperDialog() {
  gatekeeperDialog.value = {
    show: true,
    email: '',
    password: '',
    saving: false,
  };
}

async function saveGatekeeper() {
  if (!gatekeeperDialog.value.email || !gatekeeperDialog.value.email.includes('@')) {
    showToast('Please enter a valid email', 'warning');
    return;
  }
  if (!gatekeeperDialog.value.password || gatekeeperDialog.value.password.length < 6) {
    showToast('Password must be at least 6 characters', 'warning');
    return;
  }

  gatekeeperDialog.value.saving = true;
  const eventId = props.id || route.params.id;
  try {
    await api.post(`/events/${eventId}/gatekeepers`, {
      email: gatekeeperDialog.value.email.trim(),
      password: gatekeeperDialog.value.password,
    });
    showToast('Gatekeeper account assigned successfully', 'success');
    gatekeeperDialog.value.show = false;
    await fetchGatekeepers(eventId);
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to assign gatekeeper', 'error');
  } finally {
    gatekeeperDialog.value.saving = false;
  }
}

function openRemoveGatekeeperDialog(g) {
  deleteDialog.value = {
    show: true,
    userId: g.user_id,
    itemName: g.email,
    deleting: false,
  };
}

async function confirmDelete() {
  deleteDialog.value.deleting = true;
  const eventId = props.id || route.params.id;
  try {
    await api.delete(`/events/${eventId}/gatekeepers/${deleteDialog.value.userId}`);
    showToast('Gatekeeper removed successfully', 'success');
    deleteDialog.value.show = false;
    await fetchGatekeepers(eventId);
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to remove gatekeeper', 'error');
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
  padding: 0px 10px 0px 10px;
}

.max-w-5xl {
  max-width: 1050px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.gatekeepers-table {
  background: #131722 !important;
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
