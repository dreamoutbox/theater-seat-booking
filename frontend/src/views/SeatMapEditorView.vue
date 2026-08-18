<template>
  <v-container fluid class="pa-2 max-w-full editor-container">
    <!-- Top Header & Actions Bar (Compact) -->
    <div class="d-flex flex-wrap justify-space-between align-center gap-2 mb-2">
      <div>
        <div class="d-flex align-center gap-2 mb-0">
          <v-btn variant="text" size="small" density="compact" prepend-icon="mdi-arrow-left" to="/manage/theaters" class="text-none font-weight-bold mr-1">
            Theaters
          </v-btn>
          <v-icon icon="mdi-seat-passenger" color="amber-darken-1" size="22"></v-icon>
          <h1 class="text-h6 font-weight-bold text-white mb-0">Seat Map Editor</h1>
          <v-chip v-if="currentTheaterName" size="small" color="indigo-lighten-2" variant="tonal" class="font-weight-bold ml-1">
            <v-icon start icon="mdi-theater" size="14"></v-icon>
            {{ currentTheaterName }}
          </v-chip>
          <v-chip v-if="isModified" size="x-small" color="amber-darken-1" variant="flat" class="font-weight-bold ml-1">
            Unsaved ({{ draftSeats.length }} seats)
          </v-chip>
        </div>
        <span class="text-caption text-grey" style="font-size: 11px;">
          Rows A–ZZ, Seats 1–100 • Drag seats to move • Drag box over empty slots to create • Press Delete to remove
        </span>
      </div>

      <!-- Main Action Controls (Compact) -->
      <div class="d-flex align-center gap-2 flex-wrap">
        <v-btn
          prepend-icon="mdi-grid"
          color="indigo-darken-1"
          variant="flat"
          size="small"
          class="text-none font-weight-bold"
          @click="openGridGeneratorDialog"
        >
          Seat Generator
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
          Zones ({{ draftZones.length }})
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
          Save Layout
        </v-btn>
      </div>
    </div>

    <!-- Main Workspace Layout (Canvas + Persistent Sidebar) -->
    <v-row dense class="ma-0">
      <!-- Canvas Area (9.5 Columns) -->
      <v-col cols="12" md="9" lg="9" xl="10" class="pa-1">
        <v-card theme="dark" class="canvas-card rounded-lg pa-3 border bg-surface" elevation="2">
          <!-- Canvas Top Toolbar (Compact) -->
          <div class="d-flex justify-space-between align-center mb-2 flex-wrap gap-2">
            <!-- Zone Legend -->
            <div class="d-flex align-center gap-2 flex-wrap">
              <span class="text-caption text-grey font-weight-bold text-uppercase" style="font-size: 10px;">Zones:</span>
              <div
                v-for="zone in draftZones"
                :key="zone.id"
                class="d-flex align-center gap-1 zone-legend-badge"
              >
                <div class="zone-color-dot" :style="{ background: getZoneColorHex(zone.name) }"></div>
                <span class="text-caption text-white font-weight-bold" style="font-size: 11px;">{{ zone.name }}</span>
                <span class="text-caption text-grey font-weight-mono" style="font-size: 10px;">(${{ (zone.price_cents / 100).toFixed(2) }})</span>
              </div>
            </div>

            <!-- Grid Scale & Helpers Controls (Compact) -->
            <div class="d-flex align-center gap-2 flex-wrap">
              <!-- Visible Columns Selector -->
              <div class="d-flex align-center gap-1">
                <span class="text-caption text-grey" style="font-size: 11px;">Cols:</span>
                <v-select
                  v-model="gridColumnsCount"
                  :items="[20, 30, 40, 50, 75, 100]"
                  variant="outlined"
                  density="compact"
                  hide-details
                  style="width: 75px;"
                ></v-select>
              </div>

              <!-- Visible Rows Selector -->
              <div class="d-flex align-center gap-1">
                <span class="text-caption text-grey" style="font-size: 11px;">Rows:</span>
                <v-select
                  v-model="gridRowsSpan"
                  :items="[
                    { title: 'A–Z (26)', value: 'Z' },
                    { title: 'A–AZ (52)', value: 'AZ' },
                    { title: 'A–BZ (78)', value: 'BZ' },
                    { title: 'A–ZZ (702)', value: 'ZZ' },
                  ]"
                  item-title="title"
                  item-value="value"
                  variant="outlined"
                  density="compact"
                  hide-details
                  style="width: 115px;"
                ></v-select>
              </div>

              <v-btn size="x-small" variant="text" color="amber-darken-1" class="text-none" @click="selectAllSeats">
                Select All ({{ draftSeats.length }})
              </v-btn>
              <v-btn v-if="selectedSeatKeys.size > 0" size="x-small" variant="text" color="grey" class="text-none" @click="clearSelection">
                Deselect ({{ selectedSeatKeys.size }})
              </v-btn>
            </div>
          </div>

          <!-- Cinema Screen Indicator (Compact) -->
          <div class="screen-wrapper mb-3">
            <div class="cinema-screen">
              <div class="screen-text">CINEMA SCREEN</div>
            </div>
          </div>

          <!-- Interactive Drag Canvas -->
          <div
            ref="canvasRef"
            class="seat-canvas-container"
            :class="{ 'cursor-move-active': dragAction.type === 'move' }"
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

            <!-- Grid Header Column Numbers (1 to gridMaxCols) -->
            <div class="d-flex align-center gap-1 mb-1">
              <div class="row-header-placeholder"></div>
              <div class="d-flex align-center gap-1 seat-row-items">
                <div
                  v-for="col in gridMaxCols"
                  :key="col"
                  class="col-header-number text-caption text-grey font-weight-bold font-weight-mono"
                >
                  {{ col }}
                </div>
              </div>
              <div class="row-header-placeholder"></div>
            </div>

            <!-- Rows of Grid Slots (A..Z, AA..ZZ) -->
            <div class="d-flex flex-column gap-1">
              <div
                v-for="row in gridRows"
                :key="row.label"
                class="seat-row d-flex align-center gap-1"
              >
                <!-- Row Left Selector Header Button -->
                <button
                  type="button"
                  class="row-header-btn"
                  :class="{
                    'row-header-selected': isRowFullySelected(row.label),
                    'row-header-empty': !hasSeatsInRow(row.label),
                  }"
                  :title="`Click to toggle selection for Row ${row.label}`"
                  @click.stop="toggleRowSelection(row.label)"
                >
                  {{ row.label }}
                </button>

                <!-- Grid Slots in this row -->
                <div class="d-flex align-center gap-1 seat-row-items">
                  <template v-for="slot in row.slots" :key="`${slot.rowLabel}-${slot.seatNumber}`">
                    <!-- Case 1: Active Seat exists -->
                    <div
                      v-if="slot.seat"
                      :ref="el => registerSlotElement(el, slot.rowLabel, slot.seatNumber)"
                      class="seat-node"
                      :class="{
                        'seat-node-selected': isSeatSelected(slot.rowLabel, slot.seatNumber),
                        'seat-node-move-preview': isSlotInMovePreview(slot.rowLabel, slot.seatNumber),
                      }"
                      :style="{
                        borderColor: getZoneColorHex(getZoneName(slot.seat.zone_id)),
                        background: isSeatSelected(slot.rowLabel, slot.seatNumber)
                          ? 'rgba(245, 158, 11, 0.35)'
                          : getZoneBgColorHex(getZoneName(slot.seat.zone_id)),
                      }"
                      :title="`Row ${slot.rowLabel}, Seat ${slot.seatNumber} (${getZoneName(slot.seat.zone_id)} - $${(getZonePrice(slot.seat.zone_id) / 100).toFixed(2)}) • Drag to move`"
                      :data-row="slot.rowLabel"
                      :data-seat="slot.seatNumber"
                    >
                      <span class="seat-number-label">{{ slot.seatNumber }}</span>
                      <v-icon
                        v-if="isSeatSelected(slot.rowLabel, slot.seatNumber)"
                        icon="mdi-check"
                        size="11"
                        color="amber-darken-1"
                        class="selected-check-icon"
                      ></v-icon>
                    </div>

                    <!-- Case 2: Empty Grid Slot (Walkway or Vacant Slot) -->
                    <div
                      v-else
                      :ref="el => registerSlotElement(el, slot.rowLabel, slot.seatNumber)"
                      class="empty-grid-slot"
                      :class="{
                        'hint-slot-active': isSlotInHint(slot.rowLabel, slot.seatNumber),
                        'seat-node-move-target': isSlotInMovePreview(slot.rowLabel, slot.seatNumber),
                      }"
                      :title="`Empty Slot: Row ${slot.rowLabel}, Seat ${slot.seatNumber} (Click or drag to create seat)`"
                      :data-row="slot.rowLabel"
                      :data-seat="slot.seatNumber"
                    >
                      <template v-if="isSlotInHint(slot.rowLabel, slot.seatNumber)">
                        <v-icon icon="mdi-plus" size="10" color="amber-darken-1"></v-icon>
                        <span class="hint-label">{{ slot.rowLabel }}{{ slot.seatNumber }}</span>
                      </template>
                      <template v-else-if="isSlotInMovePreview(slot.rowLabel, slot.seatNumber)">
                        <v-icon icon="mdi-arrow-down-bold" size="10" color="amber-darken-1"></v-icon>
                      </template>
                      <span v-else class="empty-slot-number">{{ slot.seatNumber }}</span>
                    </div>
                  </template>
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

      <!-- Selection & Bulk Actions Sidebar (Persistent in right column, Compact) -->
      <v-col cols="12" md="3" lg="3" xl="2" class="pa-1">
        <v-card theme="dark" class="rounded-lg pa-3 border bg-surface sticky-toolbar" elevation="3">
          <!-- Sidebar Header -->
          <div class="d-flex justify-space-between align-center mb-2">
            <div class="d-flex align-center gap-1">
              <v-icon icon="mdi-checkbox-multiple-marked" :color="selectedSeatKeys.size > 0 ? 'amber-darken-1' : 'grey'" size="18"></v-icon>
              <h3 class="text-subtitle-2 font-weight-bold text-white mb-0">Selection Actions</h3>
            </div>
            <v-chip size="x-small" :color="selectedSeatKeys.size > 0 ? 'amber-darken-1' : 'grey'" variant="flat" class="font-weight-bold">
              {{ selectedSeatKeys.size }} selected
            </v-chip>
          </div>

          <!-- Move Active Indicator -->
          <div v-if="dragAction.type === 'move'" class="pa-2 rounded-lg bg-amber-darken-4 border mb-2 text-caption">
            <v-icon icon="mdi-cursor-move" size="14" class="mr-1"></v-icon>
            <strong>Moving {{ selectedSeatKeys.size }} seat(s)...</strong>
            <div class="text-caption" style="font-size: 10px;">Release mouse over destination grid slots</div>
          </div>

          <!-- Hint Slots creation prompt if empty slots dragged -->
          <div v-else-if="hintSlotsToCreate.size > 0 && selectedSeatKeys.size === 0" class="pa-2 rounded-lg bg-indigo-darken-4 border mb-2">
            <div class="d-flex align-center gap-1 mb-1">
              <v-icon icon="mdi-creation" color="amber-darken-1" size="16"></v-icon>
              <span class="text-caption font-weight-bold text-white">{{ hintSlotsToCreate.size }} Empty Slots Selected</span>
            </div>
            <div class="text-caption text-grey-lighten-2 mb-2" style="font-size: 11px;">
              Rows <strong>{{ inferredRange.startRow }}–{{ inferredRange.endRow }}</strong> (Cols {{ inferredRange.startSeat }}–{{ inferredRange.endSeat }}).
            </div>
            <v-btn
              color="amber-darken-1"
              variant="flat"
              block
              size="x-small"
              prepend-icon="mdi-plus"
              class="text-none font-weight-bold mb-1"
              @click="createSeatsFromHintSlots"
            >
              Create {{ hintSlotsToCreate.size }} Seats Here
            </v-btn>
            <v-btn
              variant="text"
              block
              size="x-small"
              class="text-none text-grey"
              @click="hintSlotsToCreate.clear()"
            >
              Cancel
            </v-btn>
          </div>

          <!-- Empty selection helper -->
          <div v-else-if="selectedSeatKeys.size === 0" class="pa-2 rounded-lg bg-surface-variant border border-dashed mb-2 text-center">
            <v-icon icon="mdi-cursor-default-click" size="18" color="grey" class="mb-1"></v-icon>
            <div class="text-caption text-grey font-weight-medium" style="font-size: 11px;">No seats selected</div>
            <div class="text-caption text-grey-darken-1 mt-1" style="font-size: 10px;">
              Drag on canvas to select or drag selected seats to move.
            </div>
          </div>

          <div v-else class="text-caption text-grey mb-2" style="font-size: 11px;">
            Editing <strong>{{ selectedSeatKeys.size }}</strong> seat(s). Drag them to move.
          </div>

          <v-divider class="mb-2"></v-divider>

          <!-- Action 1: Batch Reassign Zone -->
          <div class="mb-2">
            <label class="text-caption text-grey font-weight-bold text-uppercase d-block mb-1" style="font-size: 10px;">
              REASSIGN ZONE
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
              :disabled="selectedSeatKeys.size === 0"
              class="mb-1"
            ></v-select>
            <v-btn
              color="indigo-darken-1"
              variant="flat"
              block
              size="x-small"
              class="text-none font-weight-bold"
              :disabled="selectedSeatKeys.size === 0 || !bulkZoneId"
              @click="applyBulkZone"
            >
              Apply to {{ selectedSeatKeys.size }} Seats
            </v-btn>
          </div>

          <v-divider class="mb-2"></v-divider>

          <!-- Action 2: Row Shift / Renumber -->
          <div class="mb-2">
            <label class="text-caption text-grey font-weight-bold text-uppercase d-block mb-1" style="font-size: 10px;">
              RENAME ROW
            </label>
            <div class="d-flex gap-1">
              <v-text-field
                v-model="bulkRowLabel"
                placeholder="e.g. AA"
                variant="outlined"
                density="compact"
                hide-details
                maxlength="4"
                :disabled="selectedSeatKeys.size === 0"
                class="text-uppercase"
              ></v-text-field>
              <v-btn
                variant="outlined"
                size="small"
                class="text-none font-weight-bold"
                :disabled="selectedSeatKeys.size === 0 || !bulkRowLabel.trim()"
                @click="applyBulkRowLabel"
              >
                Apply
              </v-btn>
            </div>
          </div>

          <v-divider class="mb-2"></v-divider>

          <!-- Action 3: Quick Generator from Selection -->
          <div class="mb-2">
            <v-btn
              color="indigo-darken-1"
              variant="outlined"
              block
              size="x-small"
              prepend-icon="mdi-grid"
              class="text-none font-weight-bold"
              @click="openGridGeneratorDialog"
            >
              Seat Generator Wizard
            </v-btn>
          </div>

          <v-divider class="mb-2"></v-divider>

          <!-- Action 4: Bulk Delete & Clear -->
          <div class="d-flex flex-column gap-1">
            <v-btn
              color="red-darken-1"
              variant="flat"
              block
              size="small"
              prepend-icon="mdi-trash-can-outline"
              class="text-none font-weight-bold"
              :disabled="selectedSeatKeys.size === 0"
              @click="bulkDeleteSelectedSeats"
            >
              Delete {{ selectedSeatKeys.size }} Seats
            </v-btn>

            <v-btn
              v-if="selectedSeatKeys.size > 0"
              variant="outlined"
              block
              size="x-small"
              class="text-none"
              @click="clearSelection"
            >
              Deselect All
            </v-btn>
          </div>
        </v-card>
      </v-col>
    </v-row>

    <!-- Seat Generator Dialog -->
    <v-dialog v-model="gridDialog.show" max-width="540">
      <v-card theme="dark" class="rounded-xl pa-4 border bg-surface">
        <v-card-title class="text-h6 font-weight-bold d-flex align-center gap-2">
          <v-icon icon="mdi-grid" color="amber-darken-1"></v-icon>
          Bulk Seat Generator (A–ZZ, 1–100)
        </v-card-title>
        <v-card-text>
          <div class="text-caption text-grey mb-3">
            Generate rows (supports <strong>A..Z</strong> and <strong>AA..ZZ</strong>) and seat numbers up to <strong>100</strong>.
          </div>

          <v-row dense class="mb-2">
            <v-col cols="6">
              <v-text-field
                v-model="gridDialog.startRow"
                label="Start Row (e.g. A, AA)"
                placeholder="A"
                variant="outlined"
                density="comfortable"
                class="text-uppercase"
              ></v-text-field>
            </v-col>
            <v-col cols="6">
              <v-text-field
                v-model="gridDialog.endRow"
                label="End Row (e.g. Z, ZZ)"
                placeholder="Z"
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
                max="100"
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
                max="100"
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
            Will generate <strong>{{ calculatedGridCount }}</strong> seat(s) across rows <strong>{{ (gridDialog.startRow || 'A').toUpperCase() }}–{{ (gridDialog.endRow || 'Z').toUpperCase() }}</strong> (Seats {{ gridDialog.startSeat }}–{{ gridDialog.endSeat }}).
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
    <v-dialog v-model="singleSeatDialog.show" max-width="400">
      <v-card theme="dark" class="rounded-xl pa-4 border bg-surface">
        <v-card-title class="text-h6 font-weight-bold">Add Individual Seat</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="singleSeatDialog.rowLabel"
            label="Row Label (e.g. A, AA, ZZ)"
            placeholder="e.g. AA"
            variant="outlined"
            density="comfortable"
            class="text-uppercase mb-2"
          ></v-text-field>
          <v-text-field
            v-model.number="singleSeatDialog.seatNumber"
            label="Seat Number (1–100)"
            type="number"
            min="1"
            max="100"
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
    <v-dialog v-model="zoneDialog.show" max-width="720">
      <v-card theme="dark" class="rounded-xl pa-4 border bg-surface">
        <v-card-title class="text-h6 font-weight-bold d-flex justify-space-between align-center">
          <div class="d-flex align-center gap-2">
            <v-icon icon="mdi-palette" color="amber-darken-1" size="22"></v-icon>
            <span>Manage Pricing Zones & Colors</span>
          </div>
          <v-btn size="small" color="amber-darken-1" variant="flat" prepend-icon="mdi-plus" class="text-none font-weight-bold" @click="addNewDraftZone">
            New Zone
          </v-btn>
        </v-card-title>
        <v-card-text>
          <div class="text-caption text-grey mb-3">
            Define seating categories, pricing in USD, and custom zone colors for the seat map canvas.
          </div>

          <v-table theme="dark" class="rounded-lg border mb-3">
            <thead>
              <tr>
                <th class="text-left font-weight-bold" style="width: 170px;">Zone Name</th>
                <th class="text-left font-weight-bold" style="width: 120px;">Price (USD)</th>
                <th class="text-left font-weight-bold" style="min-width: 240px;">Color (Picker & Presets)</th>
                <th class="text-right font-weight-bold" style="width: 50px;">Action</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(z, idx) in draftZones" :key="z.id">
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
                <td class="py-2">
                  <div class="d-flex align-center gap-2">
                    <!-- Native color picker wrapper -->
                    <div
                      class="color-picker-wrapper"
                      :style="{ background: z.color || '#14b8a6' }"
                      :title="`Pick color (${z.color || '#14b8a6'})`"
                    >
                      <input
                        type="color"
                        :value="z.color || '#14b8a6'"
                        class="native-color-input"
                        @input="e => z.color = e.target.value"
                      />
                    </div>

                    <!-- Hex Code Input -->
                    <v-text-field
                      v-model="z.color"
                      placeholder="#14b8a6"
                      variant="outlined"
                      density="compact"
                      hide-details
                      class="font-weight-mono text-caption"
                      style="max-width: 95px;"
                    ></v-text-field>

                    <!-- Standard Color Preset Swatches -->
                    <div class="d-flex align-center gap-1">
                      <button
                        v-for="preset in PRESET_ZONE_COLORS.slice(0, 5)"
                        :key="preset.hex"
                        type="button"
                        class="preset-color-dot"
                        :class="{ 'preset-selected': z.color?.toLowerCase() === preset.hex.toLowerCase() }"
                        :style="{ background: preset.hex }"
                        :title="`${preset.name} (${preset.hex})`"
                        @click="z.color = preset.hex"
                      ></button>
                      <v-menu offset-y>
                        <template #activator="{ props: menuProps }">
                          <v-btn
                            v-bind="menuProps"
                            icon="mdi-dots-horizontal"
                            size="x-small"
                            variant="text"
                            color="grey"
                            title="More colors"
                          ></v-btn>
                        </template>
                        <v-card theme="dark" class="pa-2 border rounded-lg bg-surface" style="width: 170px;">
                          <div class="text-caption text-grey mb-1 font-weight-bold">Presets</div>
                          <div class="d-flex flex-wrap gap-1">
                            <button
                              v-for="preset in PRESET_ZONE_COLORS"
                              :key="preset.hex"
                              type="button"
                              class="preset-color-dot-lg"
                              :class="{ 'preset-selected': z.color?.toLowerCase() === preset.hex.toLowerCase() }"
                              :style="{ background: preset.hex }"
                              :title="`${preset.name} (${preset.hex})`"
                              @click="z.color = preset.hex"
                            ></button>
                          </div>
                        </v-card>
                      </v-menu>
                    </div>
                  </div>
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
import { ref, computed, onMounted, onUnmounted, reactive } from 'vue';
import { useRoute } from 'vue-router';
import api from '../api/client';

const route = useRoute();

const theaters = ref([]);
const selectedTheaterId = ref(null);
const loading = ref(false);
const saving = ref(false);

const originalLayout = ref({ theater: null, zones: [], seats: [] });
const currentTheaterName = computed(() => {
  const t = theaters.value.find(item => item.id === selectedTheaterId.value);
  return t ? t.name : originalLayout.value.theater?.name || '';
});
const draftZones = ref([]);
const draftSeats = ref([]);

const selectedSeatKeys = ref(new Set()); // Set of "ROW-SEATNUM" strings
const hintSlotsToCreate = ref(new Set()); // Set of "ROW-SEATNUM" empty slots currently in marquee
const slotElementMap = new Map(); // "ROW-SEATNUM" -> DOMElement (both seats and empty slots)

const bulkZoneId = ref(null);
const bulkRowLabel = ref('');

const gridColumnsCount = ref(30);
const gridRowsSpan = ref('Z');

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

// Drag state for moving selected seats or box selecting
const dragAction = reactive({
  type: 'none', // 'none' | 'pending_move' | 'move' | 'pending_marquee' | 'marquee'
  startClientX: 0,
  startClientY: 0,
  originRow: 0,
  originCol: 0,
  deltaRow: 0,
  deltaCol: 0,
  clickedSeat: null,
  clickedSlot: null,
  isShift: false,
  moveTargetSlots: new Set(),
});

// Stores range computed from last drag or selection
const inferredRange = ref({
  startRow: 'A',
  endRow: 'Z',
  startSeat: 1,
  endSeat: 30,
});

const gridDialog = ref({
  show: false,
  startRow: 'A',
  endRow: 'Z',
  startSeat: 1,
  endSeat: 30,
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

// Convert 0-based integer to row label: 0 -> A, 25 -> Z, 26 -> AA, 27 -> AB, 701 -> ZZ
function indexToRowLabel(idx) {
  let label = '';
  idx = idx + 1; // 1-based
  while (idx > 0) {
    const rem = (idx - 1) % 26;
    label = String.fromCharCode(65 + rem) + label;
    idx = Math.floor((idx - 1) / 26);
  }
  return label;
}

// Convert row label to 0-based integer: A -> 0, Z -> 25, AA -> 26, AB -> 27, ZZ -> 701
function rowLabelToIndex(label) {
  if (!label) return 0;
  label = label.toUpperCase().trim();
  let index = 0;
  for (let i = 0; i < label.length; i++) {
    const code = label.charCodeAt(i);
    if (code >= 65 && code <= 90) {
      index = index * 26 + (code - 64);
    }
  }
  return Math.max(0, index - 1);
}

function registerSlotElement(el, rowLabel, seatNumber) {
  if (el) {
    slotElementMap.set(`${rowLabel}-${seatNumber}`, el);
  }
}

// Compute maximum row index (supports A..Z, and AA..ZZ)
const gridMaxRowIndex = computed(() => {
  let maxIdx = rowLabelToIndex(gridRowsSpan.value || 'Z');
  for (const s of draftSeats.value) {
    const idx = rowLabelToIndex(s.row_label);
    if (idx > maxIdx) maxIdx = idx;
  }
  return maxIdx;
});

// Compute maximum columns (supports 1 up to 100)
const gridMaxCols = computed(() => {
  let maxNum = gridColumnsCount.value || 30;
  for (const s of draftSeats.value) {
    if (s.seat_number > maxNum) maxNum = Math.min(100, s.seat_number);
  }
  return Math.min(100, Math.max(maxNum, 14));
});

// Build 2D matrix of rows and slots (maintains fixed grid geometry without shrinking)
const gridRows = computed(() => {
  const maxIdx = gridMaxRowIndex.value;
  const rows = [];

  const seatMap = new Map();
  for (const s of draftSeats.value) {
    seatMap.set(`${s.row_label}-${s.seat_number}`, s);
  }

  for (let i = 0; i <= maxIdx; i++) {
    const rowLabel = indexToRowLabel(i);
    const slots = [];
    for (let num = 1; num <= gridMaxCols.value; num++) {
      const seat = seatMap.get(`${rowLabel}-${num}`) || null;
      slots.push({
        rowLabel,
        seatNumber: num,
        seat,
      });
    }
    rows.push({
      label: rowLabel,
      slots,
    });
  }
  return rows;
});

const isModified = computed(() => {
  const origSeats = originalLayout.value.seats;
  if (origSeats.length !== draftSeats.value.length) return true;

  const origZones = originalLayout.value.zones;
  if (origZones.length !== draftZones.value.length) return true;

  const origMap = new Map(origSeats.map(s => [`${s.row_label}-${s.seat_number}`, s.zone_id]));
  for (const s of draftSeats.value) {
    const origZone = origMap.get(`${s.row_label}-${s.seat_number}`);
    if (!origZone || origZone !== s.zone_id) return true;
  }

  return false;
});

const calculatedGridCount = computed(() => {
  const startIdx = rowLabelToIndex(gridDialog.value.startRow || 'A');
  const endIdx = rowLabelToIndex(gridDialog.value.endRow || 'Z');
  const rowCount = Math.max(0, endIdx - startIdx + 1);
  const seatCount = Math.max(0, (gridDialog.value.endSeat - gridDialog.value.startSeat + 1));
  return rowCount * seatCount;
});

const PRESET_ZONE_COLORS = [
  { name: 'Amber Gold', hex: '#f59e0b' },
  { name: 'Indigo Purple', hex: '#6366f1' },
  { name: 'Emerald Teal', hex: '#14b8a6' },
  { name: 'Rose Red', hex: '#f43f5e' },
  { name: 'Sky Cyan', hex: '#0ea5e9' },
  { name: 'Orange Coral', hex: '#f97316' },
  { name: 'Fuchsia Pink', hex: '#d946ef' },
  { name: 'Lime Green', hex: '#84cc16' },
  { name: 'Deep Violet', hex: '#8b5cf6' },
  { name: 'Slate Gray', hex: '#94a3b8' },
];

function hexToRgba(hex, alpha = 0.2) {
  if (!hex || typeof hex !== 'string') return `rgba(20, 184, 166, ${alpha})`;
  let c = hex.replace('#', '').trim();
  if (c.length === 3) c = c.split('').map(x => x + x).join('');
  if (c.length !== 6) return `rgba(20, 184, 166, ${alpha})`;
  const num = parseInt(c, 16);
  if (isNaN(num)) return `rgba(20, 184, 166, ${alpha})`;
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function getZoneName(zoneId) {
  const z = draftZones.value.find(item => item.id === zoneId);
  return z ? z.name : 'Unknown';
}

function getZonePrice(zoneId) {
  const z = draftZones.value.find(item => item.id === zoneId);
  return z ? z.price_cents : 0;
}

function getZoneColorHex(zoneIdOrName) {
  if (!zoneIdOrName) return '#14b8a6';
  // 1. Check by zone id
  const byId = draftZones.value.find(item => item.id === zoneIdOrName);
  if (byId && byId.color) return byId.color;
  // 2. Check by zone name
  const byName = draftZones.value.find(item => item.name.toLowerCase() === zoneIdOrName.toLowerCase());
  if (byName && byName.color) return byName.color;
  if (zoneIdOrName.toLowerCase().includes('vip')) return '#f59e0b';
  if (zoneIdOrName.toLowerCase().includes('premium')) return '#6366f1';
  return '#14b8a6';
}

function getZoneBgColorHex(zoneIdOrName) {
  const hex = getZoneColorHex(zoneIdOrName);
  return hexToRgba(hex, 0.18);
}

function isSeatSelected(rowLabel, seatNumber) {
  return selectedSeatKeys.value.has(`${rowLabel}-${seatNumber}`);
}

function isSlotInHint(rowLabel, seatNumber) {
  return hintSlotsToCreate.value.has(`${rowLabel}-${seatNumber}`);
}

function isSlotInMovePreview(rowLabel, seatNumber) {
  return dragAction.type === 'move' && dragAction.moveTargetSlots.has(`${rowLabel}-${seatNumber}`);
}

function hasSeatsInRow(rowLabel) {
  return draftSeats.value.some(s => s.row_label === rowLabel);
}

function isRowFullySelected(rowLabel) {
  const rowSeats = draftSeats.value.filter(s => s.row_label === rowLabel);
  if (rowSeats.length === 0) return false;
  return rowSeats.every(s => isSeatSelected(s.row_label, s.seat_number));
}

function toggleRowSelection(rowLabel) {
  const rowSeats = draftSeats.value.filter(s => s.row_label === rowLabel);

  if (rowSeats.length === 0) {
    const defaultZoneId = draftZones.value[0]?.id;
    if (defaultZoneId) {
      for (let num = 1; num <= gridMaxCols.value; num++) {
        draftSeats.value.push({
          id: null,
          zone_id: defaultZoneId,
          row_label: rowLabel,
          seat_number: num,
        });
      }
      showToast(`Created Row ${rowLabel} with ${gridMaxCols.value} seats`, 'success');
    }
    return;
  }

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
  updateInferredRangeFromSelection();
}

function handleSeatClick(seat, event) {
  const key = `${seat.row_label}-${seat.seat_number}`;
  const nextSet = new Set(selectedSeatKeys.value);

  if (event && (event.shiftKey || event.ctrlKey || event.metaKey)) {
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
  hintSlotsToCreate.value.clear();
  updateInferredRangeFromSelection();
}

function createSeatAtSlot(rowLabel, seatNumber) {
  const defaultZoneId = bulkZoneId.value || draftZones.value[0]?.id;
  if (!defaultZoneId) {
    showToast('Please create a zone first', 'warning');
    return;
  }

  draftSeats.value.push({
    id: null,
    zone_id: defaultZoneId,
    row_label: rowLabel,
    seat_number: seatNumber,
  });

  const key = `${rowLabel}-${seatNumber}`;
  const nextSet = new Set(selectedSeatKeys.value);
  nextSet.add(key);
  selectedSeatKeys.value = nextSet;
  hintSlotsToCreate.value.clear();
  showToast(`Seat ${rowLabel}${seatNumber} created`, 'success');
}

function createSeatsFromHintSlots() {
  const defaultZoneId = bulkZoneId.value || draftZones.value[0]?.id;
  if (!defaultZoneId || hintSlotsToCreate.value.size === 0) return;

  const count = hintSlotsToCreate.value.size;
  const newSelected = new Set(selectedSeatKeys.value);

  for (const key of hintSlotsToCreate.value) {
    const [row, numStr] = key.split('-');
    const num = parseInt(numStr, 10);
    const existing = draftSeats.value.find(s => s.row_label === row && s.seat_number === num);
    if (!existing) {
      draftSeats.value.push({
        id: null,
        zone_id: defaultZoneId,
        row_label: row,
        seat_number: num,
      });
    }
    newSelected.add(key);
  }

  selectedSeatKeys.value = newSelected;
  hintSlotsToCreate.value.clear();
  showToast(`Created ${count} seats in selected grid area`, 'success');
}

function selectAllSeats() {
  const nextSet = new Set();
  for (const s of draftSeats.value) {
    nextSet.add(`${s.row_label}-${s.seat_number}`);
  }
  selectedSeatKeys.value = nextSet;
  hintSlotsToCreate.value.clear();
  updateInferredRangeFromSelection();
}

function clearSelection() {
  selectedSeatKeys.value = new Set();
  hintSlotsToCreate.value.clear();
  dragAction.moveTargetSlots.clear();
}

function updateInferredRangeFromSelection() {
  if (selectedSeatKeys.value.size === 0) return;

  const rowIndices = [];
  const numbers = [];
  for (const key of selectedSeatKeys.value) {
    const parts = key.split('-');
    if (parts.length === 2) {
      rowIndices.push(rowLabelToIndex(parts[0]));
      numbers.push(parseInt(parts[1], 10));
    }
  }

  if (rowIndices.length > 0) {
    rowIndices.sort((a, b) => a - b);
    numbers.sort((a, b) => a - b);
    inferredRange.value = {
      startRow: indexToRowLabel(rowIndices[0]),
      endRow: indexToRowLabel(rowIndices[rowIndices.length - 1]),
      startSeat: numbers[0],
      endSeat: numbers[numbers.length - 1],
    };
  }
}

// Seamless Canvas Drag: Supports starting drag anywhere (empty space or over seats) and Drag-Moving seats
function handleCanvasMouseDown(e) {
  if (e.target.closest('.row-header-btn')) return;
  if (!canvasRef.value) return;

  const seatNode = e.target.closest('.seat-node');
  const emptySlot = e.target.closest('.empty-grid-slot');

  dragAction.startClientX = e.clientX;
  dragAction.startClientY = e.clientY;
  dragAction.isShift = e.shiftKey || e.ctrlKey || e.metaKey;
  dragAction.deltaRow = 0;
  dragAction.deltaCol = 0;
  dragAction.moveTargetSlots.clear();

  if (seatNode) {
    const row = seatNode.getAttribute('data-row');
    const col = parseInt(seatNode.getAttribute('data-seat'), 10);
    const key = `${row}-${col}`;
    const seatObj = draftSeats.value.find(s => s.row_label === row && s.seat_number === col);

    if (selectedSeatKeys.value.has(key)) {
      // Starting drag on an already selected seat -> prepare to move selected seats
      dragAction.type = 'pending_move';
      dragAction.originRow = rowLabelToIndex(row);
      dragAction.originCol = col;
      dragAction.clickedSeat = seatObj;
      dragAction.clickedSlot = null;
    } else {
      // Starting drag on unselected seat -> prepare to marquee drag select
      dragAction.type = 'pending_marquee';
      dragAction.originRow = rowLabelToIndex(row);
      dragAction.originCol = col;
      dragAction.clickedSeat = seatObj;
      dragAction.clickedSlot = null;
    }
  } else if (emptySlot) {
    const row = emptySlot.getAttribute('data-row');
    const col = parseInt(emptySlot.getAttribute('data-seat'), 10);
    dragAction.type = 'pending_marquee';
    dragAction.originRow = rowLabelToIndex(row);
    dragAction.originCol = col;
    dragAction.clickedSeat = null;
    dragAction.clickedSlot = { row, col };
  } else {
    // Background canvas
    dragAction.type = 'pending_marquee';
    dragAction.clickedSeat = null;
    dragAction.clickedSlot = null;
  }
}

function handleCanvasMouseMove(e) {
  if (dragAction.type === 'none' || !canvasRef.value) return;

  const dist = Math.hypot(e.clientX - dragAction.startClientX, e.clientY - dragAction.startClientY);

  // Transition from pending to active drag once past threshold (4px)
  if (dragAction.type === 'pending_move' && dist > 4) {
    dragAction.type = 'move';
  } else if (dragAction.type === 'pending_marquee' && dist > 4) {
    dragAction.type = 'marquee';
    const rect = canvasRef.value.getBoundingClientRect();
    marquee.active = true;
    marquee.startX = dragAction.startClientX - rect.left + canvasRef.value.scrollLeft;
    marquee.startY = dragAction.startClientY - rect.top + canvasRef.value.scrollTop;
    marquee.left = marquee.startX;
    marquee.top = marquee.startY;
    marquee.width = 0;
    marquee.height = 0;

    if (!dragAction.isShift) {
      selectedSeatKeys.value = new Set();
      hintSlotsToCreate.value.clear();
    }
  }

  // Handle Move Drag
  if (dragAction.type === 'move') {
    // Find hovered slot under cursor
    const hoveredEl = document.elementFromPoint(e.clientX, e.clientY);
    const targetSlot = hoveredEl ? hoveredEl.closest('.seat-node, .empty-grid-slot') : null;

    if (targetSlot) {
      const hoverRow = targetSlot.getAttribute('data-row');
      const hoverCol = parseInt(targetSlot.getAttribute('data-seat'), 10);
      if (hoverRow && hoverCol) {
        dragAction.deltaRow = rowLabelToIndex(hoverRow) - dragAction.originRow;
        dragAction.deltaCol = hoverCol - dragAction.originCol;
      }
    } else {
      // Estimate from pixel delta
      const dx = e.clientX - dragAction.startClientX;
      const dy = e.clientY - dragAction.startClientY;
      dragAction.deltaCol = Math.round(dx / 36);
      dragAction.deltaRow = Math.round(dy / 36);
    }

    // Compute preview targets
    const targetSet = new Set();
    for (const key of selectedSeatKeys.value) {
      const [r, cStr] = key.split('-');
      const targetRIdx = rowLabelToIndex(r) + dragAction.deltaRow;
      const targetCol = parseInt(cStr, 10) + dragAction.deltaCol;
      if (targetRIdx >= 0 && targetCol >= 1 && targetCol <= 100) {
        targetSet.add(`${indexToRowLabel(targetRIdx)}-${targetCol}`);
      }
    }
    dragAction.moveTargetSlots = targetSet;
    return;
  }

  // Handle Marquee Box Drag
  if (dragAction.type === 'marquee' && marquee.active) {
    const rect = canvasRef.value.getBoundingClientRect();
    const currentX = e.clientX - rect.left + canvasRef.value.scrollLeft;
    const currentY = e.clientY - rect.top + canvasRef.value.scrollTop;

    marquee.left = Math.min(marquee.startX, currentX);
    marquee.top = Math.min(marquee.startY, currentY);
    marquee.width = Math.abs(currentX - marquee.startX);
    marquee.height = Math.abs(currentY - marquee.startY);

    const boxRect = {
      left: marquee.left - canvasRef.value.scrollLeft + rect.left,
      top: marquee.top - canvasRef.value.scrollTop + rect.top,
      right: marquee.left + marquee.width - canvasRef.value.scrollLeft + rect.left,
      bottom: marquee.top + marquee.height - canvasRef.value.scrollTop + rect.top,
    };

    const nextSelected = new Set(dragAction.isShift ? selectedSeatKeys.value : []);
    const nextHints = new Set();
    const seatKeySet = new Set(draftSeats.value.map(s => `${s.row_label}-${s.seat_number}`));

    for (const [key, el] of slotElementMap.entries()) {
      if (!el) continue;
      const elRect = el.getBoundingClientRect();
      const overlaps = !(
        elRect.right < boxRect.left ||
        elRect.left > boxRect.right ||
        elRect.bottom < boxRect.top ||
        elRect.top > boxRect.bottom
      );

      if (overlaps) {
        if (seatKeySet.has(key)) {
          nextSelected.add(key);
        } else {
          nextHints.add(key);
        }
      }
    }

    selectedSeatKeys.value = nextSelected;
    hintSlotsToCreate.value = nextHints;
  }
}

function handleCanvasMouseUp() {
  if (dragAction.type === 'none') return;

  if (dragAction.type === 'pending_move' || dragAction.type === 'pending_marquee') {
    // Clicked without dragging
    if (dragAction.clickedSeat) {
      handleSeatClick(dragAction.clickedSeat, { shiftKey: dragAction.isShift });
    } else {
      // Clicked on empty space or empty slot: deselect, do NOT create seat
      if (!dragAction.isShift) clearSelection();
    }
  } else if (dragAction.type === 'move') {
    // Apply seat move translation
    if (dragAction.deltaRow !== 0 || dragAction.deltaCol !== 0) {
      const movedSeats = [];
      const newSelectedKeys = new Set();

      for (const s of draftSeats.value) {
        const key = `${s.row_label}-${s.seat_number}`;
        if (selectedSeatKeys.value.has(key)) {
          const currentRIdx = rowLabelToIndex(s.row_label);
          const targetRIdx = Math.max(0, currentRIdx + dragAction.deltaRow);
          const targetCol = Math.min(100, Math.max(1, s.seat_number + dragAction.deltaCol));
          s.row_label = indexToRowLabel(targetRIdx);
          s.seat_number = targetCol;
          newSelectedKeys.add(`${s.row_label}-${s.seat_number}`);
          movedSeats.push(s);
        }
      }

      // Deduplicate in case seats overlap
      const map = new Map();
      for (const s of draftSeats.value) {
        map.set(`${s.row_label}-${s.seat_number}`, s);
      }
      draftSeats.value = Array.from(map.values());
      selectedSeatKeys.value = newSelectedKeys;
      updateInferredRangeFromSelection();
      showToast(`Moved ${movedSeats.length} seat(s)`, 'success');
    }
  } else if (dragAction.type === 'marquee') {
    marquee.active = false;
    if (selectedSeatKeys.value.size > 0) {
      updateInferredRangeFromSelection();
    } else if (hintSlotsToCreate.value.size > 0) {
      const rowIndices = [];
      const numbers = [];
      for (const key of hintSlotsToCreate.value) {
        const parts = key.split('-');
        if (parts.length === 2) {
          rowIndices.push(rowLabelToIndex(parts[0]));
          numbers.push(parseInt(parts[1], 10));
        }
      }
      if (rowIndices.length > 0) {
        rowIndices.sort((a, b) => a - b);
        numbers.sort((a, b) => a - b);
        inferredRange.value = {
          startRow: indexToRowLabel(rowIndices[0]),
          endRow: indexToRowLabel(rowIndices[rowIndices.length - 1]),
          startSeat: numbers[0],
          endSeat: numbers[numbers.length - 1],
        };
      }
    }
  }

  dragAction.type = 'none';
  dragAction.moveTargetSlots.clear();
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
  if (count === 0) return;
  draftSeats.value = draftSeats.value.filter(
    s => !selectedSeatKeys.value.has(`${s.row_label}-${s.seat_number}`)
  );
  clearSelection();
  showToast(`Deleted ${count} seat(s) — grid walkways preserved`, 'info');
}

// Keyboard shortcuts (Delete / Backspace / Esc)
function handleGlobalKeyDown(e) {
  const activeEl = document.activeElement;
  if (activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA' || activeEl.isContentEditable)) {
    return;
  }
  if (gridDialog.value.show || singleSeatDialog.value.show || zoneDialog.value.show) {
    return;
  }

  if ((e.key === 'Delete' || e.key === 'Backspace') && selectedSeatKeys.value.size > 0) {
    e.preventDefault();
    bulkDeleteSelectedSeats();
  } else if (e.key === 'Escape') {
    clearSelection();
  }
}

// Dialogs & Generators
function openGridGeneratorDialog() {
  gridDialog.value = {
    show: true,
    startRow: inferredRange.value.startRow || 'A',
    endRow: inferredRange.value.endRow || 'Z',
    startSeat: inferredRange.value.startSeat || 1,
    endSeat: Math.min(100, inferredRange.value.endSeat || 30),
    zoneId: draftZones.value[0]?.id || null,
    mode: 'append',
  };
}

function executeGridGenerator() {
  const startIdx = rowLabelToIndex(gridDialog.value.startRow || 'A');
  const endIdx = rowLabelToIndex(gridDialog.value.endRow || 'Z');
  const startNum = Math.max(1, gridDialog.value.startSeat || 1);
  const endNum = Math.min(100, Math.max(startNum, gridDialog.value.endSeat || 30));
  const targetZoneId = gridDialog.value.zoneId;

  if (!targetZoneId) {
    showToast('Please select a zone tier', 'warning');
    return;
  }

  const newSeats = [];
  for (let i = startIdx; i <= endIdx; i++) {
    const rowLabel = indexToRowLabel(i);
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
    seatNumber: draftSeats.value.length > 0 ? Math.min(100, Math.max(...draftSeats.value.map(s => s.seat_number)) + 1) : 1,
    zoneId: draftZones.value[0]?.id || null,
  };
}

function executeAddSingleSeat() {
  const row = singleSeatDialog.value.rowLabel.trim().toUpperCase();
  const num = Math.min(100, Math.max(1, singleSeatDialog.value.seatNumber));
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
  const newId = (typeof crypto !== 'undefined' && crypto.randomUUID)
    ? crypto.randomUUID()
    : `zone_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;

  draftZones.value.push({
    id: newId,
    name: `Zone ${draftZones.value.length + 1}`,
    price_cents: 1500,
  });
}

function removeDraftZone(index) {
  if (draftZones.value.length <= 1) return;
  const removed = draftZones.value.splice(index, 1)[0];
  const fallbackZoneId = draftZones.value[0]?.id;

  for (const s of draftSeats.value) {
    if (s.zone_id === removed.id) {
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
  slotElementMap.clear();

  try {
    const res = await api.get(`/theaters/${selectedTheaterId.value}/layout`);
    originalLayout.value = res.data;
    draftZones.value = JSON.parse(JSON.stringify(res.data.zones || []));
    draftSeats.value = JSON.parse(JSON.stringify(res.data.seats || []));

    if (draftZones.value.length > 0) {
      bulkZoneId.value = draftZones.value[0].id;
    }
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
  window.addEventListener('keydown', handleGlobalKeyDown);

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

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeyDown);
});
</script>

<style scoped>
.editor-container {
  width: 100% !important;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
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
  height: 20px;
  background: linear-gradient(180deg, #f59e0b 0%, rgba(245, 158, 11, 0.1) 100%);
  transform: rotateX(-20deg);
  border-radius: 50% 50% 0 0;
  box-shadow: 0 8px 20px rgba(245, 158, 11, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
}

.screen-text {
  font-size: 9px;
  letter-spacing: 4px;
  color: #fff;
  font-weight: 900;
  opacity: 0.9;
}

.seat-canvas-container {
  position: relative;
  min-height: 480px;
  max-height: calc(100vh - 150px);
  padding: 12px;
  background: #131722;
  border-radius: 12px;
  border: 1px dashed #2a3142;
  overflow-x: auto;
  overflow-y: auto;
}

.cursor-move-active {
  cursor: move !important;
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
  min-width: max-content;
}

.row-header-placeholder {
  min-width: 32px;
}

.col-header-number {
  min-width: 32px;
  width: 32px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  opacity: 0.6;
}

.row-header-btn {
  min-width: 32px;
  height: 32px;
  padding: 0 2px;
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
  position: sticky;
  left: 0;
  z-index: 2;
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

.row-header-empty {
  opacity: 0.45;
  border-style: dashed;
}

.row-header-label-right {
  min-width: 32px;
  text-align: center;
}

.seat-node {
  position: relative;
  min-width: 32px;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1.5px solid #4b5563;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform 0.1s ease, border-color 0.15s ease, background 0.15s ease;
  flex-shrink: 0;
}

.seat-node:hover {
  transform: scale(1.12);
  z-index: 5;
}

.seat-node-selected {
  transform: scale(1.1);
  border-color: #f59e0b !important;
  border-width: 2px !important;
  box-shadow: 0 0 10px rgba(245, 158, 11, 0.6);
  z-index: 6;
  cursor: grab;
}

.seat-node-selected:active {
  cursor: grabbing;
}

.seat-node-move-target {
  border-color: #f59e0b !important;
  border-style: solid !important;
  border-width: 2px !important;
  background: rgba(245, 158, 11, 0.3) !important;
  box-shadow: 0 0 12px rgba(245, 158, 11, 0.7);
  transform: scale(1.08);
  z-index: 7;
}

.seat-node-move-preview {
  opacity: 0.6;
}

.empty-grid-slot {
  position: relative;
  min-width: 32px;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1px dashed rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.015);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.empty-grid-slot:hover {
  border-color: #f59e0b;
  background: rgba(245, 158, 11, 0.12);
}

.empty-slot-number {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.12);
  font-family: monospace;
}

.hint-slot-active {
  border-color: #f59e0b !important;
  border-style: solid !important;
  background: rgba(245, 158, 11, 0.25) !important;
  box-shadow: 0 0 8px rgba(245, 158, 11, 0.4);
  transform: scale(1.05);
  z-index: 4;
}

.hint-label {
  font-size: 8px;
  font-weight: 800;
  color: #f59e0b;
  font-family: monospace;
}

.seat-number-label {
  font-size: 10px;
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
  width: 9px;
  height: 9px;
  border-radius: 50%;
}

.color-picker-wrapper {
  position: relative;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1.5px solid rgba(255, 255, 255, 0.2);
  cursor: pointer;
  overflow: hidden;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.3);
  flex-shrink: 0;
}

.native-color-input {
  position: absolute;
  top: -10px;
  left: -10px;
  width: 50px;
  height: 50px;
  opacity: 0;
  cursor: pointer;
}

.preset-color-dot {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  border: 1px solid transparent;
  cursor: pointer;
  transition: transform 0.15s ease, border-color 0.15s ease;
  flex-shrink: 0;
}

.preset-color-dot:hover {
  transform: scale(1.25);
  border-color: #ffffff !important;
}

.preset-color-dot-lg {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  border: 1.5px solid transparent;
  cursor: pointer;
  transition: transform 0.15s ease, border-color 0.15s ease;
  flex-shrink: 0;
}

.preset-color-dot-lg:hover,
.preset-selected {
  border-color: #ffffff !important;
  box-shadow: 0 0 6px rgba(255, 255, 255, 0.8);
  transform: scale(1.15);
}

.sticky-toolbar {
  position: sticky;
  top: 70px;
}

.gap-1 {
  gap: 4px;
}

.gap-2 {
  gap: 8px;
}
</style>
