<template>
  <v-container class="py-8 max-w-6xl">
    <!-- Header -->
    <div class="d-flex flex-wrap justify-space-between align-center gap-4 mb-6">
      <div>
        <div class="d-flex align-center gap-2 mb-1">
          <v-icon icon="mdi-movie-edit" color="amber-darken-1" size="28"></v-icon>
          <h1 class="text-h4 font-weight-bold text-white mb-0">Event & Showtime Management</h1>
        </div>
        <span class="text-caption text-grey">
          Create, edit, and schedule events and showtime sessions across theaters
        </span>
      </div>

      <div class="d-flex align-center gap-2">
        <v-btn
          prepend-icon="mdi-refresh"
          variant="outlined"
          size="small"
          :loading="loading"
          @click="loadData"
        >
          Refresh
        </v-btn>
        <v-btn
          color="amber-darken-1"
          variant="flat"
          prepend-icon="mdi-plus"
          class="text-none font-weight-bold"
          @click="openCreateEventDialog"
        >
          New Event
        </v-btn>
      </div>
    </div>

    <!-- Search Input -->
    <v-card theme="dark" class="pa-3 mb-6 rounded-xl border bg-surface" elevation="2">
      <v-text-field
        v-model="searchQuery"
        placeholder="Filter by event title or theater name..."
        prepend-inner-icon="mdi-magnify"
        variant="outlined"
        density="compact"
        hide-details
        clearable
      ></v-text-field>
    </v-card>

    <!-- Loading Skeleton -->
    <div v-if="loading" class="d-flex flex-column gap-4">
      <v-skeleton-loader v-for="n in 3" :key="n" type="card, table" theme="dark" class="rounded-xl"></v-skeleton-loader>
    </div>

    <!-- Empty State -->
    <v-card v-else-if="filteredEvents.length === 0" variant="tonal" class="text-center pa-8 rounded-xl border">
      <v-icon icon="mdi-filmstrip-off" size="48" color="grey" class="mb-2"></v-icon>
      <div class="text-body-1 text-grey font-weight-medium">No events found</div>
      <div class="text-caption text-grey-darken-1 mt-1">Get started by creating your first movie or concert event.</div>
      <v-btn color="amber-darken-1" variant="flat" prepend-icon="mdi-plus" class="mt-4 text-none font-weight-bold" @click="openCreateEventDialog">
        Create Event
      </v-btn>
    </v-card>

    <!-- Events List -->
    <div v-else class="d-flex flex-column gap-6">
      <v-card
        v-for="event in filteredEvents"
        :key="event.id"
        theme="dark"
        class="event-manage-card rounded-xl pa-6 border bg-surface"
        elevation="3"
      >
        <!-- Event Card Header -->
        <div class="d-flex flex-wrap justify-space-between align-center gap-4 mb-4">
          <div class="d-flex align-center gap-3">
            <v-avatar color="amber-darken-3" size="40" rounded="lg">
              <v-icon icon="mdi-filmstrip" color="white" size="24"></v-icon>
            </v-avatar>
            <div>
              <h2 class="text-h5 font-weight-bold text-white mb-0">{{ event.title }}</h2>
              <span class="text-caption text-grey font-weight-mono">ID: #{{ event.id.substring(0, 8) }}</span>
            </div>
          </div>

          <div class="d-flex align-center gap-2">
            <v-btn
              variant="outlined"
              size="small"
              prepend-icon="mdi-pencil"
              class="text-none"
              @click="openEditEventDialog(event)"
            >
              Edit Title
            </v-btn>
            <v-btn
              variant="outlined"
              color="red-lighten-2"
              size="small"
              prepend-icon="mdi-delete-outline"
              class="text-none"
              @click="openDeleteEventDialog(event)"
            >
              Delete Event
            </v-btn>
            <v-btn
              color="amber-darken-1"
              variant="flat"
              size="small"
              prepend-icon="mdi-calendar-plus"
              class="text-none font-weight-bold"
              @click="openAddShowtimeDialog(event)"
            >
              Add Showtime
            </v-btn>
          </div>
        </div>

        <v-divider class="mb-4"></v-divider>

        <!-- Showtimes Sub-Table -->
        <div>
          <div class="d-flex justify-space-between align-center mb-3">
            <span class="text-caption text-grey font-weight-bold text-uppercase">
              SCHEDULED SHOWTIMES ({{ getEventShowtimes(event.id).length }})
            </span>
          </div>

          <div v-if="getEventShowtimes(event.id).length === 0" class="text-center pa-4 rounded-lg bg-surface-variant border border-dashed">
            <span class="text-caption text-grey">No showtimes scheduled for this event yet. Click "Add Showtime" to create one.</span>
          </div>

          <v-table v-else class="showtimes-table rounded-lg border">
            <thead>
              <tr>
                <th class="text-left font-weight-bold">Showtime</th>
                <th class="text-left font-weight-bold">Theater Hall</th>
                <th class="text-left font-weight-bold">Session ID</th>
                <th class="text-right font-weight-bold">Actions</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="show in getEventShowtimes(event.id)" :key="show.id">
                <td class="py-3 font-weight-bold text-white">
                  <v-icon icon="mdi-calendar-clock" size="16" color="amber-darken-1" class="mr-2"></v-icon>
                  {{ formatDate(show.starts_at) }}
                </td>
                <td>
                  <span class="text-body-2 text-grey-lighten-1">{{ show.theater_name }}</span>
                </td>
                <td>
                  <span class="font-weight-mono text-caption text-grey">#{{ show.id.substring(0, 8) }}</span>
                </td>
                <td class="text-right">
                  <div class="d-flex align-center justify-end gap-1">
                    <v-btn
                      variant="text"
                      size="small"
                      color="amber-darken-1"
                      prepend-icon="mdi-seat"
                      class="text-none font-weight-bold"
                      :to="`/booking/${show.id}`"
                    >
                      Seat Map
                    </v-btn>
                    <v-btn
                      icon="mdi-pencil"
                      size="small"
                      variant="text"
                      color="grey"
                      title="Edit Showtime"
                      @click="openEditShowtimeDialog(show)"
                    ></v-btn>
                    <v-btn
                      icon="mdi-delete"
                      size="small"
                      variant="text"
                      color="red-lighten-2"
                      title="Delete Showtime"
                      @click="openDeleteShowtimeDialog(show)"
                    ></v-btn>
                  </div>
                </td>
              </tr>
            </tbody>
          </v-table>
        </div>
      </v-card>
    </div>

    <!-- Event Dialog (Create / Edit) -->
    <v-dialog v-model="eventDialog.show" max-width="480">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold">
          {{ eventDialog.isEdit ? 'Edit Event Title' : 'Create New Event' }}
        </v-card-title>
        <v-card-text>
          <v-text-field
            v-model="eventDialog.title"
            label="Event Title"
            placeholder="e.g. Gladiator II: IMAX Premiere"
            variant="outlined"
            density="comfortable"
            autofocus
            class="mb-2"
            @keyup.enter="saveEvent"
          ></v-text-field>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="eventDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="amber-darken-1"
            variant="flat"
            class="font-weight-bold text-none"
            :loading="eventDialog.saving"
            @click="saveEvent"
          >
            {{ eventDialog.isEdit ? 'Save Changes' : 'Create Event' }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Showtime Dialog (Create / Edit) -->
    <v-dialog v-model="showtimeDialog.show" max-width="500">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold">
          {{ showtimeDialog.isEdit ? 'Edit Showtime Session' : 'Add Showtime Session' }}
        </v-card-title>
        <v-card-text>
          <div class="mb-4">
            <span class="text-caption text-grey text-uppercase font-weight-bold">EVENT</span>
            <div class="text-body-1 font-weight-bold text-white">{{ showtimeDialog.eventTitle }}</div>
          </div>

          <v-select
            v-model="showtimeDialog.theaterId"
            label="Theater Location"
            :items="theaters"
            item-title="name"
            item-value="id"
            variant="outlined"
            density="comfortable"
            class="mb-4"
          ></v-select>

          <div>
            <label class="text-caption text-grey mb-1 d-block font-weight-bold">START DATE & TIME (Local Time)</label>
            <input
              type="datetime-local"
              v-model="showtimeDialog.datetimeLocal"
              class="datetime-input mb-2"
            />
          </div>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="showtimeDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="amber-darken-1"
            variant="flat"
            class="font-weight-bold text-none"
            :loading="showtimeDialog.saving"
            @click="saveShowtime"
          >
            {{ showtimeDialog.isEdit ? 'Save Showtime' : 'Schedule Showtime' }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialog.show" max-width="420">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold text-red-lighten-2">
          Confirm Deletion
        </v-card-title>
        <v-card-text>
          Are you sure you want to delete <strong>{{ deleteDialog.itemName }}</strong>?
          <div class="text-caption text-grey mt-2">
            This action is irreversible. All related showtimes or references will be removed.
          </div>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="deleteDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="red-darken-1"
            variant="flat"
            class="font-weight-bold text-none"
            :loading="deleteDialog.deleting"
            @click="confirmDelete"
          >
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Snackbar -->
    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>
  </v-container>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import api from '../api/client';

const events = ref([]);
const showtimes = ref([]);
const theaters = ref([]);
const loading = ref(false);
const searchQuery = ref('');

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

// Event Dialog State
const eventDialog = ref({
  show: false,
  isEdit: false,
  eventId: null,
  title: '',
  saving: false,
});

// Showtime Dialog State
const showtimeDialog = ref({
  show: false,
  isEdit: false,
  showtimeId: null,
  eventId: null,
  eventTitle: '',
  theaterId: null,
  datetimeLocal: '',
  saving: false,
});

// Delete Dialog State
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
  });
}

function getEventShowtimes(eventId) {
  return showtimes.value
    .filter((s) => s.event_id === eventId)
    .sort((a, b) => new Date(a.starts_at) - new Date(b.starts_at));
}

const filteredEvents = computed(() => {
  if (!searchQuery.value || !searchQuery.value.trim()) {
    return events.value;
  }
  const q = searchQuery.value.trim().toLowerCase();
  return events.value.filter((e) => {
    const titleMatch = e.title.toLowerCase().includes(q);
    const showtimesMatch = getEventShowtimes(e.id).some((s) =>
      s.theater_name.toLowerCase().includes(q) || formatDate(s.starts_at).toLowerCase().includes(q)
    );
    return titleMatch || showtimesMatch;
  });
});

async function loadData() {
  loading.value = true;
  try {
    const [eventsRes, showtimesRes, theatersRes] = await Promise.all([
      api.get('/events'),
      api.get('/showtimes'),
      api.get('/theaters'),
    ]);
    events.value = eventsRes.data;
    showtimes.value = showtimesRes.data;
    theaters.value = theatersRes.data;
  } catch (err) {
    showToast('Failed to load events data', 'error');
  } finally {
    loading.value = false;
  }
}

// Event Actions
function openCreateEventDialog() {
  eventDialog.value = {
    show: true,
    isEdit: false,
    eventId: null,
    title: '',
    saving: false,
  };
}

function openEditEventDialog(event) {
  eventDialog.value = {
    show: true,
    isEdit: true,
    eventId: event.id,
    title: event.title,
    saving: false,
  };
}

async function saveEvent() {
  if (!eventDialog.value.title.trim()) {
    showToast('Please enter an event title', 'warning');
    return;
  }
  eventDialog.value.saving = true;
  try {
    if (eventDialog.value.isEdit) {
      await api.put(`/events/${eventDialog.value.eventId}`, {
        title: eventDialog.value.title.trim(),
      });
      showToast('Event updated successfully', 'success');
    } else {
      await api.post('/events', {
        title: eventDialog.value.title.trim(),
      });
      showToast('Event created successfully', 'success');
    }
    eventDialog.value.show = false;
    await loadData();
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Failed to save event';
    showToast(errMsg, 'error');
  } finally {
    eventDialog.value.saving = false;
  }
}

function openDeleteEventDialog(event) {
  deleteDialog.value = {
    show: true,
    type: 'event',
    itemId: event.id,
    itemName: event.title,
    deleting: false,
  };
}

// Showtime Actions
function openAddShowtimeDialog(event) {
  const defaultTheaterId = theaters.value[0]?.id || null;
  // Default to tomorrow + 2 hours formatted for datetime-local
  const defaultDate = new Date();
  defaultDate.setDate(defaultDate.getDate() + 1);
  defaultDate.setHours(19, 0, 0, 0);
  const localIso = new Date(defaultDate.getTime() - defaultDate.getTimezoneOffset() * 60000)
    .toISOString()
    .slice(0, 16);

  showtimeDialog.value = {
    show: true,
    isEdit: false,
    showtimeId: null,
    eventId: event.id,
    eventTitle: event.title,
    theaterId: defaultTheaterId,
    datetimeLocal: localIso,
    saving: false,
  };
}

function openEditShowtimeDialog(show) {
  const d = new Date(show.starts_at);
  const localIso = new Date(d.getTime() - d.getTimezoneOffset() * 60000)
    .toISOString()
    .slice(0, 16);

  showtimeDialog.value = {
    show: true,
    isEdit: true,
    showtimeId: show.id,
    eventId: show.event_id,
    eventTitle: show.event_title,
    theaterId: show.theater_id,
    datetimeLocal: localIso,
    saving: false,
  };
}

async function saveShowtime() {
  if (!showtimeDialog.value.datetimeLocal) {
    showToast('Please select a date and time', 'warning');
    return;
  }
  if (!showtimeDialog.value.theaterId) {
    showToast('Please select a theater', 'warning');
    return;
  }

  showtimeDialog.value.saving = true;
  const startsAtUtc = new Date(showtimeDialog.value.datetimeLocal).toISOString();

  try {
    if (showtimeDialog.value.isEdit) {
      await api.put(`/showtimes/${showtimeDialog.value.showtimeId}`, {
        theater_id: showtimeDialog.value.theaterId,
        starts_at: startsAtUtc,
      });
      showToast('Showtime updated successfully', 'success');
    } else {
      await api.post('/showtimes', {
        event_id: showtimeDialog.value.eventId,
        theater_id: showtimeDialog.value.theaterId,
        starts_at: startsAtUtc,
      });
      showToast('Showtime scheduled successfully', 'success');
    }
    showtimeDialog.value.show = false;
    await loadData();
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Failed to save showtime';
    showToast(errMsg, 'error');
  } finally {
    showtimeDialog.value.saving = false;
  }
}

function openDeleteShowtimeDialog(show) {
  deleteDialog.value = {
    show: true,
    type: 'showtime',
    itemId: show.id,
    itemName: `Showtime session on ${formatDate(show.starts_at)}`,
    deleting: false,
  };
}

async function confirmDelete() {
  deleteDialog.value.deleting = true;
  try {
    if (deleteDialog.value.type === 'event') {
      await api.delete(`/events/${deleteDialog.value.itemId}`);
      showToast('Event deleted successfully', 'success');
    } else if (deleteDialog.value.type === 'showtime') {
      await api.delete(`/showtimes/${deleteDialog.value.itemId}`);
      showToast('Showtime deleted successfully', 'success');
    }
    deleteDialog.value.show = false;
    await loadData();
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Failed to delete item';
    showToast(errMsg, 'error');
  } finally {
    deleteDialog.value.deleting = false;
  }
}

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.max-w-6xl {
  max-width: 1200px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.event-manage-card {
  transition: border-color 0.2s ease;
}

.event-manage-card:hover {
  border-color: #f59e0b;
}

.showtimes-table {
  background: #131722 !important;
  border-color: #2a3142 !important;
}

.datetime-input {
  width: 100%;
  padding: 10px 14px;
  background-color: #131722;
  border: 1px solid #3b4256;
  border-radius: 8px;
  color: #fff;
  font-family: inherit;
  font-size: 14px;
  outline: none;
}

.datetime-input:focus {
  border-color: #f59e0b;
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
