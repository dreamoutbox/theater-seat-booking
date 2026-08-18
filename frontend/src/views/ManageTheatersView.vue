<template>
  <v-container class="py-8 max-w-6xl">
    <!-- Header -->
    <div class="d-flex flex-wrap justify-space-between align-center gap-4 mb-6">
      <div>
        <div class="d-flex align-center gap-2 mb-1">
          <v-icon icon="mdi-theater" color="amber-darken-1" size="28"></v-icon>
          <h1 class="text-h4 font-weight-bold text-white mb-0">Theater Management</h1>
        </div>
        <span class="text-caption text-grey">
          Manage cinema venues, halls, and launch the interactive Seat Map Editor for each theater
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
          @click="openCreateTheaterDialog"
        >
          New Theater
        </v-btn>
      </div>
    </div>

    <!-- Search Bar Card -->
    <v-card theme="dark" class="pa-3 mb-4 rounded-xl border bg-surface" elevation="2">
      <v-text-field
        v-model="searchQuery"
        placeholder="Search by theater name, address, or ID..."
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
    <v-card v-else-if="filteredTheaters.length === 0" variant="tonal" class="text-center pa-8 rounded-xl border">
      <v-icon icon="mdi-theater" size="48" color="grey" class="mb-2"></v-icon>
      <div class="text-body-1 text-grey font-weight-medium">
        {{ theaters.length === 0 ? 'No theaters created yet' : 'No theaters matching your search' }}
      </div>
      <div class="text-caption text-grey-darken-1 mt-1">
        {{ theaters.length === 0 ? 'Get started by creating your first theater hall or auditorium.' : 'Try adjusting your search keywords.' }}
      </div>
      <v-btn v-if="theaters.length === 0" color="amber-darken-1" variant="flat" prepend-icon="mdi-plus" class="mt-4 text-none font-weight-bold" @click="openCreateTheaterDialog">
        Create Theater
      </v-btn>
    </v-card>

    <!-- Theaters Table -->
    <div v-else class="table-container rounded-xl border overflow-hidden">
      <v-table theme="dark" class="theaters-table">
        <thead>
          <tr>
            <th class="text-left font-weight-bold">Theater Venue</th>
            <th class="text-left font-weight-bold">Address / Location</th>
            <th class="text-left font-weight-bold">Theater ID</th>
            <th class="text-right font-weight-bold">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="theater in paginatedTheaters"
            :key="theater.id"
            class="clickable-row"
            @click="goToSeatMapEditor(theater.id)"
          >
            <!-- Name -->
            <td class="py-3">
              <div class="d-flex align-center gap-3">
                <v-avatar color="indigo-darken-3" size="36" rounded="md">
                  <v-icon icon="mdi-theater" color="indigo-lighten-2" size="20"></v-icon>
                </v-avatar>
                <div>
                  <div class="font-weight-bold text-body-1 text-white">{{ theater.name }}</div>
                  <div class="text-caption text-grey">Click to open Seat Map Editor</div>
                </div>
              </div>
            </td>

            <!-- Address -->
            <td>
              <div class="d-flex align-center text-caption text-grey-lighten-1">
                <v-icon icon="mdi-map-marker-outline" size="14" color="amber-darken-1" class="mr-1" v-if="theater.address"></v-icon>
                <span>{{ theater.address || '—' }}</span>
              </div>
            </td>

            <!-- ID -->
            <td>
              <span class="font-weight-mono text-caption text-amber-darken-1 font-weight-bold">
                #{{ theater.id.substring(0, 8) }}
              </span>
            </td>

            <!-- Actions -->
            <td class="text-right">
              <div class="d-flex align-center justify-end gap-1" @click.stop>
                <v-btn
                  variant="tonal"
                  size="small"
                  color="indigo-lighten-2"
                  prepend-icon="mdi-seat-passenger"
                  class="text-none font-weight-bold"
                  :to="`/manage/theaters/${theater.id}/seat-map`"
                >
                  Edit Seat Map
                </v-btn>
                <v-btn
                  icon="mdi-pencil"
                  size="small"
                  variant="text"
                  color="grey"
                  title="Edit Theater Details"
                  @click.stop="openEditTheaterDialog(theater)"
                ></v-btn>
                <v-btn
                  icon="mdi-delete-outline"
                  size="small"
                  variant="text"
                  color="red-lighten-2"
                  title="Delete Theater"
                  @click.stop="openDeleteTheaterDialog(theater)"
                ></v-btn>
              </div>
            </td>
          </tr>
        </tbody>
      </v-table>

      <!-- Pagination & Limit Footer -->
      <div class="d-flex flex-wrap justify-space-between align-center pa-4 bg-surface border-t gap-4">
        <div class="text-caption text-grey">
          Showing <strong>{{ paginationStart }}</strong>–<strong>{{ paginationEnd }}</strong> of <strong>{{ filteredTheaters.length }}</strong> theaters
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

    <!-- Theater Dialog (Create / Edit) -->
    <v-dialog v-model="theaterDialog.show" max-width="500">
      <v-card theme="dark" class="rounded-xl pa-4">
        <v-card-title class="text-h6 font-weight-bold">
          {{ theaterDialog.isEdit ? 'Edit Theater Venue' : 'Create New Theater' }}
        </v-card-title>
        <v-card-text>
          <v-text-field
            v-model="theaterDialog.name"
            label="Theater Name"
            placeholder="e.g. Grand Horizon IMAX Cinema"
            variant="outlined"
            density="comfortable"
            autofocus
            class="mb-3"
          ></v-text-field>
          <v-text-field
            v-model="theaterDialog.address"
            label="Address / Location (Optional)"
            placeholder="e.g. 100 Main Street, Floor 3"
            variant="outlined"
            density="comfortable"
            class="mb-2"
            @keyup.enter="saveTheater"
          ></v-text-field>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="theaterDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="amber-darken-1"
            variant="flat"
            class="font-weight-bold text-none"
            :loading="theaterDialog.saving"
            @click="saveTheater"
          >
            {{ theaterDialog.isEdit ? 'Save Changes' : 'Create Theater' }}
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
            This will permanently remove the theater, its zones, seats, and any associated showtimes.
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
            Delete Theater
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

const theaterDialog = ref({
  show: false,
  isEdit: false,
  theaterId: null,
  name: '',
  address: '',
  saving: false,
});

const deleteDialog = ref({
  show: false,
  theaterId: null,
  itemName: '',
  deleting: false,
});

function goToSeatMapEditor(theaterId) {
  router.push(`/manage/theaters/${theaterId}/seat-map`);
}

const filteredTheaters = computed(() => {
  if (!searchQuery.value || !searchQuery.value.trim()) {
    return theaters.value;
  }
  const q = searchQuery.value.trim().toLowerCase();
  return theaters.value.filter((t) => {
    const nameMatch = t.name.toLowerCase().includes(q);
    const idMatch = t.id.toLowerCase().includes(q);
    const addressMatch = (t.address || '').toLowerCase().includes(q);
    return nameMatch || idMatch || addressMatch;
  });
});

const totalPages = computed(() => {
  return Math.ceil(filteredTheaters.value.length / itemsPerPage.value) || 1;
});

const paginatedTheaters = computed(() => {
  const start = (page.value - 1) * itemsPerPage.value;
  return filteredTheaters.value.slice(start, start + itemsPerPage.value);
});

const paginationStart = computed(() => {
  if (filteredTheaters.value.length === 0) return 0;
  return (page.value - 1) * itemsPerPage.value + 1;
});

const paginationEnd = computed(() => {
  return Math.min(page.value * itemsPerPage.value, filteredTheaters.value.length);
});

watch([searchQuery, itemsPerPage], () => {
  page.value = 1;
});

async function loadData() {
  loading.value = true;
  try {
    const res = await api.get('/theaters');
    theaters.value = res.data;
  } catch (err) {
    showToast('Failed to load theaters', 'error');
  } finally {
    loading.value = false;
  }
}

function openCreateTheaterDialog() {
  theaterDialog.value = {
    show: true,
    isEdit: false,
    theaterId: null,
    name: '',
    address: '',
    saving: false,
  };
}

function openEditTheaterDialog(theater) {
  theaterDialog.value = {
    show: true,
    isEdit: true,
    theaterId: theater.id,
    name: theater.name,
    address: theater.address || '',
    saving: false,
  };
}

async function saveTheater() {
  if (!theaterDialog.value.name.trim()) {
    showToast('Please enter a theater name', 'warning');
    return;
  }
  theaterDialog.value.saving = true;
  try {
    if (theaterDialog.value.isEdit) {
      await api.put(`/theaters/${theaterDialog.value.theaterId}`, {
        name: theaterDialog.value.name.trim(),
        address: theaterDialog.value.address.trim() || null,
      });
      showToast('Theater updated successfully', 'success');
    } else {
      const res = await api.post('/theaters', {
        name: theaterDialog.value.name.trim(),
        address: theaterDialog.value.address.trim() || null,
      });
      showToast('Theater created successfully', 'success');
      theaterDialog.value.show = false;
      await loadData();
      router.push(`/manage/theaters/${res.data.id}/seat-map`);
      return;
    }
    theaterDialog.value.show = false;
    await loadData();
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to save theater', 'error');
  } finally {
    theaterDialog.value.saving = false;
  }
}

function openDeleteTheaterDialog(theater) {
  deleteDialog.value = {
    show: true,
    theaterId: theater.id,
    itemName: theater.name,
    deleting: false,
  };
}

async function confirmDelete() {
  deleteDialog.value.deleting = true;
  try {
    await api.delete(`/theaters/${deleteDialog.value.theaterId}`);
    showToast('Theater deleted successfully', 'success');
    deleteDialog.value.show = false;
    await loadData();
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to delete theater', 'error');
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

.theaters-table {
  background: transparent !important;
}

.clickable-row {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.clickable-row:hover {
  background-color: rgba(99, 102, 241, 0.08) !important;
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
