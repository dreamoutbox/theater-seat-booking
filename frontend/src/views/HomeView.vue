<template>
  <v-container class="py-8">
    <!-- Hero Banner -->
    <div class="hero-section text-center mb-10 pa-8 rounded-xl">
      <v-chip color="amber-darken-1" size="small" variant="flat" class="mb-3 font-weight-bold">
        EXPERIENCE PREMIUM CINEMA
      </v-chip>
      <h1 class="text-h3 font-weight-black text-white mb-3 tracking-tight">
        Live Symphony & Cinematic Masterpieces
      </h1>
      <p class="text-body-1 text-grey max-w-2xl mx-auto mb-6">
        Select a showtime below to reserve your seats with real-time zone pricing, instant hold guarantees, and seamless checkout.
      </p>
    </div>

    <!-- Showtimes Catalog -->
    <div class="d-flex justify-space-between align-center mb-6">
      <div>
        <h2 class="text-h5 font-weight-bold text-white">Upcoming Showtimes</h2>
        <span class="text-caption text-grey">All sessions currently open for reservation</span>
      </div>
      <v-btn
        prepend-icon="mdi-refresh"
        variant="text"
        size="small"
        :loading="loading"
        @click="fetchShowtimes"
      >
        Refresh
      </v-btn>
    </div>

    <!-- Loading Skeletons -->
    <v-row v-if="loading">
      <v-col v-for="n in 4" :key="n" cols="12" sm="6" md="6">
        <v-skeleton-loader type="card" theme="dark" class="rounded-xl"></v-skeleton-loader>
      </v-col>
    </v-row>

    <!-- Showtimes Grid -->
    <v-row v-else>
      <v-col
        v-for="show in showtimes"
        :key="show.id"
        cols="12"
        sm="6"
        md="6"
      >
        <v-card
          class="showtime-card rounded-xl pa-5 h-100 d-flex flex-column justify-space-between border"
          elevation="4"
        >
          <div>
            <div class="d-flex justify-space-between align-start mb-2">
              <v-chip size="small" color="amber-darken-1" variant="tonal" class="font-weight-bold">
                {{ show.theater_name }}
              </v-chip>
              <span class="text-caption text-grey font-weight-mono">
                <v-icon icon="mdi-calendar-clock" size="14" class="mr-1"></v-icon>
                {{ formatDate(show.starts_at) }}
              </span>
            </div>

            <h3 class="text-h6 font-weight-bold text-white mb-2">
              {{ show.event_title }}
            </h3>

            <div class="d-flex align-center text-caption text-grey mb-4">
              <v-icon icon="mdi-map-marker" size="14" class="mr-1 text-grey"></v-icon>
              <span>Grand Horizon Hall • Dolby Atmos Surround</span>
            </div>

            <div class="d-flex gap-2 mb-4">
              <v-chip size="x-small" variant="outlined" color="amber-lighten-2">VIP $25</v-chip>
              <v-chip size="x-small" variant="outlined" color="indigo-lighten-2">Premium $18</v-chip>
              <v-chip size="x-small" variant="outlined" color="teal-lighten-2">Standard $12</v-chip>
            </div>
          </div>

          <v-btn
            color="amber-darken-1"
            variant="flat"
            block
            prepend-icon="mdi-seat"
            class="text-none font-weight-bold mt-2"
            :to="`/booking/${show.id}`"
          >
            Select Seats & Book
          </v-btn>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import api from '../api/client';

const showtimes = ref([]);
const loading = ref(false);

function formatDate(isoStr) {
  if (!isoStr) return '—';
  const d = new Date(isoStr);
  return d.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
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

.showtime-card {
  background: #181c27;
  border-color: #2a3142;
  transition: transform 0.2s ease, border-color 0.2s ease;
}

.showtime-card:hover {
  transform: translateY(-4px);
  border-color: #f59e0b;
}

.max-w-2xl {
  max-width: 650px;
}

.gap-2 {
  gap: 8px;
}
</style>
