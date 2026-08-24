<template>
  <div class="event-sidebar-container d-flex flex-column gap-3">
    <!-- Quick Actions Card -->
    <v-card theme="dark" class="rounded-xl border bg-surface pa-3" elevation="2">
      <div class="d-flex align-center justify-space-between mb-2 px-1">
        <span class="text-caption font-weight-bold text-grey text-uppercase tracking-wider">
          Quick Actions
        </span>
        <v-icon icon="mdi-flash" color="amber-darken-1" size="16"></v-icon>
      </div>

      <div class="d-flex flex-column gap-2">
        <!-- Add Showtime Button -->
        <v-btn
          color="amber-darken-1"
          variant="flat"
          block
          size="small"
          prepend-icon="mdi-calendar-plus"
          class="text-none font-weight-bold action-btn"
          @click="$emit('add-showtime')"
        >
          Add Showtime
        </v-btn>

        <!-- Seat Map Editor Button -->
        <v-btn
          color="indigo-darken-1"
          variant="flat"
          block
          size="small"
          prepend-icon="mdi-seat-passenger"
          class="text-none font-weight-bold action-btn"
          @click="$emit('open-seat-map')"
        >
          Seat Map Editor
        </v-btn>
      </div>
    </v-card>

    <!-- Navigation Menu Card -->
    <v-card theme="dark" class="rounded-xl border bg-surface pa-2" elevation="2">
      <div class="d-flex align-center justify-space-between mb-1 px-2 pt-1">
        <span class="text-caption font-weight-bold text-grey text-uppercase tracking-wider">
          Event Navigation
        </span>
        <v-icon icon="mdi-tune-vertical" color="grey" size="16"></v-icon>
      </div>

      <v-list density="compact" class="bg-transparent pa-0 nav-list">
        <!-- Event Overview & Analytics -->
        <v-list-item
          :to="event ? `/manage/events/${event.id}` : '#'"
          :active="activeItem === 'overview'"
          rounded="lg"
          class="mb-1 nav-item"
          active-class="nav-item-active"
        >
          <template #prepend>
            <v-icon icon="mdi-view-dashboard-outline" color="amber-lighten-2" size="18" class="mr-2"></v-icon>
          </template>
          <v-list-item-title class="font-weight-bold text-body-2">
            Overview & Analytics
          </v-list-item-title>
          <v-list-item-subtitle class="text-caption text-grey">
            Sessions & revenue KPIs
          </v-list-item-subtitle>
        </v-list-item>

        <!-- Gatekeepers (Scanner Staff) -->
        <v-list-item
          :to="event ? `/manage/events/${event.id}/gatekeepers` : '#'"
          :active="activeItem === 'gatekeepers'"
          rounded="lg"
          class="mb-1 nav-item"
          active-class="nav-item-active"
        >
          <template #prepend>
            <v-icon icon="mdi-shield-account" color="teal-accent-3" size="18" class="mr-2"></v-icon>
          </template>
          <v-list-item-title class="font-weight-bold text-body-2">
            Gatekeepers
          </v-list-item-title>
          <v-list-item-subtitle class="text-caption text-grey">
            Scanner staff & access
          </v-list-item-subtitle>
        </v-list-item>

        <!-- Financial & Tax Report -->
        <v-list-item
          :to="event ? `/manage/events/${event.id}/financial-reports` : '#'"
          :active="activeItem === 'financial-report'"
          rounded="lg"
          class="mb-1 nav-item"
          active-class="nav-item-active"
        >
          <template #prepend>
            <v-icon icon="mdi-file-document-outline" color="amber-darken-1" size="18" class="mr-2"></v-icon>
          </template>
          <v-list-item-title class="font-weight-bold text-body-2">
            Financial & Tax Reports
          </v-list-item-title>
          <v-list-item-subtitle class="text-caption text-grey">
            Statements & tax filings
          </v-list-item-subtitle>
        </v-list-item>

        <!-- Settings & Tax Info -->
        <v-list-item
          :to="event ? `/manage/events/${event.id}/settings` : '#'"
          :active="activeItem === 'settings'"
          rounded="lg"
          class="mb-1 nav-item"
          active-class="nav-item-active"
        >
          <template #prepend>
            <v-icon icon="mdi-cog-outline" color="blue-grey-lighten-2" size="18" class="mr-2"></v-icon>
          </template>
          <v-list-item-title class="font-weight-bold text-body-2">
            Settings & Tax Info
          </v-list-item-title>
          <v-list-item-subtitle class="text-caption text-grey">
            Limits & organizer profile
          </v-list-item-subtitle>
        </v-list-item>

        <!-- Manage Orders -->
        <v-list-item
          :to="event ? `/orders?event_id=${event.id}` : '/orders'"
          :active="activeItem === 'orders'"
          rounded="lg"
          class="mb-1 nav-item"
          active-class="nav-item-active"
        >
          <template #prepend>
            <v-icon icon="mdi-ticket-confirmation" color="amber-lighten-2" size="18" class="mr-2"></v-icon>
          </template>
          <v-list-item-title class="font-weight-bold text-body-2">
            Manage Orders
          </v-list-item-title>
          <v-list-item-subtitle class="text-caption text-grey">
            Orders & customer passes
          </v-list-item-subtitle>
        </v-list-item>
      </v-list>
    </v-card>

    <!-- Danger Zone Card -->
    <v-card theme="dark" class="rounded-xl border border-red-darken-4 bg-surface pa-3" elevation="2">
      <div class="d-flex align-center justify-space-between mb-2 px-1">
        <span class="text-caption font-weight-bold text-red-lighten-2 text-uppercase tracking-wider">
          Danger Zone
        </span>
        <v-icon icon="mdi-alert-outline" color="red-lighten-2" size="16"></v-icon>
      </div>

      <v-btn
        variant="outlined"
        color="red-lighten-2"
        block
        size="small"
        prepend-icon="mdi-delete-outline"
        class="text-none font-weight-bold delete-btn"
        @click="$emit('delete-event')"
      >
        Delete Event
      </v-btn>
    </v-card>
  </div>
</template>

<script setup>
defineProps({
  event: {
    type: Object,
    default: null,
  },
  activeItem: {
    type: String,
    default: 'overview',
  },
});

defineEmits(['add-showtime', 'open-seat-map', 'delete-event']);
</script>

<style scoped>
.event-sidebar-container {
  width: 100%;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.border-red-darken-4 {
  border-color: rgba(239, 68, 68, 0.25) !important;
}

.tracking-wider {
  letter-spacing: 0.08em;
  font-size: 0.7rem !important;
}

.gap-2 {
  gap: 8px;
}

.gap-3 {
  gap: 12px;
}

.action-btn {
  transition: transform 0.15s ease, box-shadow 0.15s ease;
  height: 36px !important;
}

.action-btn:hover {
  transform: translateY(-1px);
}

.nav-list {
  background: transparent !important;
}

.nav-item {
  border: 1px solid transparent;
  transition: all 0.2s ease;
  padding: 6px 10px !important;
  min-height: 42px !important;
}

.nav-item:hover {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.nav-item-active {
  background: rgba(245, 158, 11, 0.12) !important;
  border: 1px solid rgba(245, 158, 11, 0.35) !important;
}

.nav-item-active :deep(.v-list-item-title) {
  color: #f59e0b !important;
}

.delete-btn {
  transition: all 0.2s ease;
  height: 34px !important;
}

.delete-btn:hover {
  background: rgba(239, 68, 68, 0.1) !important;
}
</style>
