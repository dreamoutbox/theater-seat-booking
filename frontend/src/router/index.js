import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '../views/HomeView.vue';
import BookingView from '../views/BookingView.vue';
import OrdersView from '../views/OrdersView.vue';
import OrderDetailView from '../views/OrderDetailView.vue';
import ManageEventsView from '../views/ManageEventsView.vue';
import ManageEventDetailView from '../views/ManageEventDetailView.vue';
import ManageEventGatekeepersView from '../views/ManageEventGatekeepersView.vue';
import EventFinancialReportsListView from '../views/EventFinancialReportsListView.vue';
import EventFinancialReportView from '../views/EventFinancialReportView.vue';
import ManageEventSettingsView from '../views/ManageEventSettingsView.vue';
import CheckoutView from '../views/CheckoutView.vue';
import ManageTheatersView from '../views/ManageTheatersView.vue';
import SeatMapEditorView from '../views/SeatMapEditorView.vue';
import ScanTicketView from '../views/ScanTicketView.vue';
import DashboardView from '../views/DashboardView.vue';
import LoginView from '../views/LoginView.vue';
import RegisterView from '../views/RegisterView.vue';
import { useAuthStore } from '../stores/auth';

const routes = [
  {
    path: '/',
    name: 'Home',
    component: HomeView,
  },
  {
    path: '/booking/:showtimeId',
    alias: '/showtimes/:showtimeId/booking',
    name: 'Booking',
    component: BookingView,
    props: true,
  },
  {
    path: '/checkout/:showtimeId',
    name: 'Checkout',
    component: CheckoutView,
    props: true,
    meta: { requiresAuth: true },
  },
  {
    path: '/orders',
    name: 'Orders',
    component: OrdersView,
    meta: { requiresAuth: true },
  },
  {
    path: '/orders/:id',
    name: 'OrderDetail',
    component: OrderDetailView,
    props: true,
    meta: { requiresAuth: true },
  },
  {
    path: '/scan',
    name: 'ScanTickets',
    component: ScanTicketView,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer', 'gatekeeper'] },
  },
  {
    path: '/manage/dashboard',
    name: 'Dashboard',
    component: DashboardView,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/theaters',
    name: 'ManageTheaters',
    component: ManageTheatersView,
    meta: { requiresAuth: true, roles: ['admin', 'staff'] },
  },
  {
    path: '/manage/events',
    name: 'ManageEvents',
    component: ManageEventsView,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/events/:id',
    name: 'ManageEventDetail',
    component: ManageEventDetailView,
    props: true,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/events/:id/gatekeepers',
    name: 'ManageEventGatekeepers',
    component: ManageEventGatekeepersView,
    props: true,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/events/:id/financial-reports',
    alias: '/manage/events/:id/financial-report',
    name: 'EventFinancialReportsList',
    component: EventFinancialReportsListView,
    props: true,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/events/:id/financial-reports/:reportId',
    alias: '/manage/events/:id/financial-report/:reportId',
    name: 'EventFinancialReportDetail',
    component: EventFinancialReportView,
    props: true,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/events/:id/settings',
    name: 'ManageEventSettings',
    component: ManageEventSettingsView,
    props: true,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/events/:id/orders',
    name: 'ManageEventOrders',
    component: OrdersView,
    props: true,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/seat-map',
    name: 'SeatMapEditor',
    component: SeatMapEditorView,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/manage/theaters/:id/seat-map',
    name: 'TheaterSeatMapEditor',
    component: SeatMapEditorView,
    props: true,
    meta: { requiresAuth: true, roles: ['admin', 'staff', 'event_organizer'] },
  },
  {
    path: '/login',
    name: 'Login',
    component: LoginView,
  },
  {
    path: '/register',
    name: 'Register',
    component: RegisterView,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore();
  if (!authStore.initialized) {
    await authStore.fetchMe();
  }

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    return next({ name: 'Login', query: { redirect: to.fullPath } });
  }

  if (to.meta.roles && Array.isArray(to.meta.roles)) {
    if (!to.meta.roles.includes(authStore.role)) {
      return next({ name: 'Home' });
    }
  }

  next();
});

export default router;
