<template>
  <v-container :fluid="authStore.isElevated" class="py-6 px-4 px-md-6 px-lg-8">
    <!-- Elevated Management Layout (Admin / Staff / Event Organizer) -->
    <v-row v-if="authStore.isElevated && (activeEvent || events.length > 0)" class="mt-1">
      <!-- Sidebar Column (Left) -->
      <v-col cols="12" md="3" xl="2">
        <div class="sticky-sidebar d-flex flex-column gap-3">
          <!-- Event Selector Card if multiple events exist -->
          <v-card v-if="events.length > 1" theme="dark" class="rounded-xl border bg-surface pa-3" elevation="2">
            <div class="d-flex align-center justify-space-between mb-2 px-1">
              <span class="text-caption font-weight-bold text-grey text-uppercase tracking-wider">
                Event Filter
              </span>
              <v-icon icon="mdi-filter-variant" color="amber-darken-1" size="16"></v-icon>
            </div>
            <v-select v-model="selectedEventId" :items="eventSelectItems" item-title="label" item-value="id"
              variant="outlined" density="compact" hide-details class="event-selector"
              @update:model-value="onSelectEvent"></v-select>
          </v-card>

          <!-- Main Event Sidebar -->
          <EventSidebar v-if="activeEvent" :event="activeEvent" active-item="orders" @add-showtime="goToEventDetails"
            @open-seat-map="goToEventDetails" @delete-event="goToEventDetails" />
        </div>
      </v-col>

      <!-- Main Content Column (Right) -->
      <v-col cols="12" md="9" xl="10">
        <OrdersTable :scope="authStore.isElevated ? 'all' : 'own'" :event-id="activeEvent?.id"
          :event-title="activeEvent?.title" />
      </v-col>
    </v-row>

    <!-- Standard View (Customer or elevated without events) -->
    <div v-else>
      <OrdersTable :scope="authStore.isElevated ? 'all' : 'own'" />
    </div>
  </v-container>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import api from '../api/client';
import OrdersTable from '../components/OrdersTable.vue';
import EventSidebar from '../components/EventSidebar.vue';

const props = defineProps({
  id: {
    type: String,
    default: null,
  },
});

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const events = ref([]);
const selectedEventId = ref(null);
const loadingEvents = ref(false);

const activeEvent = computed(() => {
  if (!selectedEventId.value) return null;
  return events.value.find((e) => e.id === selectedEventId.value) || null;
});

const eventSelectItems = computed(() => {
  const items = events.value.map((e) => ({
    id: e.id,
    label: e.title,
  }));
  if (authStore.isAdmin || authStore.isStaff) {
    items.unshift({ id: 'all', label: '— All System Events —' });
  }
  return items;
});

function goToEventDetails() {
  if (activeEvent.value) {
    router.push(`/manage/events/${activeEvent.value.id}`);
  }
}

function onSelectEvent(newVal) {
  if (!newVal || newVal === 'all') {
    selectedEventId.value = null;
    router.replace({ path: '/orders' });
  } else {
    selectedEventId.value = newVal;
    router.replace({ path: '/orders', query: { event_id: newVal } });
  }
}

async function loadEvents() {
  if (!authStore.isElevated) return;

  loadingEvents.value = true;
  try {
    const res = await api.get('/events');
    events.value = res.data || [];

    const queryEventId = props.id || route.query.event_id;
    if (queryEventId && events.value.some((e) => e.id === queryEventId)) {
      selectedEventId.value = queryEventId;
    } else if (events.value.length > 0) {
      selectedEventId.value = events.value[0].id;
    }
  } catch (err) {
    console.error('Failed to load events for orders view', err);
  } finally {
    loadingEvents.value = false;
  }
}

watch(
  () => route.query.event_id,
  (newVal) => {
    if (newVal && events.value.some((e) => e.id === newVal)) {
      selectedEventId.value = newVal;
    } else if (!newVal && authStore.isAdmin) {
      selectedEventId.value = null;
    }
  }
);

onMounted(() => {
  loadEvents();
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

.tracking-wider {
  letter-spacing: 0.08em;
  font-size: 0.7rem !important;
}

.gap-3 {
  gap: 12px;
}
</style>
