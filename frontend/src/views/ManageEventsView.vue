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
          Manage event titles, view scheduled showtime sessions, and access theater configurations
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

    <!-- Search Bar Card -->
    <v-card theme="dark" class="pa-3 mb-4 rounded-xl border bg-surface" elevation="2">
      <v-text-field
        v-model="searchQuery"
        placeholder="Search by event title, ID, or theater name..."
        prepend-inner-icon="mdi-magnify"
        variant="outlined"
        density="compact"
        hide-details
        clearable
      ></v-text-field>
    </v-card>

    <!-- Loading Skeleton -->
    <div v-if="loading" class="d-flex flex-column gap-4">
      <v-skeleton-loader type="table" theme="dark" class="rounded-xl"></v-skeleton-loader>
    </div>

    <!-- Empty State -->
    <v-card v-else-if="filteredEvents.length === 0" variant="tonal" class="text-center pa-8 rounded-xl border">
      <v-icon icon="mdi-filmstrip-off" size="48" color="grey" class="mb-2"></v-icon>
      <div class="text-body-1 text-grey font-weight-medium">
        {{ events.length === 0 ? 'No events created yet' : 'No events matching your search' }}
      </div>
      <div class="text-caption text-grey-darken-1 mt-1">
        {{ events.length === 0 ? 'Get started by creating your first movie or concert event.' : 'Try adjusting your search keywords.' }}
      </div>
      <v-btn v-if="events.length === 0" color="amber-darken-1" variant="flat" prepend-icon="mdi-plus" class="mt-4 text-none font-weight-bold" @click="openCreateEventDialog">
        Create Event
      </v-btn>
    </v-card>

    <!-- Events Table -->
    <div v-else class="table-container rounded-xl border overflow-hidden">
      <v-table theme="dark" class="events-table">
        <thead>
          <tr>
            <th class="text-left font-weight-bold">Event Title</th>
            <th class="text-left font-weight-bold">Event ID</th>
            <th class="text-left font-weight-bold">Showtimes</th>
            <th class="text-left font-weight-bold">Theaters</th>
            <th class="text-right font-weight-bold">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="event in paginatedEvents"
            :key="event.id"
            class="clickable-row"
            @click="goToEventDetail(event.id)"
          >
            <!-- Title -->
            <td class="py-3">
              <div class="d-flex align-center gap-3">
                <v-avatar color="amber-darken-4" size="32" rounded="md">
                  <v-icon icon="mdi-filmstrip" color="amber-lighten-2" size="18"></v-icon>
                </v-avatar>
                <div>
                  <div class="font-weight-bold text-body-1 text-white">{{ event.title }}</div>
                  <div class="text-caption text-grey">Click to view & manage showtimes</div>
                </div>
              </div>
            </td>

            <!-- ID -->
            <td>
              <span class="font-weight-mono text-caption text-amber-darken-1 font-weight-bold">
                #{{ event.id.substring(0, 8) }}
              </span>
            </td>

            <!-- Total Showtimes -->
            <td>
              <v-chip
                size="small"
                :color="getEventShowtimes(event.id).length > 0 ? 'indigo-darken-1' : 'grey'"
                variant="flat"
                class="font-weight-bold"
              >
                {{ getEventShowtimes(event.id).length }} {{ getEventShowtimes(event.id).length === 1 ? 'Session' : 'Sessions' }}
              </v-chip>
            </td>

            <!-- Theaters -->
            <td>
              <div class="text-caption text-grey-lighten-1">
                {{ getEventTheaters(event.id) }}
              </div>
            </td>

            <!-- Actions -->
            <td class="text-right">
              <div class="d-flex align-center justify-end gap-1" @click.stop>
                <v-btn
                  variant="outlined"
                  size="small"
                  color="amber-lighten-2"
                  prepend-icon="mdi-chart-box-outline"
                  class="text-none font-weight-bold"
                  title="View Event Analytics"
                  @click.stop="$router.push(`/manage/dashboard?event_id=${event.id}`)"
                >
                  Analytics
                </v-btn>
                <v-btn
                  variant="tonal"
                  size="small"
                  color="amber-darken-1"
                  prepend-icon="mdi-eye"
                  class="text-none font-weight-bold"
                  @click.stop="goToEventDetail(event.id)"
                >
                  Manage
                </v-btn>
              </div>
            </td>
          </tr>
        </tbody>
      </v-table>

      <!-- Pagination & Limit Footer -->
      <div class="d-flex flex-wrap justify-space-between align-center pa-4 bg-surface border-t gap-4">
        <div class="text-caption text-grey">
          Showing <strong>{{ paginationStart }}</strong>–<strong>{{ paginationEnd }}</strong> of <strong>{{ filteredEvents.length }}</strong> events
        </div>

        <div class="d-flex align-center gap-4">
          <!-- Rows per page -->
          <div class="d-flex align-center gap-2">
            <span class="text-caption text-grey">Rows per page:</span>
            <v-select
              v-model="itemsPerPage"
              :items="itemsPerPageOptions"
              variant="outlined"
              density="compact"
              hide-details
              style="width: 80px;"
            ></v-select>
          </div>

          <!-- Page Controls -->
          <div class="d-flex align-center gap-1">
            <v-btn
              icon="mdi-chevron-left"
              variant="outlined"
              size="small"
              :disabled="page <= 1"
              @click="page--"
            ></v-btn>
            <span class="text-caption text-grey mx-2 font-weight-mono">
              Page {{ page }} of {{ totalPages || 1 }}
            </span>
            <v-btn
              icon="mdi-chevron-right"
              variant="outlined"
              size="small"
              :disabled="page >= totalPages"
              @click="page++"
            ></v-btn>
          </div>
        </div>
      </div>
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
            placeholder="e.g. Dune: Part Two IMAX Experience"
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

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialog.show" max-width="420">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold text-red-lighten-2">
          Confirm Deletion
        </v-card-title>
        <v-card-text>
          Are you sure you want to delete <strong>{{ deleteDialog.itemName }}</strong>?
          <div class="text-caption text-grey mt-2">
            This will permanently remove the event and all associated scheduled showtimes.
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
            Delete Event
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
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import api from '../api/client';

const router = useRouter();

const events = ref([]);
const showtimes = ref([]);
const theaters = ref([]);
const loading = ref(false);
const searchQuery = ref('');

const page = ref(1);
const itemsPerPage = ref(10);
const itemsPerPageOptions = [5, 10, 25, 50];

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

const eventDialog = ref({
  show: false,
  isEdit: false,
  eventId: null,
  title: '',
  saving: false,
});

const deleteDialog = ref({
  show: false,
  eventId: null,
  itemName: '',
  deleting: false,
});

function getEventShowtimes(eventId) {
  return showtimes.value.filter((s) => s.event_id === eventId);
}

function getEventTheaters(eventId) {
  const matchingShowtimes = getEventShowtimes(eventId);
  if (matchingShowtimes.length === 0) return '—';
  const names = [...new Set(matchingShowtimes.map((s) => s.theater_name))];
  return names.join(', ');
}

function goToEventDetail(eventId) {
  router.push(`/manage/events/${eventId}`);
}

const filteredEvents = computed(() => {
  if (!searchQuery.value || !searchQuery.value.trim()) {
    return events.value;
  }
  const q = searchQuery.value.trim().toLowerCase();
  return events.value.filter((e) => {
    const titleMatch = e.title.toLowerCase().includes(q);
    const idMatch = e.id.toLowerCase().includes(q);
    const theaterMatch = getEventTheaters(e.id).toLowerCase().includes(q);
    return titleMatch || idMatch || theaterMatch;
  });
});

const totalPages = computed(() => {
  return Math.ceil(filteredEvents.value.length / itemsPerPage.value) || 1;
});

const paginatedEvents = computed(() => {
  const start = (page.value - 1) * itemsPerPage.value;
  return filteredEvents.value.slice(start, start + itemsPerPage.value);
});

const paginationStart = computed(() => {
  if (filteredEvents.value.length === 0) return 0;
  return (page.value - 1) * itemsPerPage.value + 1;
});

const paginationEnd = computed(() => {
  return Math.min(page.value * itemsPerPage.value, filteredEvents.value.length);
});

watch([searchQuery, itemsPerPage], () => {
  page.value = 1;
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
      const res = await api.post('/events', {
        title: eventDialog.value.title.trim(),
      });
      showToast('Event created successfully', 'success');
      eventDialog.value.show = false;
      await loadData();
      router.push(`/manage/events/${res.data.id}`);
      return;
    }
    eventDialog.value.show = false;
    await loadData();
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to save event', 'error');
  } finally {
    eventDialog.value.saving = false;
  }
}

function openDeleteEventDialog(event) {
  deleteDialog.value = {
    show: true,
    eventId: event.id,
    itemName: event.title,
    deleting: false,
  };
}

async function confirmDelete() {
  deleteDialog.value.deleting = true;
  try {
    await api.delete(`/events/${deleteDialog.value.eventId}`);
    showToast('Event deleted successfully', 'success');
    deleteDialog.value.show = false;
    await loadData();
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to delete event', 'error');
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

.table-container {
  background: #181c27;
  border-color: #2a3142;
}

.events-table {
  background: transparent !important;
}

.clickable-row {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.clickable-row:hover {
  background-color: rgba(245, 158, 11, 0.08) !important;
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
</style>
