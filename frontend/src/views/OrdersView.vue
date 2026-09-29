<template>
  <!-- Elevated Management Layout (Admin / Staff / Event Organizer) -->
  <div v-if="authStore.isElevated && (activeEvent || events.length > 0)" class="manage-layout-body">
    <!-- Sidebar Column (Flush Left) -->
    <EventSidebar v-if="activeEvent" :event="activeEvent" active-item="orders" />

    <!-- Main Content Column (Right) -->
    <main class="manage-main-content">
      <OrdersTable :scope="authStore.isElevated ? 'all' : 'own'" :event-id="activeEvent?.id"
        :event-title="activeEvent?.title" />
    </main>
  </div>

  <!-- Standard View (Customer or elevated without events) -->
  <v-container v-else class="py-8 max-w-5xl">
    <OrdersTable :scope="authStore.isElevated ? 'all' : 'own'" />
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
  padding: 0px 0px 1px;
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
