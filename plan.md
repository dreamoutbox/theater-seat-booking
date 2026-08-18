# Theater Seat Booking POC — Plan

## 1. Goals & Scope

Minimal but architecturally honest POC demonstrating: role-based auth, seat inventory with
zone pricing, a **partitioned booking queue** that guarantees per-seat ordering under
concurrency, Stripe checkout, and a role-aware Vue3 SPA.

Explicitly **not** goals for this POC (see §11).

## 2. Tech Stack

- Backend: Rust, Axum, SQLx (Postgres), Tokio
- DB: PostgreSQL
- Frontend: Vue 3 (Composition API, `<script setup>`), Vuetify 3, Pinia
- Payments: Stripe (PaymentIntents + webhooks)
- Auth: JWT, httpOnly cookie (not localStorage — avoids XSS token theft)

## 3. Architecture Overview

```
Vue3 SPA ──HTTP──▶ Axum API ──▶ Postgres
                       │            ▲
                       ▼            │
                 booking_queue ◀────┘
                       ▲
                       │ poll (advisory-lock partitions)
                 Queue Worker(s) ──▶ seat_bookings / orders
                       │
                       ▼
                    Stripe (PaymentIntent + webhook)
```

## 4. Database Schema

```sql
CREATE TYPE user_role AS ENUM ('admin','staff','event_organizer','customer');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'customer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE theaters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    address TEXT
);

CREATE TABLE zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    theater_id UUID NOT NULL REFERENCES theaters(id) ON DELETE CASCADE,
    name TEXT NOT NULL,                 -- "VIP", "Standard"
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0)
);

CREATE TABLE seats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    theater_id UUID NOT NULL REFERENCES theaters(id) ON DELETE CASCADE,
    zone_id UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    row_label TEXT NOT NULL,
    seat_number INT NOT NULL,
    UNIQUE (theater_id, row_label, seat_number)
);

CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL
);

CREATE TABLE showtimes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    theater_id UUID NOT NULL REFERENCES theaters(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL
);

-- The queue: one row per seat-click request
CREATE TABLE booking_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),   -- this IS the booking_uuid returned to client
    user_id UUID NOT NULL REFERENCES users(id),
    showtime_id UUID NOT NULL REFERENCES showtimes(id),
    zone_id UUID NOT NULL REFERENCES zones(id),
    seat_id UUID NOT NULL REFERENCES seats(id),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','processing','success','failed')),
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ
);
CREATE INDEX idx_booking_queue_partition
    ON booking_queue (showtime_id, zone_id, status, created_at);

-- Source of truth for seat occupancy (separate from the queue on purpose)
CREATE TABLE seat_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    showtime_id UUID NOT NULL REFERENCES showtimes(id),
    seat_id UUID NOT NULL REFERENCES seats(id),
    user_id UUID NOT NULL REFERENCES users(id),
    booking_queue_id UUID REFERENCES booking_queue(id),
    status TEXT NOT NULL CHECK (status IN ('held','confirmed','released')),
    held_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

-- Defense-in-depth: DB physically cannot hold two active claims on one seat,
-- regardless of any bug in the worker/lock logic.
CREATE UNIQUE INDEX uniq_active_seat_per_showtime
    ON seat_bookings (showtime_id, seat_id)
    WHERE status IN ('held','confirmed');

CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    showtime_id UUID NOT NULL REFERENCES showtimes(id),
    total_amount_cents BIGINT NOT NULL CHECK (total_amount_cents >= 0),
    status TEXT NOT NULL DEFAULT 'pending_payment'
        CHECK (status IN ('pending_payment','paid','cancelled','expired')),
    stripe_payment_intent_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE order_seats (
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    seat_booking_id UUID NOT NULL REFERENCES seat_bookings(id),
    price_cents BIGINT NOT NULL,
    PRIMARY KEY (order_id, seat_booking_id)
);

-- Stripe webhook idempotency (Stripe redelivers events)
CREATE TABLE payment_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stripe_event_id TEXT UNIQUE NOT NULL,
    order_id UUID REFERENCES orders(id),
    type TEXT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Money**: `BIGINT` cents everywhere (not `INT` — protects against overflow on large
multi-seat totals). Frontend formats via `(cents / 100).toFixed(2)`, never does money math
in floats.

## 5. State Machines

```
booking_queue.status:  pending → processing → success | failed

seat_bookings.status:  held (TTL) → confirmed (payment succeeds)
                                   → released (payment fails / hold expires)

orders.status:         pending_payment → paid
                                        → cancelled | expired
```

## 6. Booking Queue Worker Design

Your spec (per-zone worker ownership + a custom rebalancing algorithm) is essentially
reimplementing Kafka consumer-group partition assignment in application code — heavy for a
POC and a second system to get right (leader election, heartbeats, rebalance protocol).

**Recommendation: use Postgres transaction-scoped advisory locks as the partition
mechanism.** Partition key = `(showtime_id, zone_id)` (not zone alone — your scenario 2
shows partitioning must be per-showtime too, otherwise one worker could permanently own a
hot zone across every showtime).

```rust
loop {
    let partitions = fetch_candidate_partitions(&pool).await?;
    // SELECT DISTINCT showtime_id, zone_id FROM booking_queue
    // WHERE status = 'pending' ORDER BY MIN(created_at)

    for (showtime_id, zone_id) in partitions {
        process_partition(&pool, showtime_id, zone_id).await?;
    }

    tokio::time::sleep(POLL_INTERVAL).await;
}

async fn process_partition(pool: &PgPool, showtime_id: Uuid, zone_id: Uuid) -> Result<()> {
    let key = partition_lock_key(showtime_id, zone_id); // hash to i64

    while let Some(row) = fetch_next_pending(pool, showtime_id, zone_id).await? {
        let mut tx = pool.begin().await?;

        // pg_try_advisory_xact_lock: auto-released on COMMIT/ROLLBACK,
        // including on connection loss / worker crash. No manual unlock,
        // no leaked locks, no stale-lock reaper needed.
        let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
            .bind(key)
            .fetch_one(&mut *tx)
            .await?;

        if !acquired {
            break; // another worker owns this partition right now — move on
        }

        // Row-level lock as a second guard, unique index as the final guard.
        let taken: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM seat_bookings
             WHERE showtime_id = $1 AND seat_id = $2 AND status IN ('held','confirmed')
             FOR UPDATE)"
        )
        .bind(row.showtime_id)
        .bind(row.seat_id)
        .fetch_one(&mut *tx)
        .await?;

        if taken {
            update_queue_status(&mut tx, row.id, "failed", Some("seat_taken")).await?;
        } else {
            insert_seat_hold(&mut tx, &row, hold_ttl()).await?; // relies on unique index too
            update_queue_status(&mut tx, row.id, "success", None).await?;
        }

        tx.commit().await?; // lock, status update, and hold insert commit atomically
    }
    Ok(())
}
```

Why this satisfies "auto rebalance" for free: ownership of a partition isn't statically
assigned to a worker — it's whichever worker's poll loop gets there first and holds the lock
only for the duration of one transaction. Idle workers naturally drain whatever's unlocked.
No coordinator, no heartbeats, no split-brain risk.

**Non-negotiable regardless of worker logic**: the partial unique index on `seat_bookings`
is the actual correctness guarantee. The advisory lock gives you ordering/fairness and
avoids wasted work; the unique index is what makes double-booking *impossible* even if the
worker has a bug — same lesson as the TOCTOU fix in dreamticket2.

### 6.1 Seat Hold Expiry

A held seat that never reaches checkout must not stay locked forever. `seat_bookings`
already has `expires_at` (§4) — this section is what enforces it.

**Config**: `HOLD_TTL` (e.g. 10 minutes), set at insert time in the worker:
`expires_at = now() + HOLD_TTL`.

**Enforcement is two-layered, not just a background job:**

1. **Read-time (immediate, no cron lag)** — every place that checks if a seat is taken
   (`process_partition` in §6, and `POST /orders`) treats an expired hold as not-taken:

   ```sql
   SELECT EXISTS(
       SELECT 1 FROM seat_bookings
       WHERE showtime_id = $1 AND seat_id = $2
         AND status IN ('held','confirmed')
         AND (status = 'confirmed' OR expires_at > now())
       FOR UPDATE
   )
   ```

   This means correctness never depends on the sweeper having run yet — an expired hold
   is immediately eligible to be claimed by the next booking request, even if the row's
   `status` column still literally says `held`.

2. **Sweep task (cleanup / consistency, not correctness)** — a `tokio::time::interval`
   loop in the worker binary (or a separate small binary), every 30s:

   ```rust
   async fn sweep_expired_holds(pool: &PgPool) -> Result<()> {
       let mut tx = pool.begin().await?;

       // Release the seats
       sqlx::query(
           "UPDATE seat_bookings SET status = 'released'
            WHERE status = 'held' AND expires_at <= now()"
       )
       .execute(&mut *tx)
       .await?;

       // Expire any order still waiting on payment whose seats just got released
       sqlx::query(
           "UPDATE orders SET status = 'expired'
            WHERE status = 'pending_payment'
              AND id IN (
                  SELECT order_id FROM order_seats os
                  JOIN seat_bookings sb ON sb.id = os.seat_booking_id
                  WHERE sb.status = 'released'
              )"
       )
       .execute(&mut *tx)
       .await?;

       tx.commit().await?;
       Ok(())
   }
   ```

   This exists so the DB doesn't accumulate stale `held` rows and so `GET /orders`
   reflects reality — but it's a housekeeping job, not the thing preventing double-booking.
   Don't let the sweep interval become a source of race conditions by relying on it for
   correctness; that's what the `expires_at > now()` check in step 1 is for.

**Why not just `DELETE` expired rows instead of `status = 'released'`**: keeping the row
gives you an audit trail (who held what, when, why it was released) and lets `order_seats`
foreign keys stay valid if an order was already created against it.

## 7. API Endpoints (minimal)

```
POST   /auth/register
POST   /auth/login
GET    /me

GET    /theaters/:id/seats?showtime_id=
POST   /bookings                      -> { booking_uuid }         (enqueue)
GET    /bookings/:uuid                -> { status, failure_reason }
POST   /orders                        -> { order_id, stripe_client_secret }
POST   /webhooks/stripe

GET    /orders            (customer: own orders; admin/staff/organizer: all)
```

## 8. Frontend Structure

- `stores/auth.ts` (Pinia) — user + role, drives route guards. **Backend re-checks role on
  every request regardless** — frontend role gating is UX only, never authorization.
- `SeatMap.vue` — fetches full seat status on load; re-fetches only when the user's own
  tracked `booking_uuid` resolves (success/failed), not on a generic interval (see §12).
- `OrdersTable.vue` — single reusable component, `scope: 'own' | 'all'` prop; backend
  decides what "all" means per role, component just renders what the API returns.
- Router meta `{ roles: [...] }` + navigation guard for page-level access.

## 9. Payment Flow

1. User accumulates one or more seats in `seat_bookings.status = 'held'` via independent
   per-seat booking requests (§12 — no cart-level transaction; each seat succeeds/fails on
   its own).
2. `POST /orders` — server re-validates holds still valid & unexpired, creates `orders`
   row (`pending_payment`) + Stripe PaymentIntent, returns `client_secret`.
3. Frontend confirms payment with Stripe.js.
4. `/webhooks/stripe` on `payment_intent.succeeded`: check `payment_events` for the Stripe
   event id (idempotency — Stripe redelivers), then mark order `paid` and seats `confirmed`
   in one transaction.
5. On failure/expiry: order → `expired`/`cancelled`, seats → `released`.

## 10. Gaps — Resolved

All open questions from earlier drafts (webhook idempotency, booking-limit check
ordering, partial-failure UX, seat map refresh behavior, seeding, worker tests,
observability) are answered and locked in §12.

## 11. Explicitly Out of Scope for POC

Refunds/cancellations, multi-currency, coupons/discounts,
email notifications, i18n.

## 12. Locked Decisions (do not leave these to the coding agent's judgment)

These were open questions in earlier drafts. Pinning concrete values/choices so
implementation doesn't drift or improvise:

- **`HOLD_TTL = 10 minutes`**. **`POLL_INTERVAL = 1s`** for worker partition scan.
  **`MAX_BOOKINGS_PER_USER = 4`** active (`held` seat_bookings + `pending`/`processing`
  booking_queue rows, combined) per user at a time. Put all three in env config, not
  hardcoded, but these are the POC defaults.
- **Partial cart failure policy: keep the successes, reselect only the failed seat(s).**
  Each seat click is already an independent `POST /bookings` call with its own
  `booking_uuid` (§7) — there is no cart-level transaction to roll back. If seat A1
  succeeds and A2 fails, A1's hold stands untouched; the frontend shows A2 as taken and
  lets the user pick a different seat, re-issuing a new `POST /bookings` for just that
  one. `POST /orders` is only ever called with the set of currently-`held` seat_bookings
  at the moment the user hits checkout — by definition it never includes a failed seat.
  No special "cart" concept needed on the backend at all.
- **One order = one showtime.** `POST /orders` must reject a request whose seat_bookings
  span more than one `showtime_id` — validate in the handler before creating the order row.
  Cross-showtime cart is out of scope (§11).
- **Password hashing: Argon2id** (`argon2` crate), not bcrypt/sha256. Use the crate's
  default params; don't hand-roll cost parameters for a POC.
- **CSRF**: since the JWT lives in an httpOnly cookie, set `SameSite=Strict` (or `Lax` if
  the SPA and API are on different subdomains and you need cross-site navigation to work)
  and require the API to reject requests without it. `SameSite=Strict` alone is sufficient
  for a same-origin SPA POC — don't add a separate CSRF token scheme on top, that's
  over-engineering for this scope.
- **Role authorization is middleware, not per-handler `if`**: a single Axum extractor/
  middleware that reads role from the JWT claims and checks against a route's required
  role(s), applied consistently. Prevents the "forgot to check role on this one endpoint"
  class of bug.
- **Seat map refresh trigger**: no generic interval poll of the whole seat map. It fetches
  full state once on load, then re-fetches only when the currently-tracked `booking_uuid`
  poll (§7 `GET /bookings/:uuid`) resolves to `success` or `failed`. Other users' in-flight
  `pending` bookings are never shown — only settled `held`/`confirmed`/freed seats, and only
  as a side effect of your own booking action resolving. This is a deliberate scope
  limitation, not a bug: no live multi-user view of the seat map in this POC.
- **Stripe webhook idempotency**: every webhook handler invocation starts by checking
  `payment_events` for the incoming `stripe_event_id`; if present, return 200 immediately
  without reprocessing. Insert the event row in the same transaction as the order/seat
  status update, not before or after — avoids a window where the event is marked processed
  but the state change didn't actually commit.
- **Max booking limit check ordering**: `POST /bookings` runs the active-count check and
  the `INSERT INTO booking_queue` in the same transaction (`SELECT COUNT(...) FOR UPDATE`
  style guard, or a `CHECK`-friendly count query immediately before insert) so two parallel
  requests from the same user can't both pass the check before either commits.
- **Migrations & seed**: `sqlx migrate add ...` per schema change from §4, plus one seed
  binary/script (`cargo run --bin seed`) that creates a theater, 2–3 zones, a seat grid,
  an event, and a couple of showtimes — needed for manual demoing and as fixture setup for
  the §6 worker integration tests.
- **Worker integration tests**: `#[sqlx::test]` cases reproducing scenario 1 (same seat,
  same showtime, second request fails) and scenario 2 (same seat, different showtimes,
  both succeed) from the original spec, run against a real ephemeral Postgres — this is
  the part most likely to silently regress, so it's the one piece of test coverage that
  isn't optional for the POC.
- **Observability**: every log line and trace span in the booking path (API handler,
  queue insert, worker pickup, worker resolution) carries `booking_uuid` as a field.
  Minimum metrics: queue depth per partition, time-in-queue (enqueue → processed_at).
  This is also the best way to *demonstrate* the auto-balancing behavior in a demo —
  show two workers each draining a different partition concurrently.
