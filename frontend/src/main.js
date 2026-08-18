import { createApp } from 'vue';
import { createPinia } from 'pinia';
import router from './router';
import App from './App.vue';

// Vuetify 3
import 'vuetify/styles';
import '@mdi/font/css/materialdesignicons.css';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

import './style.css';

const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'dark',
    themes: {
      dark: {
        dark: true,
        colors: {
          background: '#0d111a',
          surface: '#131722',
          primary: '#f59e0b',
          secondary: '#6366f1',
          accent: '#10b981',
          error: '#ef4444',
          info: '#3b82f6',
          success: '#10b981',
          warning: '#f59e0b',
        },
      },
    },
  },
});

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(vuetify);

app.mount('#app');
