import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import api from '../api/client';

export const useAuthStore = defineStore('auth', () => {
  const user = ref(null);
  const token = ref(localStorage.getItem('theater_token') || null);
  const loading = ref(false);
  const initialized = ref(false);

  const isAuthenticated = computed(() => !!user.value);
  const role = computed(() => user.value?.role || null);
  const isAdmin = computed(() => user.value?.role === 'admin');
  const isStaff = computed(() => ['admin', 'staff'].includes(user.value?.role));
  const isElevated = computed(() => ['admin', 'staff', 'event_organizer'].includes(user.value?.role));
  const isGatekeeper = computed(() => user.value?.role === 'gatekeeper');

  async function fetchMe() {
    try {
      loading.value = true;
      const res = await api.get('/me');
      user.value = res.data;
      return user.value;
    } catch (err) {
      user.value = null;
      token.value = null;
      localStorage.removeItem('theater_token');
      return null;
    } finally {
      loading.value = false;
      initialized.value = true;
    }
  }

  async function login(email, password) {
    loading.value = true;
    try {
      const res = await api.post('/auth/login', { email, password });
      user.value = res.data.user;
      token.value = res.data.token;
      localStorage.setItem('theater_token', res.data.token);
      return { success: true, user: user.value };
    } catch (err) {
      const errorMsg = err.response?.data?.error || 'Login failed';
      return { success: false, error: errorMsg };
    } finally {
      loading.value = false;
    }
  }

  async function register(email, password, role = 'customer') {
    loading.value = true;
    try {
      const res = await api.post('/auth/register', { email, password, role });
      user.value = res.data.user;
      token.value = res.data.token;
      localStorage.setItem('theater_token', res.data.token);
      return { success: true, user: user.value };
    } catch (err) {
      const errorMsg = err.response?.data?.error || 'Registration failed';
      return { success: false, error: errorMsg };
    } finally {
      loading.value = false;
    }
  }

  async function logout() {
    try {
      await api.post('/auth/logout');
    } catch (e) {
      // Ignore
    } finally {
      user.value = null;
      token.value = null;
      localStorage.removeItem('theater_token');
    }
  }

  return {
    user,
    token,
    loading,
    initialized,
    isAuthenticated,
    role,
    isAdmin,
    isStaff,
    isElevated,
    isGatekeeper,
    fetchMe,
    login,
    register,
    logout,
  };
});
