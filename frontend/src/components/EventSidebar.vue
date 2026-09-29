<template>
  <div class="bootstrap-sidebar">
    <!-- Brand Header: Dream Cinema Logo on top of Event Title -->
    <div class="sidebar-header pb-2">
      <!-- Dream Cinema Logo -->
      <!-- <div class="sidebar-brand-wrapper mb-2">
        <router-link to="/" class="sidebar-brand-link d-inline-flex align-center">
          <v-icon icon="mdi-theater" color="amber-darken-1" size="22" class="mr-2"></v-icon>
          <span class="brand-text text-white font-weight-bold">
            DREAM<span class="text-amber-darken-1 font-weight-light">CINEMA</span>
          </span>
        </router-link>
      </div> -->

      <!-- Event Title in Full (No ellipsis, wraps naturally) -->
      <div class="event-title text-white font-weight-bold">
        {{ event?.title || 'Manage Event' }}
      </div>
    </div>

    <!-- Divider -->
    <hr class="sidebar-divider" />

    <!-- Navigation List -->
    <ul class="sidebar-nav">
      <!-- Overview & Analytics -->
      <li class="sidebar-nav-item">
        <router-link :to="event ? `/manage/events/${event.id}` : '#'" class="sidebar-nav-link"
          :class="{ active: activeItem === 'overview' }">
          <v-icon icon="mdi-home-outline" size="18" class="sidebar-icon"></v-icon>
          <span class="sidebar-label">Overview & Analytics</span>
        </router-link>
      </li>

      <!-- Scheduled Showtimes -->
      <li class="sidebar-nav-item">
        <router-link :to="event ? `/manage/events/${event.id}/showtimes` : '#'" class="sidebar-nav-link"
          :class="{ active: activeItem === 'showtimes' }">
          <v-icon icon="mdi-calendar-clock" size="18" class="sidebar-icon"></v-icon>
          <span class="sidebar-label">Scheduled Showtimes</span>
        </router-link>
      </li>

      <!-- Seat Map Editor -->
      <li class="sidebar-nav-item">
        <button type="button" class="sidebar-nav-link" :class="{ active: activeItem === 'seat-map' }"
          @click="handleSeatMapClick">
          <v-icon icon="mdi-seat-passenger" size="18" class="sidebar-icon"></v-icon>
          <span class="sidebar-label">Seat Map Editor</span>
        </button>
      </li>

      <!-- Orders -->
      <li class="sidebar-nav-item">
        <router-link :to="event ? `/orders?event_id=${event.id}` : '/orders'" class="sidebar-nav-link"
          :class="{ active: activeItem === 'orders' }">
          <v-icon icon="mdi-table-large" size="18" class="sidebar-icon"></v-icon>
          <span class="sidebar-label">Orders</span>
        </router-link>
      </li>

      <!-- Gatekeepers -->
      <li class="sidebar-nav-item">
        <router-link :to="event ? `/manage/events/${event.id}/gatekeepers` : '#'" class="sidebar-nav-link"
          :class="{ active: activeItem === 'gatekeepers' }">
          <v-icon icon="mdi-shield-account-outline" size="18" class="sidebar-icon"></v-icon>
          <span class="sidebar-label">Gatekeepers</span>
        </router-link>
      </li>

      <!-- Financial Reports -->
      <li class="sidebar-nav-item">
        <router-link :to="event ? `/manage/events/${event.id}/financial-reports` : '#'" class="sidebar-nav-link"
          :class="{ active: activeItem === 'financial-report' }">
          <v-icon icon="mdi-file-document-outline" size="18" class="sidebar-icon"></v-icon>
          <span class="sidebar-label">Financial Reports</span>
        </router-link>
      </li>

      <!-- Settings & Tax Info -->
      <li class="sidebar-nav-item">
        <router-link :to="event ? `/manage/events/${event.id}/settings` : '#'" class="sidebar-nav-link"
          :class="{ active: activeItem === 'settings' }">
          <v-icon icon="mdi-cog-outline" size="18" class="sidebar-icon"></v-icon>
          <span class="sidebar-label">Settings</span>
        </router-link>
      </li>
    </ul>

    <!-- Bottom Footer: Back to Events / Details Navigation Link -->
    <div class="sidebar-footer mt-auto pt-3">
      <hr class="sidebar-divider mb-3" />

      <!-- Subpage context: Back to Event Details -->
      <router-link v-if="activeItem !== 'overview'" :to="event ? `/manage/events/${event.id}` : '/manage/events'"
        class="sidebar-nav-link back-link mb-1">
        <v-icon icon="mdi-arrow-left" size="18" class="sidebar-icon"></v-icon>
        <span class="sidebar-label">Back to Event Details</span>
      </router-link>

      <!-- Back to all events -->
      <router-link to="/manage/events" class="sidebar-nav-link back-link"
        :class="{ 'opacity-75': activeItem !== 'overview' }">
        <v-icon :icon="activeItem === 'overview' ? 'mdi-arrow-left' : 'mdi-view-grid-outline'" size="18"
          class="sidebar-icon"></v-icon>
        <span class="sidebar-label">{{ activeItem === 'overview' ? 'Back to Events' : 'Back to All Events' }}</span>
      </router-link>
    </div>
  </div>
</template>

<script setup>
import { useAttrs } from 'vue';
import { useRouter } from 'vue-router';

const props = defineProps({
  event: {
    type: Object,
    default: null,
  },
  activeItem: {
    type: String,
    default: 'overview',
  },
});

const emit = defineEmits(['open-seat-map']);
const attrs = useAttrs();
const router = useRouter();

function handleSeatMapClick() {
  if (attrs.onOpenSeatMap || attrs['onOpen-seat-map']) {
    emit('open-seat-map');
  } else {
    router.push('/manage/seat-map');
  }
}
</script>

<style scoped>
.bootstrap-sidebar {
  width: 100%;
  background-color: #212529;
  /* Exact Bootstrap 5 bg-dark */
  border-right: 1px solid rgba(255, 255, 255, 0.15);
  border-top: none;
  border-left: none;
  border-bottom: none;
  border-radius: 0;
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  box-sizing: border-box;
}

@media (min-width: 960px) {
  .bootstrap-sidebar {
    width: 280px;
    min-width: 280px;
    max-width: 280px;
    height: calc(100vh - 56px);
    position: sticky;
    top: 56px;
    z-index: 10;
  }
}

.sidebar-brand-wrapper {
  display: flex;
  align-items: center;
}

.sidebar-brand-link {
  text-decoration: none;
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.sidebar-brand-link:hover {
  opacity: 0.85;
}

.brand-text {
  font-size: 1.05rem;
  letter-spacing: -0.01em;
}

.event-title {
  font-size: 0.95rem;
  font-weight: 600;
  line-height: 1.4;
  word-break: break-word;
  white-space: normal;
  color: #f8fafc;
}

.sidebar-divider {
  border: 0;
  border-top: 1px solid rgba(255, 255, 255, 0.15);
  margin: 12px 0 16px 0;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0;
  margin: 0;
  list-style: none;
  overflow-y: auto;
}

.sidebar-nav-item {
  width: 100%;
}

.sidebar-nav-link {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.95rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.85);
  text-decoration: none;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: color 0.15s ease-in-out, background-color 0.15s ease-in-out;
  text-align: left;
}

.sidebar-nav-link:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: #ffffff;
}

.sidebar-nav-link.active {
  background-color: #0d6efd !important;
  /* Bootstrap primary blue */
  color: #ffffff !important;
  font-weight: 600;
}

.sidebar-icon {
  margin-right: 12px;
  flex-shrink: 0;
  color: inherit !important;
}

.sidebar-label {
  color: inherit;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar-footer {
  flex-shrink: 0;
}

.back-link {
  color: rgba(255, 255, 255, 0.7);
}

.back-link:hover {
  color: #ffffff;
  background-color: rgba(255, 255, 255, 0.08);
}

.opacity-75 {
  opacity: 0.75;
}
</style>
