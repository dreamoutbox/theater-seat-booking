# Theater Seat Booking POC

A role-based theater seat booking system featuring:
- **Partitioned Booking Queue**: Guarantees per-seat ordering and fairness under concurrency via PostgreSQL transaction-scoped advisory locks on `(showtime_id, zone_id)`.
- **Defense-in-Depth Seat Hold Guarantee**: Partial unique index on `seat_bookings` physically preventing double bookings.
- **Seat Hold Expiry Engine**: Immediate read-time enforcement + background sweeper for stale holds and uncompleted orders.
- **Role-Based Auth**: Argon2id password hashing, JWT in `httpOnly` SameSite cookie, and Axum authorization extractors (`Admin`, `Staff`, `EventOrganizer`, `Customer`).
- **Payments**: Stripe PaymentIntent creation with transactional, idempotent webhook handling.
- **Vue 3 SPA**: Built with Vuetify 3, Pinia, and Vue Router with an interactive seat map, countdown timers, and reusable orders management.

---

## 🛠️ Tech Stack

- **Backend**: Rust, Axum, SQLx (PostgreSQL), Tokio
- **Database**: PostgreSQL (with transaction advisory locks and partial unique indexes)
- **Frontend**: Vue 3 (Composition API), Vuetify 3, Pinia, Vue Router, Axios
- **Payments**: Stripe (PaymentIntents + Webhooks)
- **Auth**: Argon2id + JWT `httpOnly` cookie

---

## 🚀 Quickstart Guide

### 1. Start PostgreSQL
```bash
docker compose up -d
```

### 2. Run Database Migrations & Seed Sample Data
```bash
cargo run --bin seed
```

This seeds theaters, VIP/Premium/Standard zones, seat grids, showtimes, and demo accounts:
| Role | Email | Password |
|---|---|---|
| Administrator | `admin@theater.local` | `admin123` |
| Theater Staff | `staff@theater.local` | `staff123` |
| Event Organizer | `organizer@theater.local` | `organizer123` |
| Customer 1 (Alice) | `alice@customer.local` | `customer123` |
| Customer 2 (Bob) | `bob@customer.local` | `customer123` |

---

### 3. Development Workflow Scripts

Three dedicated helper scripts are provided for local development:

- **Run Backend API Server**:
  ```bash
  ./dev-backend.sh
  ```
  Runs the Axum API on `http://localhost:3000`.

- **Run Dedicated Queue Worker**:
  ```bash
  ./dev-queue-worker.sh
  ```
  Runs the standalone booking queue worker (`src/bin/queue_worker.rs`) draining partition advisory locks.

- **Run Vue 3 Frontend**:
  ```bash
  ./dev-frontend
  ```
  Runs Vite dev server with proxy to backend on `http://localhost:5173`.

---

## 🧪 Running Integration Tests

To run the worker concurrency & isolation integration tests manually:
```bash
cargo test
```
Tests cover:
1. **Scenario 1**: Same seat, same showtime concurrent booking requests (first succeeds, second fails with `seat_taken`).
2. **Scenario 2**: Same seat across different showtimes (both succeed independently).
