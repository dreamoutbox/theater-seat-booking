<template>
  <v-container class="py-8 max-w-5xl">
    <!-- Back Navigation -->
    <div class="mb-4">
      <v-btn
        variant="text"
        prepend-icon="mdi-arrow-left"
        to="/manage/events"
        class="text-none font-weight-bold"
      >
        Back to Events
      </v-btn>
    </div>

    <!-- Loading Skeleton -->
    <v-skeleton-loader
      v-if="loading"
      type="article, table, actions"
      theme="dark"
      class="rounded-xl"
    ></v-skeleton-loader>

    <!-- Error State -->
    <v-alert
      v-else-if="error"
      type="error"
      variant="tonal"
      class="rounded-xl mb-6"
    >
      {{ error }}
    </v-alert>

    <div v-else-if="event" class="d-flex flex-column gap-6">
      <!-- Event Header Card -->
      <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="3">
        <div class="d-flex flex-wrap justify-space-between align-start gap-4 mb-4">
          <div class="d-flex align-start gap-3">
            <v-avatar color="amber-darken-3" size="48" rounded="lg">
              <v-icon icon="mdi-filmstrip" color="white" size="28"></v-icon>
            </v-avatar>
            <div>
              <div class="d-flex align-center gap-2 mb-1">
                <span class="text-caption text-grey font-weight-mono text-uppercase">EVENT REFERENCE</span>
                <span class="text-caption font-weight-mono text-amber-darken-1 font-weight-bold">#{{ event.id }}</span>
              </div>
              <h1 class="text-h4 font-weight-bold text-white mb-1">
                {{ event.title }}
              </h1>
              <span class="text-caption text-grey">
                {{ eventShowtimes.length }} {{ eventShowtimes.length === 1 ? 'Showtime session scheduled' : 'Showtime sessions scheduled' }}
              </span>
            </div>
          </div>

          <!-- Header Action Buttons -->
          <div class="d-flex align-center gap-2 flex-wrap">
            <v-btn
              variant="outlined"
              size="small"
              prepend-icon="mdi-pencil"
              class="text-none font-weight-bold"
              @click="openEditEventDialog"
            >
              Edit Title
            </v-btn>
            <v-btn
              variant="outlined"
              color="red-lighten-2"
              size="small"
              prepend-icon="mdi-delete-outline"
              class="text-none font-weight-bold"
              @click="openDeleteEventDialog"
            >
              Delete Event
            </v-btn>
            <v-btn
              color="indigo-darken-1"
              variant="flat"
              size="small"
              prepend-icon="mdi-seat-passenger"
              class="text-none font-weight-bold"
              @click="openTheaterSeatMapSelector"
            >
              Seat Map Editor
            </v-btn>
            <v-btn
              color="amber-darken-1"
              variant="flat"
              size="small"
              prepend-icon="mdi-calendar-plus"
              class="text-none font-weight-bold"
              @click="openAddShowtimeDialog"
            >
              Add Showtime
            </v-btn>
          </div>
        </div>
      </v-card>

      <!-- Scheduled Showtimes Card -->
      <v-card theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="3">
        <div class="d-flex justify-space-between align-center mb-4">
          <div>
            <h2 class="text-h6 font-weight-bold text-white mb-1">Scheduled Showtimes</h2>
            <span class="text-caption text-grey">Manage sessions, customer seat maps, and hall layouts</span>
          </div>
          <v-btn
            prepend-icon="mdi-plus"
            color="amber-darken-1"
            variant="tonal"
            size="small"
            class="text-none font-weight-bold"
            @click="openAddShowtimeDialog"
          >
            New Session
          </v-btn>
        </div>

        <!-- Empty State -->
        <div v-if="eventShowtimes.length === 0" class="text-center pa-8 rounded-lg bg-surface-variant border border-dashed">
          <v-icon icon="mdi-calendar-clock" size="48" color="grey" class="mb-2"></v-icon>
          <div class="text-body-1 text-grey font-weight-medium">No showtimes scheduled for this event</div>
          <div class="text-caption text-grey-darken-1 mt-1">Add your first showtime session to open ticket booking for users.</div>
          <v-btn color="amber-darken-1" variant="flat" prepend-icon="mdi-plus" class="mt-4 text-none font-weight-bold" @click="openAddShowtimeDialog">
            Add Showtime
          </v-btn>
        </div>

        <!-- Showtimes Table -->
        <v-table v-else class="showtimes-table rounded-lg border">
          <thead>
            <tr>
              <th class="text-left font-weight-bold">Date & Time</th>
              <th class="text-left font-weight-bold">Theater Location</th>
              <th class="text-left font-weight-bold">Session ID</th>
              <th class="text-right font-weight-bold">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="show in eventShowtimes" :key="show.id">
              <!-- Starts at -->
              <td class="py-3 font-weight-bold text-white">
                <v-icon icon="mdi-calendar-clock" size="16" color="amber-darken-1" class="mr-2"></v-icon>
                {{ formatDate(show.starts_at) }}
              </td>

              <!-- Theater name -->
              <td>
                <div class="font-weight-medium text-white">{{ show.theater_name }}</div>
                <div class="text-caption text-grey font-weight-mono">Grand Horizon Hall</div>
              </td>

              <!-- Session ID -->
              <td>
                <span class="font-weight-mono text-caption text-grey">#{{ show.id.substring(0, 8) }}</span>
              </td>

              <!-- Actions -->
              <td class="text-right">
                <div class="d-flex align-center justify-end gap-1">
                  <!-- Seat Map Editor for Theater -->
                  <v-btn
                    variant="tonal"
                    size="small"
                    color="indigo-lighten-2"
                    prepend-icon="mdi-seat-passenger"
                    class="text-none font-weight-bold"
                    title="Open Seat Map Editor for this Theater"
                    :to="`/manage/theaters/${show.theater_id}/seat-map`"
                  >
                    Edit Layout
                  </v-btn>

                  <!-- Customer Booking View -->
                  <v-btn
                    variant="text"
                    size="small"
                    color="amber-darken-1"
                    prepend-icon="mdi-seat"
                    class="text-none font-weight-bold"
                    title="View Customer Booking Seat Map"
                    :to="`/booking/${show.id}`"
                  >
                    Seat Map
                  </v-btn>

                  <!-- Edit Showtime -->
                  <v-btn
                    icon="mdi-pencil"
                    size="small"
                    variant="text"
                    color="grey"
                    title="Edit Showtime Session"
                    @click="openEditShowtimeDialog(show)"
                  ></v-btn>

                  <!-- Delete Showtime -->
                  <v-btn
                    icon="mdi-delete"
                    size="small"
                    variant="text"
                    color="red-lighten-2"
                    title="Delete Showtime Session"
                    @click="openDeleteShowtimeDialog(show)"
                  ></v-btn>
                </div>
              </td>
            </tr>
          </tbody>
        </v-table>
      </v-card>
    </div>

    <!-- Edit Event Dialog -->
    <v-dialog v-model="editEventDialog.show" max-width="480">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold">Edit Event Title</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="editEventDialog.title"
            label="Event Title"
            variant="outlined"
            density="comfortable"
            autofocus
            class="mb-2"
            @keyup.enter="saveEventTitle"
          ></v-text-field>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="editEventDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="amber-darken-1"
            variant="flat"
            class="font-weight-bold text-none"
            :loading="editEventDialog.saving"
            @click="saveEventTitle"
          >
            Save Changes
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Showtime Dialog (Add / Edit) -->
    <v-dialog v-model="showtimeDialog.show" max-width="500">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold">
          {{ showtimeDialog.isEdit ? 'Edit Showtime Session' : 'Add Showtime Session' }}
        </v-card-title>
        <v-card-text>
          <div class="mb-4">
            <span class="text-caption text-grey text-uppercase font-weight-bold">EVENT</span>
            <div class="text-body-1 font-weight-bold text-white">{{ event?.title }}</div>
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

    <!-- Theater Selector Dialog for Seat Map Editor -->
    <v-dialog v-model="theaterSelectorDialog.show" max-width="450">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold">Select Theater Hall</v-card-title>
        <v-card-text>
          <div class="text-caption text-grey mb-4">Choose a theater to open the interactive Seat Map Editor.</div>
          <v-select
            v-model="theaterSelectorDialog.theaterId"
            label="Theater"
            :items="theaters"
            item-title="name"
            item-value="id"
            variant="outlined"
            density="comfortable"
          ></v-select>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="theaterSelectorDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="indigo-darken-1"
            variant="flat"
            class="font-weight-bold text-none"
            :disabled="!theaterSelectorDialog.theaterId"
            @click="goToSeatMapEditor"
          >
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
import { useRoute, useRouter } from 'vue-router';
import api from '../api/client';

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

const editEventDialog = ref({
  show: false,
  title: '',
  saving: false,
});

const showtimeDialog = ref({
  show: false,
  isEdit: false,
  showtimeId: null,
  theaterId: null,
  datetimeLocal: '',
  saving: false,
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
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load event details';
  } finally {
    loading.value = false;
  }
}

function openEditEventDialog() {
  if (!event.value) return;
  editEventDialog.value = {
    show: true,
    title: event.value.title,
    saving: false,
  };
}

async function saveEventTitle() {
  if (!editEventDialog.value.title.trim()) {
    showToast('Please enter an event title', 'warning');
    return;
  }
  editEventDialog.value.saving = true;
  try {
    const res = await api.put(`/events/${event.value.id}`, {
      title: editEventDialog.value.title.trim(),
    });
    event.value = res.data;
    editEventDialog.value.show = false;
    showToast('Event title updated', 'success');
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to update event', 'error');
  } finally {
    editEventDialog.value.saving = false;
  }
}

function openDeleteEventDialog() {
  if (!event.value) return;
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
  if (defaultTheaterId) {
    router.push(`/manage/theaters/${defaultTheaterId}/seat-map`);
  } else {
    theaterSelectorDialog.value = {
      show: true,
      theaterId: defaultTheaterId,
    };
  }
}

function goToSeatMapEditor() {
  if (theaterSelectorDialog.value.theaterId) {
    theaterSelectorDialog.value.show = false;
    router.push(`/manage/theaters/${theaterSelectorDialog.value.theaterId}/seat-map`);
  }
}

function openAddShowtimeDialog() {
  const defaultTheaterId = eventShowtimes.value[0]?.theater_id || theaters.value[0]?.id || null;
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
      showToast('Showtime session updated', 'success');
    } else {
      await api.post('/showtimes', {
        event_id: event.value.id,
        theater_id: showtimeDialog.value.theaterId,
        starts_at: startsAtUtc,
      });
      showToast('Showtime session scheduled', 'success');
    }
    showtimeDialog.value.show = false;
    const showtimesRes = await api.get('/showtimes');
    allShowtimes.value = showtimesRes.data;
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to save showtime', 'error');
  } finally {
    showtimeDialog.value.saving = false;
  }
}

function openDeleteShowtimeDialog(show) {
  deleteDialog.value = {
    show: true,
    type: 'showtime',
    itemId: show.id,
    itemName: `Showtime on ${formatDate(show.starts_at)}`,
    deleting: false,
  };
}

async function confirmDelete() {
  deleteDialog.value.deleting = true;
  try {
    if (deleteDialog.value.type === 'event') {
      await api.delete(`/events/${deleteDialog.value.itemId}`);
      showToast('Event deleted successfully', 'success');
      deleteDialog.value.show = false;
      router.push('/manage/events');
    } else if (deleteDialog.value.type === 'showtime') {
      await api.delete(`/showtimes/${deleteDialog.value.itemId}`);
      showToast('Showtime deleted successfully', 'success');
      deleteDialog.value.show = false;
      const showtimesRes = await api.get('/showtimes');
      allShowtimes.value = showtimesRes.data;
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
