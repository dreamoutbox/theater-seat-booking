<template>
  <v-container class="py-8">
    <!-- Hero Banner -->
    <!-- <div class="hero-section text-center mb-10 pa-8 rounded-xl">
      <v-chip color="amber-darken-1" size="small" variant="flat" class="mb-3 font-weight-bold">
        EXPERIENCE PREMIUM CINEMA
      </v-chip>
      <h1 class="text-h3 font-weight-black text-white mb-3 tracking-tight">
        Live Symphony & Cinematic Masterpieces
      </h1>
      <p class="text-body-1 text-grey max-w-2xl mx-auto mb-6">
        Select a showtime below to reserve your seats with real-time zone pricing, instant hold guarantees, and seamless
        checkout.
      </p>
    </div> -->

    <!-- Showtimes Catalog -->
    <div class="d-flex justify-space-between align-center mb-6">
      <div>
        <h2 class="text-h5 font-weight-bold text-white">Upcoming Events & Showtimes</h2>
        <span class="text-caption text-grey">Browse movies and concerts with available booking sessions</span>
      </div>
      <v-btn prepend-icon="mdi-refresh" variant="text" size="small" :loading="loading" @click="fetchShowtimes">
        Refresh
      </v-btn>
    </div>

    <!-- Loading Skeletons -->
    <v-row v-if="loading">
      <v-col v-for="n in 2" :key="n" cols="12" md="6">
        <v-skeleton-loader type="card, list-item-two-line, actions" theme="dark" class="rounded-xl"></v-skeleton-loader>
      </v-col>
    </v-row>

    <!-- Grouped Events Grid -->
    <v-row v-else>
      <v-col v-for="event in groupedEvents" :key="event.event_id" cols="12" md="6">
        <v-card class="event-card rounded-xl pa-6 h-100 d-flex flex-column justify-space-between border" elevation="4">
          <div>
            <!-- Header Badges -->
            <div class="d-flex justify-space-between align-center mb-3">
              <v-chip size="small" color="amber-darken-1" variant="tonal" class="font-weight-bold">
                <v-icon start icon="mdi-theater" size="14"></v-icon>
                {{ event.theater_name }}
              </v-chip>
              <span class="text-caption text-grey font-weight-mono font-weight-bold">
                {{ event.showtimes.length }} {{ event.showtimes.length === 1 ? 'Showtime' : 'Showtimes' }}
              </span>
            </div>

            <!-- Event Title -->
            <h3 class="text-h5 font-weight-bold text-white mb-2">
              {{ event.event_title }}
            </h3>

            <!-- Venue Description -->
            <div class="d-flex align-center text-caption text-grey mb-4">
              <v-icon icon="mdi-map-marker" size="14" class="mr-1 text-grey"></v-icon>
              <span>Main Hall • Dolby Atmos • Laser Projection</span>
            </div>

            <!-- Pricing Tiers -->
            <div class="d-flex flex-wrap gap-2 mb-5">
              <v-chip size="x-small" variant="outlined" color="amber-lighten-2" class="font-weight-bold">
                VIP $25.00
              </v-chip>
              <v-chip size="x-small" variant="outlined" color="indigo-lighten-2" class="font-weight-bold">
                Premium $18.00
              </v-chip>
              <v-chip size="x-small" variant="outlined" color="teal-lighten-2" class="font-weight-bold">
                Standard $12.00
              </v-chip>
            </div>

            <v-divider class="mb-4"></v-divider>

            <!-- Showtimes in this Event -->
            <div class="mb-2">
              <div class="text-caption text-grey font-weight-bold text-uppercase mb-3">
                SELECT A SHOWTIME SESSION
              </div>
              <div class="d-flex flex-column gap-2">
                <div v-for="show in event.showtimes" :key="show.id"
                  class="showtime-session-item d-flex justify-space-between align-center pa-3 rounded-lg bg-surface-variant border">
                  <div class="d-flex align-center gap-2">
                    <v-icon icon="mdi-calendar-clock" color="amber-darken-1" size="18"></v-icon>
                    <div>
                      <div class="text-body-2 font-weight-bold text-white">
                        {{ formatDate(show.starts_at) }}
                      </div>
                      <div class="text-caption text-grey">
                        {{ getTimeRelative(show.starts_at) }}
                      </div>
                    </div>
                  </div>

                  <v-btn color="amber-darken-1" variant="flat" size="small" prepend-icon="mdi-seat"
                    class="text-none font-weight-bold px-4" :to="`/booking/${show.id}`">
                    Select Seats
                  </v-btn>
                </div>
              </div>
            </div>
          </div>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import api from '../api/client';

const showtimes = ref([]);
const loading = ref(false);

const groupedEvents = computed(() => {
  const map = new Map();
  for (const s of showtimes.value) {
    if (!map.has(s.event_id)) {
      map.set(s.event_id, {
        event_id: s.event_id,
        event_title: s.event_title,
        theater_name: s.theater_name,
        theater_id: s.theater_id,
        showtimes: [],
      });
    }
    map.get(s.event_id).showtimes.push(s);
  }

  // Sort showtimes in each event by start date ASC
  for (const event of map.values()) {
    event.showtimes.sort((a, b) => new Date(a.starts_at) - new Date(b.starts_at));
  }

  return Array.from(map.values());
});

function formatDate(isoStr) {
  if (!isoStr) return '—';
  const d = new Date(isoStr);
  return d.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  });
}

function getTimeRelative(isoStr) {
  if (!isoStr) return '';
  const d = new Date(isoStr);
  const now = new Date();
  const diffHours = Math.round((d.getTime() - now.getTime()) / (1000 * 60 * 60));
  if (diffHours <= 0) return 'Starting soon';
  if (diffHours < 24) return `In ~${diffHours} hours`;
  const diffDays = Math.round(diffHours / 24);
  return `In ${diffDays} day${diffDays > 1 ? 's' : ''}`;
}

async function fetchShowtimes() {
  try {
    loading.value = true;
    const res = await api.get('/showtimes');
    showtimes.value = res.data;
  } catch (err) {
    console.error('Failed to load showtimes', err);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  fetchShowtimes();
});
</script>

<style scoped>
.hero-section {
  background: radial-gradient(circle at center, #1f2536 0%, #131722 100%);
  border: 1px solid #2a3142;
}

.event-card {
  background: #181c27;
  border-color: #2a3142;
  transition: border-color 0.2s ease;
}

.event-card:hover {
  border-color: #f59e0b;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.showtime-session-item {
  transition: background-color 0.15s ease;
}

.showtime-session-item:hover {
  background: #252b3b !important;
}

.max-w-2xl {
  max-width: 650px;
}

.gap-2 {
  gap: 8px;
}
</style>
