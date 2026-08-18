import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
  withCredentials: true, // Send httpOnly cookies
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  },
});

// Also support Authorization header if token is stored in localStorage as fallback
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('theater_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    return Promise.reject(error);
  }
);

export default api;
