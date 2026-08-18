<template>
  <v-container fluid class="py-6 px-6 editor-container">
    <!-- Top Header & Actions Bar -->
    <div class="d-flex flex-wrap justify-space-between align-center gap-4 mb-6">
      <div>
        <div class="d-flex align-center gap-2 mb-1">
          <v-btn variant="text" size="small" prepend-icon="mdi-arrow-left" to="/manage/events" class="text-none font-weight-bold mr-2">
            Events
          </v-btn>
          <v-icon icon="mdi-seat-passenger" color="amber-darken-1" size="28"></v-icon>
          <h1 class="text-h4 font-weight-bold text-white mb-0">Seat Map Editor</h1>
          <v-chip v-if="isModified" size="small" color="amber-darken-1" variant="flat" class="font-weight-bold ml-2">
            Unsaved Changes ({{ draftSeats.length }} seats)
          </v-chip>
        </div>
        <span class="text-caption text-grey">
          Configure physical hall geometry, bulk generate rows, drag-select seats, and adjust zone tiers
        </span>
      </div>

      <!-- Main Action Controls -->
      <div class="d-flex align-center gap-2 flex-wrap">
        <!-- Theater Selector -->
        <v-select
          v-model="selectedTheaterId"
          :items="theaters"
          item-title="name"
          item-value="id"
          variant="outlined"
          density="compact"
          hide-details
          style="min-width: 220px;"
          @update:model-value="fetchLayout"
        ></v-select>

        <v-btn
          prepend-icon="mdi-grid"
          color="indigo-darken-1"
          variant="flat"
          size="small"
          class="text-none font-weight-bold"
          @click="openGridGeneratorDialog"
        >
          Grid Generator
        </v-btn>

        <v-btn
          prepend-icon="mdi-plus"
          variant="outlined"
          size="small"
          class="text-none font-weight-bold"
          @click="openAddSeatDialog"
        >
          Add Seat
        </v-btn>

        <v-btn
          prepend-icon="mdi-tag-multiple"
          variant="outlined"
          size="small"
          class="text-none font-weight-bold"
          @click="openZoneManagerDialog"
        >
          Manage Zones ({{ draftZones.length }})
        </v-btn>

        <v-btn
          v-if="isModified"
          variant="text"
          size="small"
          color="grey"
          class="text-none"
          @click="discardChanges"
        >
          Discard
        </v-btn>

        <v-btn
          color="amber-darken-1"
          variant="flat"
          prepend-icon="mdi-content-save"
          size="small"
          class="text-none font-weight-bold"
          :loading="saving"
          :disabled="!isModified"
          @click="saveLayout"
        >
          Save Layout Changes
        </v-btn>
      </div>
    </div>

    <!-- Main Workspace Layout (Canvas + Sticky Sidebar) -->
    <v-row>
      <!-- Canvas Area -->
      <v-col cols="12" :md="selectedSeatKeys.size > 0 ? 9 : 12">
        <v-card theme="dark" class="canvas-card rounded-xl pa-6 border bg-surface" elevation="3">
          <!-- Canvas Top Toolbar -->
          <div class="d-flex justify-space-between align-center mb-6 flex-wrap gap-4">
            <!-- Zone Legend -->
            <div class="d-flex align-center gap-3 flex-wrap">
              <span class="text-caption text-grey font-weight-bold text-uppercase">Zones:</span>
              <div
                v-for="zone in draftZones"
                :key="zone.id || zone.tempId"
                class="d-flex align-center gap-1 zone-legend-badge"
              >
                <div class="zone-color-dot" :style="{ background: getZoneColorHex(zone.name) }"></div>
                <span class="text-caption text-white font-weight-bold">{{ zone.name }}</span>
                <span class="text-caption text-grey font-weight-mono">(${{ (zone.price_cents / 100).toFixed(2) }})</span>
              </div>
            </div>

            <!-- Quick Selection Helpers -->
            <div class="d-flex align-center gap-2">
              <span class="text-caption text-grey">Drag box to select multiple</span>
              <v-btn size="x-small" variant="text" color="amber-darken-1" @click="selectAllSeats">
                Select All ({{ draftSeats.length }})
              </v-btn>
              <v-btn v-if="selectedSeatKeys.size > 0" size="x-small" variant="text" color="grey" @click="clearSelection">
                Deselect ({{ selectedSeatKeys.size }})
              </v-btn>
            </div>
          </div>

          <!-- Cinema Screen Indicator -->
          <div class="screen-wrapper mb-8">
            <div class="cinema-screen">
              <div class="screen-text">CINEMA SCREEN</div>
            </div>
          </div>

          <!-- Interactive Drag-Select Canvas -->
          <div
            ref="canvasRef"
            class="seat-canvas-container"
            @mousedown="handleCanvasMouseDown"
            @mousemove="handleCanvasMouseMove"
            @mouseup="handleCanvasMouseUp"
            @mouseleave="handleCanvasMouseUp"
          >
            <!-- Marquee Selection Rectangle Overlay -->
            <div
              v-if="marquee.active"
              class="marquee-selection-box"
              :style="{
                left: `${marquee.left}px`,
                top: `${marquee.top}px`,
                width: `${marquee.width}px`,
                height: `${marquee.height}px`,
              }"
            ></div>

            <!-- Empty State -->
            <div v-if="draftSeats.length === 0" class="text-center pa-12">
              <v-icon icon="mdi-seat-outline" size="56" color="grey" class="mb-3"></v-icon>
              <div class="text-h6 text-grey font-weight-medium">No seats in this theater yet</div>
              <div class="text-caption text-grey-darken-1 mt-1">Use the Grid Generator to create a complete auditorium layout in seconds.</div>
              <v-btn color="indigo-darken-1" variant="flat" prepend-icon="mdi-grid" class="mt-4 text-none font-weight-bold" @click="openGridGeneratorDialog">
                Open Grid Generator
              </v-btn>
            </div>

            <!-- Rows of Seats -->
            <div v-else class="d-flex flex-column gap-3 align-center">
              <div
                v-for="row in sortedRows"
                :key="row.label"
                class="seat-row d-flex align-center gap-2"
              >
                <!-- Row Selector Header Button -->
                <button
                  type="button"
                  class="row-header-btn"
                  :class="{ 'row-header-selected': isRowFullySelected(row.label) }"
                  :title="`Click to toggle selection for Row ${row.label}`"
                  @click.stop="toggleRowSelection(row.label)"
                >
                  {{ row.label }}
                </button>

                <!-- Seats in this row -->
                <div class="d-flex align-center gap-1 seat-row-items">
                  <div
                    v-for="seat in row.seats"
                    :key="`${seat.row_label}-${seat.seat_number}`"
                    :ref="el => registerSeatElement(el, seat.row_label, seat.seat_number)"
                    class="seat-node"
                    :class="{
                      'seat-node-selected': isSeatSelected(seat.row_label, seat.seat_number),
                    }"
                    :style="{
                      borderColor: getZoneColorHex(getZoneName(seat.zone_id)),
                      background: isSeatSelected(seat.row_label, seat.seat_number)
                        ? 'rgba(245, 158, 11, 0.35)'
                        : getZoneBgColorHex(getZoneName(seat.zone_id)),
                    }"
                    :title="`Row ${seat.row_label}, Seat ${seat.seat_number} (${getZoneName(seat.zone_id)} - $${(getZonePrice(seat.zone_id) / 100).toFixed(2)})`"
                    @click.stop="handleSeatClick(seat, $event)"
                  >
                    <span class="seat-number-label">{{ seat.seat_number }}</span>
                    <v-icon
                      v-if="isSeatSelected(seat.row_label, seat.seat_number)"
                      icon="mdi-check"
                      size="12"
                      color="amber-darken-1"
                      class="selected-check-icon"
                    ></v-icon>
                  </div>
                </div>

                <!-- Row Right Header Label -->
                <div class="row-header-label-right text-caption text-grey font-weight-bold font-weight-mono">
                  {{ row.label }}
                </div>
              </div>
            </div>
          </div>
        </v-card>
      </v-col>

      <!-- Selection & Bulk Actions Sidebar (Sticky when seats selected) -->
      <v-col v-if="selectedSeatKeys.size > 0" cols="12" md="3">
        <v-card theme="dark" class="rounded-xl pa-5 border bg-surface sticky-toolbar" elevation="4">
          <div class="d-flex justify-space-between align-center mb-4">
            <div class="d-flex align-center gap-2">
              <v-icon icon="mdi-checkbox-multiple-marked" color="amber-darken-1" size="20"></v-icon>
              <h3 class="text-h6 font-weight-bold text-white mb-0">Selected Seats</h3>
            </div>
            <v-chip size="small" color="amber-darken-1" variant="flat" class="font-weight-bold">
              {{ selectedSeatKeys.size }}
            </v-chip>
          </div>

          <div class="text-caption text-grey mb-4">
            Perform bulk actions on all <strong>{{ selectedSeatKeys.size }}</strong> selected seat(s).
          </div>

          <v-divider class="mb-4"></v-divider>

          <!-- Action 1: Batch Reassign Zone -->
          <div class="mb-4">
            <label class="text-caption text-grey font-weight-bold text-uppercase d-block mb-1">
              REASSIGN ZONE TIER
            </label>
            <v-select
              v-model="bulkZoneId"
              :items="draftZones"
              :item-title="item => `${item.name} ($${(item.price_cents / 100).toFixed(2)})`"
              item-value="id"
              variant="outlined"
              density="compact"
              hide-details
              placeholder="Select Zone..."
              class="mb-2"
            ></v-select>
            <v-btn
              color="indigo-darken-1"
              variant="flat"
              block
              size="small"
              class="text-none font-weight-bold"
              :disabled="!bulkZoneId"
              @click="applyBulkZone"
            >
              Apply Zone to {{ selectedSeatKeys.size }} Seats
            </v-btn>
          </div>

          <v-divider class="mb-4"></v-divider>

          <!-- Action 2: Row Shift / Renumber -->
          <div class="mb-4">
            <label class="text-caption text-grey font-weight-bold text-uppercase d-block mb-1">
              RENAME ROW LABEL
            </label>
            <div class="d-flex gap-2">
              <v-text-field
                v-model="bulkRowLabel"
                placeholder="e.g. C"
                variant="outlined"
                density="compact"
                hide-details
                maxlength="3"
                class="text-uppercase"
              ></v-text-field>
              <v-btn
                variant="outlined"
                size="small"
                class="text-none font-weight-bold"
                :disabled="!bulkRowLabel.trim()"
                @click="applyBulkRowLabel"
              >
                Apply Row
              </v-btn>
            </div>
          </div>

          <v-divider class="mb-4"></v-divider>

          <!-- Action 3: Bulk Delete -->
          <div class="d-flex flex-column gap-2">
            <v-btn
              color="red-darken-1"
              variant="flat"
              block
              prepend-icon="mdi-trash-can-outline"
              class="text-none font-weight-bold"
              @click="bulkDeleteSelectedSeats"
            >
              Delete {{ selectedSeatKeys.size }} Seats
            </v-btn>

            <v-btn
              variant="outlined"
              block
              size="small"
              class="text-none"
              @click="clearSelection"
            >
              Deselect All
            </v-btn>
          </div>
        </v-card>
      </v-col>
    </v-row>

    <!-- Grid Generator Dialog -->
    <v-dialog v-model="gridDialog.show" max-width="540">
      <v-card theme="dark" class="rounded-xl pa-5 border bg-surface">
        <v-card-title class="text-h6 font-weight-bold d-flex align-center gap-2">
          <v-icon icon="mdi-grid" color="amber-darken-1"></v-icon>
          Bulk Seat Grid Generator
        </v-card-title>
        <v-card-text>
          <div class="text-caption text-grey mb-4">
            Quickly generate rows and seats in a standardized auditorium matrix.
          </div>

          <v-row dense class="mb-2">
            <v-col cols="6">
              <v-text-field
                v-model="gridDialog.startRow"
                label="Start Row Label"
                placeholder="A"
                variant="outlined"
                density="comfortable"
                class="text-uppercase"
              ></v-text-field>
            </v-col>
            <v-col cols="6">
              <v-text-field
                v-model="gridDialog.endRow"
                label="End Row Label"
                placeholder="H"
                variant="outlined"
                density="comfortable"
                class="text-uppercase"
              ></v-text-field>
            </v-col>
          </v-row>

          <v-row dense class="mb-2">
            <v-col cols="6">
              <v-text-field
                v-model.number="gridDialog.startSeat"
                label="Start Seat Number"
                type="number"
                min="1"
                variant="outlined"
                density="comfortable"
              ></v-text-field>
            </v-col>
            <v-col cols="6">
              <v-text-field
                v-model.number="gridDialog.endSeat"
                label="End Seat Number"
                type="number"
                min="1"
                variant="outlined"
                density="comfortable"
              ></v-text-field>
            </v-col>
          </v-row>

          <v-select
            v-model="gridDialog.zoneId"
            label="Default Zone"
            :items="draftZones"
            :item-title="item => `${item.name} ($${(item.price_cents / 100).toFixed(2)})`"
            item-value="id"
            variant="outlined"
            density="comfortable"
            class="mb-3"
          ></v-select>

          <v-radio-group v-model="gridDialog.mode" inline density="compact">
            <v-radio label="Append to existing layout" value="append"></v-radio>
            <v-radio label="Replace all existing seats" value="replace" color="red-accent-2"></v-radio>
          </v-radio-group>

          <v-alert type="info" variant="tonal" density="compact" class="rounded-lg">
            Will generate <strong>{{ calculatedGridCount }}</strong> seats.
          </v-alert>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="gridDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn
            color="amber-darken-1"
            variant="flat"
            class="font-weight-bold text-none"
            :disabled="calculatedGridCount <= 0 || !gridDialog.zoneId"
            @click="executeGridGenerator"
          >
            Generate {{ calculatedGridCount }} Seats
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Single Add Seat Dialog -->
    <v-dialog v-model="singleSeatDialog.show" max-width="420">
      <v-card theme="dark" class="rounded-xl pa-5 border bg-surface">
        <v-card-title class="text-h6 font-weight-bold">Add Individual Seat</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="singleSeatDialog.rowLabel"
            label="Row Label"
            placeholder="e.g. A"
            variant="outlined"
            density="comfortable"
            class="text-uppercase mb-2"
          ></v-text-field>
          <v-text-field
            v-model.number="singleSeatDialog.seatNumber"
            label="Seat Number"
            type="number"
            min="1"
            variant="outlined"
            density="comfortable"
            class="mb-2"
          ></v-text-field>
          <v-select
            v-model="singleSeatDialog.zoneId"
            label="Zone Tier"
            :items="draftZones"
            :item-title="item => `${item.name} ($${(item.price_cents / 100).toFixed(2)})`"
            item-value="id"
            variant="outlined"
            density="comfortable"
          ></v-select>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-btn variant="text" @click="singleSeatDialog.show = false">Cancel</v-btn>
          <v-spacer></v-spacer>
          <v-btn color="amber-darken-1" variant="flat" class="font-weight-bold text-none" @click="executeAddSingleSeat">
            Add Seat
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Zone Manager Dialog -->
    <v-dialog v-model="zoneDialog.show" max-width="600">
      <v-card theme="dark" class="rounded-xl pa-5 border bg-surface">
        <v-card-title class="text-h6 font-weight-bold d-flex justify-space-between align-center">
          <span>Manage Pricing Zones</span>
          <v-btn size="small" color="amber-darken-1" variant="flat" prepend-icon="mdi-plus" class="text-none font-weight-bold" @click="addNewDraftZone">
            New Zone
          </v-btn>
        </v-card-title>
        <v-card-text>
          <div class="text-caption text-grey mb-4">
            Define seating categories, names, and ticket pricing in USD for this theater hall.
          </div>

          <v-table theme="dark" class="rounded-lg border mb-4">
            <thead>
              <tr>
                <th class="text-left font-weight-bold">Zone Name</th>
                <th class="text-left font-weight-bold" style="width: 140px;">Price (USD)</th>
                <th class="text-right font-weight-bold" style="width: 80px;">Action</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(z, idx) in draftZones" :key="z.id || z.tempId">
                <td class="py-2">
                  <v-text-field
                    v-model="z.name"
                    placeholder="e.g. VIP Lounge"
                    variant="outlined"
                    density="compact"
                    hide-details
                  ></v-text-field>
                </td>
                <td>
                  <v-text-field
                    :model-value="(z.price_cents / 100).toFixed(2)"
                    prefix="$"
                    type="number"
                    step="0.50"
                    min="0"
                    variant="outlined"
                    density="compact"
                    hide-details
                    @update:model-value="val => z.price_cents = Math.round(parseFloat(val || 0) * 100)"
                  ></v-text-field>
                </td>
                <td class="text-right">
                  <v-btn
                    icon="mdi-delete-outline"
                    size="small"
                    variant="text"
                    color="red-lighten-2"
                    :disabled="draftZones.length <= 1"
                    @click="removeDraftZone(idx)"
                  ></v-btn>
                </td>
              </tr>
            </tbody>
          </v-table>
        </v-card-text>
        <v-card-actions class="pa-4 pt-0">
          <v-spacer></v-spacer>
          <v-btn color="amber-darken-1" variant="flat" class="font-weight-bold text-none px-6" @click="zoneDialog.show = false">
            Done
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
import { ref, computed, onMounted, reactive } from 'vue';
import { useRoute } from 'vue-router';
import api from '../api/client';

const route = useRoute();

const theaters = ref([]);
const selectedTheaterId = ref(null);
const loading = ref(false);
const saving = ref(false);

const originalLayout = ref({ theater: null, zones: [], seats: [] });
const draftZones = ref([]);
const draftSeats = ref([]);

const selectedSeatKeys = ref(new Set()); // Set of "ROW-SEATNUM" strings
const seatElementMap = new Map(); // "ROW-SEATNUM" -> DOMElement

const bulkZoneId = ref(null);
const bulkRowLabel = ref('');

const canvasRef = ref(null);

const marquee = reactive({
  active: false,
  startX: 0,
  startY: 0,
  left: 0,
  top: 0,
  width: 0,
  height: 0,
});

const gridDialog = ref({
  show: false,
  startRow: 'A',
  endRow: 'H',
  startSeat: 1,
  endSeat: 12,
  zoneId: null,
  mode: 'append',
});

const singleSeatDialog = ref({
  show: false,
  rowLabel: 'A',
  seatNumber: 1,
  zoneId: null,
});

const zoneDialog = ref({
  show: false,
});

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

function registerSeatElement(el, rowLabel, seatNumber) {
  if (el) {
    seatElementMap.set(`${rowLabel}-${seatNumber}`, el);
  }
}

// Group and sort draft seats into rows
const sortedRows = computed(() => {
  const rowMap = new Map();
  for (const s of draftSeats.value) {
    if (!rowMap.has(s.row_label)) {
      rowMap.set(s.row_label, []);
    }
    rowMap.get(s.row_label).push(s);
  }

  const sortedLabels = Array.from(rowMap.keys()).sort((a, b) => a.localeCompare(b));
  return sortedLabels.map(label => {
    const seats = rowMap.get(label).sort((a, b) => a.seat_number - b.seat_number);
    return { label, seats };
  });
});

const isModified = computed(() => {
  const origSeats = originalLayout.value.seats;
  if (origSeats.length !== draftSeats.value.length) return true;

  const origZones = originalLayout.value.zones;
  if (origZones.length !== draftZones.value.length) return true;

  // Check seat changes
  const origMap = new Map(origSeats.map(s => [`${s.row_label}-${s.seat_number}`, s.zone_id]));
  for (const s of draftSeats.value) {
    const origZone = origMap.get(`${s.row_label}-${s.seat_number}`);
    if (!origZone || origZone !== s.zone_id) return true;
  }

  return false;
});

const calculatedGridCount = computed(() => {
  const startChar = (gridDialog.value.startRow || 'A').toUpperCase().charCodeAt(0);
  const endChar = (gridDialog.value.endRow || 'A').toUpperCase().charCodeAt(0);
  const rowCount = Math.max(0, endChar - startChar + 1);
  const seatCount = Math.max(0, (gridDialog.value.endSeat - gridDialog.value.startSeat + 1));
  return rowCount * seatCount;
});

function getZoneName(zoneId) {
  const z = draftZones.value.find(item => item.id === zoneId || item.tempId === zoneId);
  return z ? z.name : 'Unknown';
}

function getZonePrice(zoneId) {
  const z = draftZones.value.find(item => item.id === zoneId || item.tempId === zoneId);
  return z ? z.price_cents : 0;
}

function getZoneColorHex(name) {
  if (!name) return '#94a3b8';
  if (name.toLowerCase().includes('vip')) return '#f59e0b';
  if (name.toLowerCase().includes('premium')) return '#6366f1';
  return '#14b8a6';
}

function getZoneBgColorHex(name) {
  if (!name) return 'rgba(148, 163, 184, 0.15)';
  if (name.toLowerCase().includes('vip')) return 'rgba(245, 158, 11, 0.18)';
  if (name.toLowerCase().includes('premium')) return 'rgba(99, 102, 241, 0.18)';
  return 'rgba(20, 184, 166, 0.18)';
}

function isSeatSelected(rowLabel, seatNumber) {
  return selectedSeatKeys.value.has(`${rowLabel}-${seatNumber}`);
}

function isRowFullySelected(rowLabel) {
  const rowSeats = draftSeats.value.filter(s => s.row_label === rowLabel);
  if (rowSeats.length === 0) return false;
  return rowSeats.every(s => isSeatSelected(s.row_label, s.seat_number));
}

function toggleRowSelection(rowLabel) {
  const rowSeats = draftSeats.value.filter(s => s.row_label === rowLabel);
  const fullySelected = isRowFullySelected(rowLabel);

  const nextSet = new Set(selectedSeatKeys.value);
  for (const s of rowSeats) {
    const key = `${s.row_label}-${s.seat_number}`;
    if (fullySelected) {
      nextSet.delete(key);
    } else {
      nextSet.add(key);
    }
  }
  selectedSeatKeys.value = nextSet;
}

function handleSeatClick(seat, event) {
  const key = `${seat.row_label}-${seat.seat_number}`;
  const nextSet = new Set(selectedSeatKeys.value);

  if (event.shiftKey || event.ctrlKey || event.metaKey) {
    if (nextSet.has(key)) {
      nextSet.delete(key);
    } else {
      nextSet.add(key);
    }
  } else {
    if (nextSet.has(key) && nextSet.size === 1) {
      nextSet.clear();
    } else {
      nextSet.clear();
      nextSet.add(key);
    }
  }
  selectedSeatKeys.value = nextSet;
}

function selectAllSeats() {
  const nextSet = new Set();
  for (const s of draftSeats.value) {
    nextSet.add(`${s.row_label}-${s.seat_number}`);
  }
  selectedSeatKeys.value = nextSet;
}

function clearSelection() {
  selectedSeatKeys.value = new Set();
}

// Marquee Box Drag Selection Handlers
function handleCanvasMouseDown(e) {
  if (e.target.closest('.seat-node') || e.target.closest('.row-header-btn')) return;
  if (!canvasRef.value) return;

  const rect = canvasRef.value.getBoundingClientRect();
  marquee.active = true;
  marquee.startX = e.clientX - rect.left;
  marquee.startY = e.clientY - rect.top;
  marquee.left = marquee.startX;
  marquee.top = marquee.startY;
  marquee.width = 0;
  marquee.height = 0;

  if (!e.shiftKey && !e.ctrlKey) {
    selectedSeatKeys.value = new Set();
  }
}

function handleCanvasMouseMove(e) {
  if (!marquee.active || !canvasRef.value) return;

  const rect = canvasRef.value.getBoundingClientRect();
  const currentX = e.clientX - rect.left;
  const currentY = e.clientY - rect.top;

  marquee.left = Math.min(marquee.startX, currentX);
  marquee.top = Math.min(marquee.startY, currentY);
  marquee.width = Math.abs(currentX - marquee.startX);
  marquee.height = Math.abs(currentY - marquee.startY);

  // Compute intersections with seat elements
  const boxRect = {
    left: marquee.left + rect.left,
    top: marquee.top + rect.top,
    right: marquee.left + marquee.width + rect.left,
    bottom: marquee.top + marquee.height + rect.top,
  };

  const nextSet = new Set(selectedSeatKeys.value);
  for (const [key, el] of seatElementMap.entries()) {
    if (!el) continue;
    const elRect = el.getBoundingClientRect();
    const overlaps = !(
      elRect.right < boxRect.left ||
      elRect.left > boxRect.right ||
      elRect.bottom < boxRect.top ||
      elRect.top > boxRect.bottom
    );
    if (overlaps) {
      nextSet.add(key);
    }
  }
  selectedSeatKeys.value = nextSet;
}

function handleCanvasMouseUp() {
  marquee.active = false;
}

// Bulk Actions
function applyBulkZone() {
  if (!bulkZoneId.value) return;
  for (const s of draftSeats.value) {
    if (selectedSeatKeys.value.has(`${s.row_label}-${s.seat_number}`)) {
      s.zone_id = bulkZoneId.value;
    }
  }
  showToast(`Reassigned ${selectedSeatKeys.value.size} seats to ${getZoneName(bulkZoneId.value)}`, 'success');
}

function applyBulkRowLabel() {
  const newRow = bulkRowLabel.value.trim().toUpperCase();
  if (!newRow) return;

  for (const s of draftSeats.value) {
    if (selectedSeatKeys.value.has(`${s.row_label}-${s.seat_number}`)) {
      s.row_label = newRow;
    }
  }
  bulkRowLabel.value = '';
  clearSelection();
  showToast(`Updated row labels to ${newRow}`, 'success');
}

function bulkDeleteSelectedSeats() {
  const count = selectedSeatKeys.value.size;
  draftSeats.value = draftSeats.value.filter(
    s => !selectedSeatKeys.value.has(`${s.row_label}-${s.seat_number}`)
  );
  clearSelection();
  showToast(`Deleted ${count} seat(s)`, 'info');
}

// Dialogs & Generators
function openGridGeneratorDialog() {
  gridDialog.value = {
    show: true,
    startRow: 'A',
    endRow: 'H',
    startSeat: 1,
    endSeat: 12,
    zoneId: draftZones.value[0]?.id || draftZones.value[0]?.tempId || null,
    mode: 'append',
  };
}

function executeGridGenerator() {
  const startChar = gridDialog.value.startRow.toUpperCase().charCodeAt(0);
  const endChar = gridDialog.value.endRow.toUpperCase().charCodeAt(0);
  const startNum = gridDialog.value.startSeat;
  const endNum = gridDialog.value.endSeat;
  const targetZoneId = gridDialog.value.zoneId;

  const newSeats = [];
  for (let c = startChar; c <= endChar; c++) {
    const rowLabel = String.fromCharCode(c);
    for (let num = startNum; num <= endNum; num++) {
      newSeats.push({
        id: null,
        zone_id: targetZoneId,
        row_label: rowLabel,
        seat_number: num,
      });
    }
  }

  if (gridDialog.value.mode === 'replace') {
    draftSeats.value = newSeats;
  } else {
    // Append or update existing keys
    const existingMap = new Map(draftSeats.value.map(s => [`${s.row_label}-${s.seat_number}`, s]));
    for (const ns of newSeats) {
      existingMap.set(`${ns.row_label}-${ns.seat_number}`, ns);
    }
    draftSeats.value = Array.from(existingMap.values());
  }

  gridDialog.value.show = false;
  clearSelection();
  showToast(`Generated ${newSeats.length} seats`, 'success');
}

function openAddSeatDialog() {
  singleSeatDialog.value = {
    show: true,
    rowLabel: 'A',
    seatNumber: draftSeats.value.length > 0 ? Math.max(...draftSeats.value.map(s => s.seat_number)) + 1 : 1,
    zoneId: draftZones.value[0]?.id || draftZones.value[0]?.tempId || null,
  };
}

function executeAddSingleSeat() {
  const row = singleSeatDialog.value.rowLabel.trim().toUpperCase();
  const num = singleSeatDialog.value.seatNumber;
  const zoneId = singleSeatDialog.value.zoneId;

  if (!row || num <= 0 || !zoneId) {
    showToast('Please fill out all fields', 'warning');
    return;
  }

  const existingIdx = draftSeats.value.findIndex(s => s.row_label === row && s.seat_number === num);
  if (existingIdx >= 0) {
    draftSeats.value[existingIdx].zone_id = zoneId;
  } else {
    draftSeats.value.push({
      id: null,
      zone_id: zoneId,
      row_label: row,
      seat_number: num,
    });
  }

  singleSeatDialog.value.show = false;
  showToast(`Seat ${row}${num} added`, 'success');
}

function openZoneManagerDialog() {
  zoneDialog.value.show = true;
}

function addNewDraftZone() {
  const tempId = `temp_zone_${Date.now()}`;
  draftZones.value.push({
    id: null,
    tempId,
    name: `Zone ${draftZones.value.length + 1}`,
    price_cents: 1500,
  });
}

function removeDraftZone(index) {
  if (draftZones.value.length <= 1) return;
  const removed = draftZones.value.splice(index, 1)[0];
  const fallbackZoneId = draftZones.value[0]?.id || draftZones.value[0]?.tempId;

  // Reassign any seats in the removed zone to fallback zone
  for (const s of draftSeats.value) {
    if (s.zone_id === removed.id || s.zone_id === removed.tempId) {
      s.zone_id = fallbackZoneId;
    }
  }
}

function discardChanges() {
  draftZones.value = JSON.parse(JSON.stringify(originalLayout.value.zones));
  draftSeats.value = JSON.parse(JSON.stringify(originalLayout.value.seats));
  clearSelection();
  showToast('Draft changes discarded', 'info');
}

async function fetchLayout() {
  if (!selectedTheaterId.value) return;
  loading.value = true;
  clearSelection();
  seatElementMap.clear();

  try {
    const res = await api.get(`/theaters/${selectedTheaterId.value}/layout`);
    originalLayout.value = res.data;
    draftZones.value = JSON.parse(JSON.stringify(res.data.zones || []));
    draftSeats.value = JSON.parse(JSON.stringify(res.data.seats || []));
  } catch (err) {
    showToast('Failed to load theater seat layout', 'error');
  } finally {
    loading.value = false;
  }
}

async function saveLayout() {
  if (!selectedTheaterId.value) return;
  saving.value = true;

  try {
    const payload = {
      zones: draftZones.value.map(z => ({
        id: z.id || null,
        name: z.name.trim(),
        price_cents: z.price_cents,
      })),
      seats: draftSeats.value.map(s => ({
        id: s.id || null,
        zone_id: s.zone_id,
        row_label: s.row_label.trim().toUpperCase(),
        seat_number: s.seat_number,
      })),
    };

    const res = await api.put(`/theaters/${selectedTheaterId.value}/layout`, payload);
    originalLayout.value = res.data;
    draftZones.value = JSON.parse(JSON.stringify(res.data.zones || []));
    draftSeats.value = JSON.parse(JSON.stringify(res.data.seats || []));
    clearSelection();
    showToast('Seat map layout saved successfully!', 'success');
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Failed to save layout';
    showToast(errMsg, 'error');
  } finally {
    saving.value = false;
  }
}

onMounted(async () => {
  try {
    const theatersRes = await api.get('/theaters');
    theaters.value = theatersRes.data;

    const routeTheaterId = route.params.id;
    if (routeTheaterId && theaters.value.some(t => t.id === routeTheaterId)) {
      selectedTheaterId.value = routeTheaterId;
    } else if (theaters.value.length > 0) {
      selectedTheaterId.value = theaters.value[0].id;
    }

    if (selectedTheaterId.value) {
      await fetchLayout();
    }
  } catch (err) {
    showToast('Failed to initialize editor', 'error');
  }
});
</script>

<style scoped>
.editor-container {
  max-width: 1440px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.canvas-card {
  position: relative;
  user-select: none;
}

.screen-wrapper {
  perspective: 400px;
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
}

.cinema-screen {
  height: 24px;
  background: linear-gradient(180deg, #f59e0b 0%, rgba(245, 158, 11, 0.1) 100%);
  transform: rotateX(-20deg);
  border-radius: 50% 50% 0 0;
  box-shadow: 0 10px 25px rgba(245, 158, 11, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
}

.screen-text {
  font-size: 10px;
  letter-spacing: 4px;
  color: #fff;
  font-weight: 900;
  opacity: 0.9;
}

.seat-canvas-container {
  position: relative;
  min-height: 480px;
  padding: 30px;
  background: #131722;
  border-radius: 16px;
  border: 1px dashed #2a3142;
  overflow-x: auto;
}

.marquee-selection-box {
  position: absolute;
  pointer-events: none;
  background: rgba(245, 158, 11, 0.15);
  border: 1px dashed #f59e0b;
  border-radius: 4px;
  z-index: 10;
}

.seat-row {
  flex-wrap: nowrap;
}

.row-header-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1px solid #3b4256;
  background: #1e2330;
  color: #94a3b8;
  font-size: 11px;
  font-weight: bold;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.row-header-btn:hover {
  background: #f59e0b;
  color: #000;
  border-color: #f59e0b;
}

.row-header-selected {
  background: #f59e0b !important;
  color: #000 !important;
  border-color: #f59e0b !important;
}

.row-header-label-right {
  width: 28px;
  text-align: center;
}

.seat-node {
  position: relative;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1.5px solid #4b5563;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform 0.1s ease, border-color 0.15s ease, background 0.15s ease;
}

.seat-node:hover {
  transform: scale(1.15);
  z-index: 5;
}

.seat-node-selected {
  transform: scale(1.12);
  border-color: #f59e0b !important;
  border-width: 2px !important;
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.6);
  z-index: 6;
}

.seat-number-label {
  font-size: 11px;
  font-weight: 700;
  color: #fff;
}

.selected-check-icon {
  position: absolute;
  top: -4px;
  right: -4px;
  background: #181c27;
  border-radius: 50%;
}

.zone-color-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.sticky-toolbar {
  position: sticky;
  top: 80px;
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
