# AGENTS.md — Theater Seat Booking Project Guide

This document is the authoritative guide for AI coding agents working on this codebase.

---

## 1. Project Overview

A **theater seat booking system** (POC/production-ready) with:
- Multi-role access: Admin, Staff, Event Organizer, Gatekeeper, Customer
- Interactive drag-to-place seat map editor (canvas-like, Vue)
- Queue-based booking engine with Postgres advisory locks (prevents double-booking)
- Stripe payment integration (real + mock)
- QR-code digital tickets generated on payment, scanned by gatekeeper staff
- Owned events, scoped gatekeepers, per-event permission model

---

## 2. Tech Stack

| Layer | Technology |
|---|---|
| Backend language | Rust (Edition 2024) |
| Web framework | Axum 0.8 |
| ORM / DB | sqlx 0.8 (raw SQL, no ORM) |
| Database | PostgreSQL 18 |
| Auth | JWT (jsonwebtoken) + Argon2 password hashing, sent via HttpOnly cookie + Bearer fallback |
| Payment | Stripe API via `reqwest` |
| Email Delivery | `lettre` 0.11 (async SMTP via Tokio & Rustls) |
| Object Storage | MinIO S3 via `aws-sdk-s3` |
| PDF Generation | `lopdf` (binary generation for financial statements & order receipts) |
| Frontend | Vue 3 (Composition API, `<script setup>`) |
| UI Library | Vuetify 4 |
| Frontend state | Pinia |
| Frontend routing | Vue Router 5 |
| HTTP client | Axios (proxied through Vite dev server to `/api`) |
| QR codes | `qrcode.vue` (client-side) |
| Build tool | Vite 8 |
| Dev DB | Docker Compose (Postgres 18 + Adminer + MinIO) |

---

## 3. Project Structure

```
theater-seat-booking/
├── Cargo.toml                  # Root Cargo workspace config
├── Cargo.lock
├── .env                        # Local environment (not committed)
├── .env.example                # Template for .env
├── docker-compose.yml          # Postgres + Adminer containers
├── migrations/                 # SQLx migrations (applied in order)
│   ├── 20260818000001_initial_schema.sql
│   ├── 20260818000002_event_booking_limits.sql
│   ├── 20260818000003_zone_color.sql
│   ├── 20260818000004_cascade_foreign_keys.sql
│   └── 20260819000001_event_owner_tickets_gatekeepers.sql
│
├── dev-bootstrap.sh            # Bootstrap: docker compose up + run backend, worker, frontend in parallel
├── dev-reset-db.sh             # Wipe DB + re-run migrations + seed
├── dev-backend.sh              # Run backend API (with cargo-watch if available)
├── dev-frontend.sh             # Run Vite dev server (frontend)
├── dev-queue-worker.sh         # Run the queue worker (with cargo-watch)
│
├── crates/
│   └── repository/             # Workspace member crate — all SQL lives here
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                         # Re-exports everything
│           ├── models.rs                      # All DB structs + enums
│           ├── user_repository.rs
│           ├── theater_repository.rs
│           ├── zone_repository.rs
│           ├── seat_repository.rs
│           ├── event_repository.rs
│           ├── showtime_repository.rs
│           ├── booking_queue_repository.rs
│           ├── seat_booking_repository.rs
│           ├── order_repository.rs
│           ├── ticket_repository.rs           # Ticket generation + scan
│           ├── gatekeeper_repository.rs       # Event gatekeeper assignments
│           └── lock_repository.rs             # Advisory lock helpers
│
├── src/                        # Main binary crate
│   ├── main.rs                 # Entry point: server startup + pool init
│   ├── lib.rs                  # Exports auth, config, db modules for bins
│   ├── auth.rs                 # JWT, Argon2, Axum extractors, check_role()
│   ├── config.rs               # AppConfig struct from env vars
│   ├── db.rs                   # Pool creation + run_migrations()
│   ├── models.rs               # Re-exports from repository (legacy shim)
│   ├── stripe.rs               # StripeService (create_payment_intent)
│   ├── worker.rs               # Booking queue worker logic (partition processor)
│   ├── bin/
│   │   ├── queue_worker.rs     # Binary: standalone queue polling loop
│   │   └── seed.rs             # Binary: seed demo data after fresh migration
│   └── routes/
│       ├── mod.rs              # Router assembly + CORS + AppState
│       ├── auth.rs             # /auth/register, /auth/login, /auth/logout, /me
│       ├── theater.rs          # Theater, Event, Showtime, Layout CRUD
│       ├── booking.rs          # Seat hold queue + release + status
│       ├── order.rs            # Create order, list/get orders, mock payment
│       ├── webhook.rs          # Stripe webhook handler
│       ├── ticket.rs           # List tickets, scan ticket, get by code
│       └── gatekeeper.rs       # Assign/remove gatekeepers per event
│
└── frontend/
    ├── package.json
    ├── vite.config.js          # Dev proxy: /api → localhost:3000/api
    └── src/
        ├── main.js             # Vue app init (Vuetify + Pinia + Router)
        ├── App.vue             # Root layout with Navbar
        ├── api/
        │   └── client.js       # Axios instance (baseURL=/api, withCredentials)
        ├── stores/
        │   └── auth.js         # Pinia store: user, token, role helpers
        ├── router/
        │   └── index.js        # All routes with requiresAuth + roles guards
        ├── components/
        │   └── Navbar.vue      # Top navigation bar
        └── views/
            ├── HomeView.vue              # Showtimes listing for customers
            ├── LoginView.vue             # Login page
            ├── RegisterView.vue          # Register page
            ├── BookingView.vue           # Customer seat map (read-only seat editor)
            ├── OrdersView.vue            # Order list
            ├── OrderDetailView.vue       # Order + QR ticket pass cards
            ├── ManageEventsView.vue      # Admin/Staff/Organizer event list
            ├── ManageEventDetailView.vue # Event detail + showtimes + gatekeepers
            ├── ManageTheatersView.vue    # Theater CRUD
            ├── SeatMapEditorView.vue     # Interactive seat editor (drag, zones)
            └── ScanTicketView.vue        # Gatekeeper QR scanner page
```

---

## 4. Development Scripts

| Script | Purpose |
|---|---|
| `./dev-bootstrap.sh` | **Full stack dev**: start Docker (Postgres/RustFS/Adminer) + run backend, worker, frontend in parallel |
| `docker compose up -d` | Start Postgres + Adminer (port 8081) + RustFS (port 9000/9001) |
| `./dev-reset-db.sh` | **Full reset**: drop schema → re-apply all migrations → seed demo data |
| `./dev-reset-s3.sh` | **Reset S3**: wipe and recreate `theater-documents` bucket in S3 storage |
| `./dev-backend.sh` | Start Rust API on port 3000 (hot-reloads with `cargo-watch` if installed) |
| `./dev-queue-worker.sh` | Start the booking queue worker (polls every 1s, processes holds) |
| `./dev-frontend.sh` | Start Vite dev server on port 5173 (proxies `/api` to backend) |
| `cargo check --workspace` | Verify all crates compile without errors |
| `cargo run --bin seed` | Re-seed demo data (idempotent, skips existing) |
| `./test-e2e.sh` | Reset database & run all Playwright E2E tests (supports flags e.g. `--headed`, `--ui`) |

> **IMPORTANT**: Always run `cargo check --workspace` after backend changes.
> **Do NOT run tests automatically** — the user runs tests manually and reports results.

---

## 5. Environment Variables

All read from `.env` (loaded by `dotenvy`) with safe defaults for local dev:

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `postgres://postgres:example@localhost:5432/postgres` | PostgreSQL connection string |
| `JWT_SECRET` | `theater_booking_poc_jwt_secret_key_default_12345` | HMAC key for JWT signing |
| `PORT` | `3000` | Axum server port |
| `HOLD_TTL_SECONDS` | `600` | Seat hold expiry (10 min) |
| `POLL_INTERVAL_MS` | `1000` | Queue worker polling interval |
| `MAX_BOOKINGS_PER_USER` | `4` | Max active held seats per user globally |
| `STRIPE_SECRET_KEY` | `sk_test_mock_key_for_poc` | Stripe API key |
| `STRIPE_WEBHOOK_SECRET` | `whsec_mock_key_for_poc` | Stripe webhook signing secret |
| `STRIPE_CURRENCY` | `thb` | Three-letter ISO currency code for Stripe charges (e.g. `thb`, `usd`, `eur`) |
| `RUST_LOG` | `info,theater_seat_booking=debug` | Tracing filter |
| `S3_ENDPOINT` | `http://127.0.0.1:9000` | MinIO S3 endpoint |
| `S3_ACCESS_KEY` | `miniominio` | S3 Access Key |
| `S3_SECRET_KEY` | `miniominio` | S3 Secret Key |
| `S3_BUCKET` | `theater-documents` | S3 bucket name |
| `S3_REGION` | `us-east-1` | S3 region |
| `SMTP_HOST` | `smtp.ethereal.email` | SMTP relay host (optional) |
| `SMTP_PORT` | `587` | SMTP port (587 STARTTLS / 465 TLS) |
| `SMTP_USERNAME` | `""` | SMTP authentication username |
| `SMTP_PASSWORD` | `""` | SMTP authentication password |
| `SMTP_FROM_EMAIL` | `"Theater Booking <noreply@theaterbooking.local>"` | Outgoing sender email address |

---

## 6. Backend: Rust Coding Patterns

### 6.1 Workspace Crate Split

All SQL queries live in the `crates/repository` workspace crate.
**Never add `sqlx::query` calls directly to `src/`** — add them to the appropriate `*_repository.rs`.

```rust
// Correct — query in repository crate, called from route
let event = EventRepository::find_by_id(&state.pool, id).await?;

// Wrong — raw query in route handler
sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
```

### 6.2 Repository Method Naming Convention (Spring Boot style)

| Pattern | Example |
|---|---|
| `find_by_*` | `find_by_id`, `find_by_email`, `find_by_owner_id` |
| `find_all` | Full table scan with optional ordering |
| `save` | INSERT (create new) |
| `update` | UPDATE by id |
| `delete_by_id` | DELETE |
| `exists_by_*` | Returns `bool` |
| `count_*` | Returns scalar count |
| `update_status` | Targeted status-only UPDATE |

### 6.3 Route Error Pattern

All handlers return `Result<_, (StatusCode, Json<serde_json::Value>)>`.
Use `.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?`.

```rust
pub async fn my_handler(
    auth_user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<MyType>, (StatusCode, Json<serde_json::Value>)> {
    let result = MyRepository::find_by_id(&state.pool, id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;
    Ok(Json(result))
}
```

### 6.4 Auth Extractors

Two Axum extractors are defined in `src/auth.rs`:

| Extractor | Behavior |
|---|---|
| `AuthenticatedUser` | Required auth — returns 401 if missing/invalid token |
| `OptionalAuthenticatedUser(Option<AuthenticatedUser>)` | Graceful — None if unauthenticated |

Use `check_role(&auth_user, &[UserRole::Admin, UserRole::Staff])` for role enforcement.

### 6.5 User Roles (Enum)

Defined in `crates/repository/src/models.rs`, maps to Postgres `user_role` ENUM:

```rust
pub enum UserRole {
    Admin,          // Full access
    Staff,          // Same as Admin for most operations
    EventOrganizer, // Manages own events only
    Gatekeeper,     // Can scan tickets for assigned events
    Customer,       // Books seats, views own orders
}
```

### 6.6 Transaction Pattern

Always use explicit transactions for multi-step operations:

```rust
let mut tx = state.pool.begin().await?;
// ... operations with &mut *tx
tx.commit().await?;
```

For `create_tickets_for_paid_order`, pass `&mut *tx` as `&mut PgConnection` directly — this function cannot use the generic `E: PgExecutor + Copy` bound because it reuses the connection across multiple queries in a loop.

### 6.7 Adding New Routes

1. Create `src/routes/my_feature.rs`
2. Add `pub mod my_feature;` to `src/routes/mod.rs`
3. Register routes in `create_router()` inside `src/routes/mod.rs`
4. If you need new HTTP verbs (e.g., `delete`), add them to the `use axum::routing::{...}` line in `mod.rs`

---

## 7. Frontend: Vue Coding Patterns

### 7.1 All API Calls via `api/client.js`

```js
import api from '../api/client';
const res = await api.get('/events');         // GET /api/events
await api.post('/tickets/scan', { code });    // POST /api/tickets/scan
```

The client sets `baseURL: '/api'`, `withCredentials: true`, and sends `Authorization: Bearer <token>` from localStorage.

### 7.2 Auth Store (`stores/auth.js`)

Available computed properties:
- `isAuthenticated` — any logged-in user
- `isAdmin` — role === 'admin'
- `isStaff` — admin or staff
- `isElevated` — admin, staff, or event_organizer
- `isGatekeeper` — role === 'gatekeeper'
- `role` — raw role string (e.g., `'event_organizer'`)

### 7.3 Router Guards

Routes use `meta: { requiresAuth: true, roles: ['admin', 'staff'] }`. The guard in `router/index.js` blocks unauthenticated access and redirects unauthorized roles to Home.

Gatekeeper users auto-redirect to `/scan` on login (handled in `Navbar.vue` `quickLogin()`).

### 7.4 Dark Theme

All UI uses Vuetify dark theme. Custom background colors:
- Cards/surfaces: `#181c27` (`.bg-surface`)
- Variants/insets: `#1e2330` (`.bg-surface-variant`)
- Accent/highlight: `amber-darken-1`

Always add `theme="dark"` to `<v-card>`, `<v-table>`, `<v-dialog>`, etc.

### 7.5 Time Formatting

**Always use 24-hour format**: `hour12: false` in all `toLocaleString()` calls.

### 7.6 SeatMapEditorView Notes

This is the largest file (~65KB). It is a fully custom canvas-like seat editor:
- Drag to **create** seats (empty space drag)
- Drag **on** a seat to **move** selected seats to a new position
- Click to select individual seats (shift/ctrl for multi-select)
- Seat rows go A–Z, then AA–ZZ; seat numbers 1–100
- Zone color picker with preset colors + hex input

---

## 8. Database Schema

### Core Tables

| Table | Description |
|---|---|
| `users` | All user accounts with `user_role` enum |
| `theaters` | Physical venues |
| `zones` | Pricing zones within a theater (has `color`) |
| `seats` | Individual seats (row_label + seat_number per theater) |
| `events` | Shows/concerts (has `owner_id` FK to users, `max_seats_per_booking`, `max_seats_per_user`) |
| `showtimes` | Specific date/time of an event at a theater |
| `booking_queue` | Queue entries when customer clicks a seat |
| `seat_bookings` | Source of truth for seat occupancy (held/confirmed/released) |
| `orders` | Payment records linking user to showtime |
| `order_seats` | Join table: order ↔ seat_bookings |
| `payment_events` | Stripe webhook idempotency log |
| `tickets` | Digital tickets (one per seat booking, generated on payment) |
| `event_gatekeepers` | Assigns gatekeeper users to specific events |
| `refunds` | Audit trail of ticket refund transactions |
| `refund_tickets` | Join table: refund ↔ tickets refunded |
| `ticket_seat_changes` | Historical tracking log for seat changes with machine-readable metadata |

### Key Constraints

- `seat_bookings` has a **partial unique index** preventing two active claims on the same seat per showtime:
  `UNIQUE (showtime_id, seat_id) WHERE status IN ('held', 'confirmed')`
- `tickets.code` is `CHAR(6) UNIQUE` — 6 alphanumeric chars, no ambiguous characters (no 0, O, 1, I)
- `tickets` has a `UNIQUE INDEX` on `seat_booking_id` — one ticket per seat booking
- `event_gatekeepers` has a composite `PRIMARY KEY (event_id, user_id)`
- `refund_tickets` has a composite `PRIMARY KEY (refund_id, ticket_id)`

### Migration Ordering

Migrations run in filename-sorted order (SQLx). Always use timestamp prefix `YYYYMMDDNNNNNN_`. Never modify an already-applied migration — add a new one.

---

## 9. API Route Reference

All routes are prefixed with `/api`.

### Auth
| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/auth/register` | Public | Register new user |
| POST | `/auth/login` | Public | Login, set cookie + return token |
| POST | `/auth/logout` | Public | Clear cookie |
| GET | `/me` | Optional | Current user info |

### Theaters & Layout
| Method | Path | Auth | Description |
|---|---|---|---|
| GET/POST | `/theaters` | Staff+ (write) | List or create theaters |
| PUT/DELETE | `/theaters/:id` | Staff | Update or delete theater |
| GET | `/theaters/:id/layout` | Optional | Get seat layout |
| PUT | `/theaters/:id/layout` | Staff | Save seat layout |
| GET | `/theaters/:id/seats` | Optional | Seats with showtime status |

### Events & Showtimes
| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/events` | Optional | List events (organizer sees only own) |
| POST | `/events` | Elevated | Create event with optional owner_id |
| PUT | `/events/:id` | Elevated | Update event (organizer = own only) |
| DELETE | `/events/:id` | Elevated | Delete event (organizer = own only) |
| GET | `/events/:id/layout` | Optional | Get event-owned seat layout (zones & seats) |
| PUT | `/events/:id/layout` | Elevated | Save event-owned seat layout (organizer = own only) |
| GET | `/events/:id/booked-seats` | Elevated | List booked seats for an event across showtimes |
| GET/POST | `/showtimes` | Optional/Elevated | List or create showtimes |
| PUT/DELETE | `/showtimes/:id` | Elevated | Update or delete showtime |
| GET | `/showtimes/:id/seats` | Optional | Get seats with availability for a showtime |

### Booking
| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/hold` (alias `/bookings`) | Customer | Enqueue seat hold request |
| POST | `/release-hold` | Customer | Release held seat |
| GET | `/booking/status/:uuid` | Customer | Poll queue item status |

### Orders, Refunds & Seat Changes
| Method | Path | Auth | Description |
|---|---|---|---|
| POST/GET | `/orders` | Auth | Create order or list orders |
| GET | `/orders/:id` | Auth | Get order detail |
| POST | `/orders/:id/confirm-mock` | Auth | Mock payment confirmation |
| POST | `/orders/:id/refund` | Elevated | Refund specific tickets in an order & release seats |
| GET | `/orders/:id/refunds` | Auth | Get refund transaction history for an order |
| POST | `/orders/:id/change-seat` | Auth | Change seat for a booked ticket and log change |
| GET | `/orders/:id/seat-changes` | Auth | Get machine-readable seat change history for an order |
| GET | `/orders/:id/tickets` | Auth | List tickets for an order |
| GET | `/orders/:id/pdf` | Auth | Stream/download Order Receipt PDF from S3 |

### Financial Reports
| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/events/:id/reports` | Elevated | List historical financial & tax report snapshots with cooldown status |
| POST | `/events/:id/reports/generate` | Elevated | Generate snapshot report & upload PDF to S3 (30-min cooldown) |
| GET | `/reports/:id` | Elevated | Get report snapshot details |
| GET | `/reports/:id/pdf` | Elevated | Stream/download Statement PDF from S3 |

### Tickets
| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/tickets/scan` | Gatekeeper/Staff/Admin | Scan ticket code, mark as used |
| GET | `/tickets/:code` | Auth | Get ticket detail by 6-digit code |

### Analytics
| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/analytics/dashboard` | Elevated | Real-time KPI metrics & showtime performance breakdown |

### Gatekeepers
| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/events/:id/gatekeepers` | Elevated | List gatekeepers for event |
| POST | `/events/:id/gatekeepers` | Elevated | Create user + assign as gatekeeper |
| DELETE | `/events/:id/gatekeepers/:user_id` | Elevated | Remove gatekeeper from event |
| GET | `/gatekeeper/events` | Auth | Events assigned to current gatekeeper |

### Webhooks
| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/webhooks/stripe` | Stripe signature | Stripe payment event handler |

---

## 10. Feature History (What Has Been Done)

> Ordered from oldest to newest. This is the complete feature set as of the last commit.

### Backend

1. **Initial schema** — users, theaters, zones, seats, events, showtimes, booking_queue, seat_bookings, orders, order_seats, payment_events
2. **Auth system** — JWT + Argon2, HttpOnly cookie + Bearer header dual-mode, `AuthenticatedUser` and `OptionalAuthenticatedUser` Axum extractors
3. **Theater & layout API** — CRUD theaters, zones, seats; `PUT /theaters/:id/layout` saves the full grid atomically
4. **Event & showtime management** — CRUD with role-based access
5. **Queue-based booking engine** — Postgres advisory locks per (showtime, zone) partition, hold TTL expiry sweeper, `booking_queue` → `seat_bookings` pipeline
6. **Order & payment flow** — Create order, Stripe `payment_intent`, mock confirm endpoint, Stripe webhook handler (idempotent via `payment_events` table)
7. **Event booking limits** — `max_seats_per_booking` and `max_seats_per_user` fields on events
8. **Zone colors** — `color` column on zones, saved in layout PUT
9. **Cascade foreign keys** — All child records cascade-delete from parent
10. **Repository crate** — All SQL extracted from route handlers into `crates/repository` with Spring Boot–style method names; faster incremental compilation
11. **Order ID fix** — `OrderRepository::save` takes explicit `id: Uuid` so the same UUID flows through Stripe metadata → `orders.id` → `order_seats.order_id` FK
12. **Event owner** — `events.owner_id` FK, EventOrganizer-role scoping (create, update, delete, list own events only)
13. **Tickets table** — Created on payment (both mock + Stripe webhook); 1 ticket per seat booking; unique 6-char alphanumeric code (charset excludes 0, O, 1, I)
14. **Gatekeeper role** — New `user_role` enum value `gatekeeper`; `event_gatekeepers` join table; POST `/events/:id/gatekeepers` creates user + assigns; scoped scan authorization
15. **Ticket scan endpoint** — `POST /tickets/scan {code}` marks ticket as `used` with timestamp and `used_by`; scoped to gatekeeper's assigned events
16. **Ticket refund endpoint & inventory release** — `POST /orders/:id/refund` refunds single/multiple active tickets, calls Stripe refund, updates `orders.status` to `partially_refunded`/`refunded`, logs to `refunds` & `refund_tickets`, and releases seat bookings back to inventory
17. **Analytics API** — `GET /analytics/dashboard` returns gross revenue, refunds, net profit, tickets scanned, attendance rate %, and showtimes breakdown with role scoping
18. **MinIO S3 Integration & Auto-Bucket Creation** — S3 client configured for MinIO with automatic `theater-documents` bucket creation on startup and `./dev-reset-s3.sh` reset script
19. **lopdf Binary PDF Generation** — Server-side A4 PDF document generator for Financial Statements & Order Receipt digital ticket passes
20. **Financial & Tax Report Snapshotting** — Snapshot API endpoints (`/events/:id/reports` & `/events/:id/reports/generate`) with role-based cooldowns (0s for Admin/Staff, 5m for Event Organizers) and S3 PDF uploads
21. **Event Organizer Order Scoping** — `GET /orders` & `GET /orders/:id` strictly scoped by event ownership for `event_organizer` role and added `?event_id=...` query filtering
22. **Email Delivery with PDF Pass Attachments** — Asynchronous SMTP order confirmation emails powered by `lettre` with rich HTML + plaintext receipts and attached PDF ticket passes sent upon mock and Stripe payments
23. **Event-Owned Seat Maps & Layout Isolation** — Zones and seats are linked to `events.id`, separating layouts completely across events. Added `/api/events/:id/layout` (with strict organizer ownership authorization) and `/api/events/:id/booked-seats`.

### Frontend

1. **Home page** — Showtimes grouped by event, book buttons
2. **Booking view** — Customer seat map (read-only mode of seat editor), hold + release
3. **Order detail** — Order info, seat table, mock pay button
4. **QR ticket pass cards** — Monospace code badge, client-side QR code, scan status chip
5. **Auth pages** — Login, Register
6. **Manage Events** — Searchable/paginated event table
7. **Event Detail** — Showtimes management + Add/Remove Gatekeepers section
8. **Manage Theaters** — Theater CRUD
9. **Seat Map Editor** — Full interactive editor: drag to create, drag to move, marquee select, zone manager with color picker (presets + hex), save layout, grid from A–Z/AA–ZZ rows × 1–100 seats
10. **Scan Ticket View** (`/scan`) — Gatekeeper scanner page: code input, scan result (green/red card), session scan history
11. **Navbar** — Role-aware links, Demo Switcher with Gatekeeper account, Scan Passes button for gatekeepers/elevated roles
12. **Order items QR column** — Embedded QR code and ticket code inside the Order Items table
13. **Ticket refund flow** — Modal dialog with ticket selection checkboxes + per-row refund trigger + refunded badges & financial summary
14. **Event Analytics Dashboard** (`/manage/dashboard`) — Full KPI summary cards, event selector dropdown, and showtime-by-showtime attendance and financial performance table
15. **Event Management Sidebar** (`EventSidebar.vue`) — Compact navigation sidebar across Event Overview, Scheduled Showtimes, Gatekeepers, Settings, Financial Reports, and Orders
16. **Event Financial Reports List View** (`/manage/events/:id/financial-reports`) — Historical snapshot list, live KPI snapshot cards, inline button cooldown timer, and direct PDF streaming links
17. **Event Settings & Tax Info View** (`/manage/events/:id/settings`) — Configurable seat limits and business tax entity profile for customer receipts & tax statements
18. **Scheduled Showtimes View** (`/manage/events/:id/showtimes`) — Dedicated showtimes management view with session scheduling, theater selection, customer seat map links, and hall layout editor triggers
19. **Event-Scoped Seat Map Editor Navigation** — Direct route `/manage/events/:id/seat-map` accessible from Event Sidebar; new events start with empty canvas without layout selection dialogs; theaters decoupled from seat editing.

---

## 11. Critical Cautions for AI Agents

### STOP: Never Auto-Run Tests
Do NOT run `cargo test` or frontend test commands. The user runs tests manually and reports results.

### STOP: No SQL in `src/routes/`
Never add `sqlx::query*` calls directly into `src/routes/` or `src/`. All SQL belongs in `crates/repository/src/*_repository.rs`. Adding queries elsewhere breaks the crate separation and negates the faster compile time benefit.

### `&mut PgConnection` vs Generic `PgExecutor`
The generic `E: PgExecutor<'e>` bound works for single queries. If a function needs to run multiple queries sequentially on the same connection (e.g., a loop), use `conn: &mut PgConnection` and reborrow as `&mut *conn`. The `Copy` bound is NOT implementable on `&mut PgConnection`.

### Migrations Are Append-Only
NEVER modify an existing migration file — sqlx checksums them and will error on mismatch. Always create a new migration file with a new timestamp prefix.

### Postgres Enum Values Must Exist in DB Before Rust References Them
The Rust `UserRole` enum (in `models.rs`) must match the Postgres `user_role` ENUM exactly. Adding a variant to Rust without a matching migration causes sqlx runtime panics. Add with:
```sql
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'new_value';
```
This must be in a new migration and applied **before** deploying any Rust code that references the new variant.

### Ticket Code Charset Is Fixed
The `tickets.code` generator uses charset `ABCDEFGHJKLMNPQRSTUVWXYZ23456789` (no `0`, `O`, `1`, `I` to avoid visual ambiguity). Do not change this charset — existing codes in the database will not be regenerated.

### `OrderRepository::save` Requires Explicit `id: Uuid`
The Stripe metadata, `orders.id`, and `order_seats.order_id` must all use the same UUID. If you change the order creation flow, always pass the pre-generated UUID explicitly — never let Postgres generate it automatically.

### `SeatMapEditorView.vue` Is Very Large
This file is ~65KB. When editing it, be precise with `replace_file_content`. Read surrounding lines carefully before making changes. Avoid rewriting the entire file.

### Vite Dev Proxy
In development, `http://localhost:5173/api/*` proxies to `http://localhost:3000/api/*` via `vite.config.js`. In production, the Axum server serves the built `frontend/dist/` as static files and handles all `/api/*` routes directly.

### Auth Dual-Mode
JWT is sent both as an HttpOnly cookie (`theater_auth_token`) and as `localStorage.theater_token`. The Axum extractor checks cookie first, then `Authorization: Bearer` header. The frontend sends both simultaneously. Do not remove either mode.

### Event Organizer Scoping
When an `event_organizer` calls `GET /events`, they only see events they own (`owner_id = auth_user.id`). Admin/staff see all events. This is handled in `list_events` with `OptionalAuthenticatedUser`. Do not break this scoping when modifying event list endpoints.

### Always Add Crate Dependencies Before Using Them
If you use a new crate macro or type (e.g., `tracing::`, `rand::`, `chrono::`) in `crates/repository`, add it to `crates/repository/Cargo.toml`. The repository crate has its own `Cargo.toml` separate from the root workspace.

### Response Format
All API responses are JSON. Error responses: `{ "error": "message string" }`. Success responses return the relevant struct or status 200 with json body `{ "message": "..." }`.

### Playwright Tests: Seat & Account Isolation for Parallel Execution
When writing end-to-end Playwright tests:
- **Never reuse the same seat across different test files** (e.g., Test A uses `D1`, Test B uses `D2`, Test C uses `D3`). The booking engine uses Postgres advisory locks and partial unique constraints on `(showtime_id, seat_id)` that will cause lock contention, race conditions, and test timeouts if tests run concurrently.
- **Use distinct demo accounts** (e.g., `alice@customer.local` vs `bob@customer.local`) when multiple tests perform checkout or account actions in parallel.
- Playwright is configured to run tests in parallel (`fullyParallel: true`). Strict seat and account isolation ensures fast, flake-free execution.
