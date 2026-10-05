import http from 'k6/http';
import { check, sleep } from 'k6';
import crypto from 'k6/crypto';
import encoding from 'k6/encoding';
import { Counter, Trend } from 'k6/metrics';

// Custom Metrics for Seat Booking & Flash Sale Engine
const seatsHeldSuccess = new Counter('seats_held_success');
const seatsHeldConflict = new Counter('seats_held_conflict');
const ordersCompleted = new Counter('orders_completed');
const holdQueueDuration = new Trend('hold_queue_duration_ms');
const orderDuration = new Trend('order_duration_ms');

// Load seeded test metadata
let testData;
try {
  testData = JSON.parse(open('./test-data.json'));
} catch (e) {
  testData = {
    showtime_id: __ENV.SHOWTIME_ID || '15ca6331-559d-41af-84ad-37a5d34d532f',
    jwt_secret: __ENV.JWT_SECRET || 'theater_booking_poc_jwt_secret_key_1234567890_super_secure',
    users: [],
  };
}

const TARGET_URL = __ENV.TARGET_URL || 'http://backend:3000/api';
const SHOWTIME_ID = __ENV.SHOWTIME_ID || testData.showtime_id;
const JWT_SECRET = __ENV.JWT_SECRET || testData.jwt_secret;

export const options = {
  vus: parseInt(__ENV.VUS || '1000', 10),
  duration: __ENV.DURATION || '30s',
  thresholds: {
    // Expected HTTP failure rate (< 15% to tolerate expected 400s on already-taken seats)
    http_req_failed: ['rate<0.15'],
    seats_held_success: ['count>0'],
    orders_completed: ['count>0'],
  },
};

function base64url(str) {
  try {
    return encoding.b64encode(str, 'rawurl');
  } catch (e) {
    return encoding.b64encode(str, 'std').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }
}

function signHmacSha256(secret, data) {
  try {
    return crypto.hmac('sha256', secret, data, 'base64rawurl');
  } catch (e) {
    try {
      return crypto.hmac('sha256', secret, data, 'base64url');
    } catch (e2) {
      const b64 = crypto.hmac('sha256', secret, data, 'base64');
      return b64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    }
  }
}

function makeJwt(userId, email, secret) {
  const header = base64url(JSON.stringify({ alg: 'HS256', typ: 'JWT' }));
  const now = Math.floor(Date.now() / 1000);
  const payload = base64url(
    JSON.stringify({
      sub: userId,
      email: email,
      role: 'customer',
      exp: now + 86400,
    })
  );
  const signature = signHmacSha256(secret, `${header}.${payload}`);
  return `${header}.${payload}.${signature}`;
}

export default function () {
  // 1. Resolve user for this VU
  let userId, email;
  if (testData.users && testData.users.length > 0) {
    const userIndex = (__VU - 1) % testData.users.length;
    const user = testData.users[userIndex];
    userId = user.id;
    email = user.email;
  } else {
    // Fallback deterministic user
    const hexIndex = (__VU).toString(16).padStart(12, '0');
    userId = `00000000-0000-0000-0002-${hexIndex}`;
    email = `loadtest_user_${__VU}@loadtest.local`;
  }

  const token = makeJwt(userId, email, JWT_SECRET);
  const authHeaders = {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${token}`,
  };

  // 2. Query Available Seats
  const seatsRes = http.get(`${TARGET_URL}/showtimes/${SHOWTIME_ID}/seats`, {
    headers: authHeaders,
  });

  if (seatsRes.status !== 200) {
    sleep(0.5);
    return;
  }

  let seats;
  try {
    seats = seatsRes.json();
  } catch (err) {
    return;
  }

  if (!Array.isArray(seats) || seats.length === 0) {
    sleep(1);
    return;
  }

  const availableSeats = seats.filter((s) => s.status === 'available');
  if (availableSeats.length === 0) {
    // Sold out: wait before next iteration
    sleep(1);
    return;
  }

  // 3. Contention Strategy: pick from the first 25-50 available seats
  // This simulates the realistic spike where multiple customers rush for premier seats
  const poolSize = Math.min(availableSeats.length, 30);
  const candidateSeat = availableSeats[Math.floor(Math.random() * poolSize)];

  // 4. Enqueue Seat Hold
  const holdStart = Date.now();
  const holdPayload = JSON.stringify({
    showtime_id: SHOWTIME_ID,
    seat_id: candidateSeat.id,
  });

  const holdRes = http.post(`${TARGET_URL}/bookings`, holdPayload, {
    headers: authHeaders,
  });

  // If active booking limit reached or validation error, pause and retry
  if (holdRes.status !== 200 && holdRes.status !== 202) {
    seatsHeldConflict.add(1);
    sleep(0.5);
    return;
  }

  const holdData = holdRes.json();
  const bookingUuid = holdData.booking_uuid;
  if (!bookingUuid) {
    return;
  }

  // 5. Poll Queue Status Until Processed by Worker
  let resolvedStatus = null;
  const pollStart = Date.now();
  for (let i = 0; i < 15; i++) {
    sleep(0.2);
    const pollRes = http.get(`${TARGET_URL}/bookings/${bookingUuid}`, {
      headers: authHeaders,
    });

    if (pollRes.status === 200) {
      const statusData = pollRes.json();
      if (statusData.status === 'success' || statusData.status === 'failed') {
        resolvedStatus = statusData.status;
        break;
      }
    }
  }

  holdQueueDuration.add(Date.now() - pollStart);

  if (resolvedStatus !== 'success') {
    // Contention collision: seat was won by another concurrent user
    seatsHeldConflict.add(1);
    sleep(0.5);
    return;
  }

  seatsHeldSuccess.add(1);

  // 6. Create Order for Acquired Hold
  const orderStart = Date.now();
  const orderPayload = JSON.stringify({
    seat_booking_ids: [candidateSeat.id],
    billing_name: `Loadtest Buyer ${__VU}`,
    tax_id: 'US-999-LOADTEST',
    billing_address: '888 Arena Parkway, Suite 100',
  });

  const orderRes = http.post(`${TARGET_URL}/orders`, orderPayload, {
    headers: authHeaders,
  });

  const orderCreated = check(orderRes, {
    'order created (201)': (r) => r.status === 201,
  });

  if (!orderCreated) {
    sleep(0.5);
    return;
  }

  const orderData = orderRes.json();
  const orderId = orderData.order_id;

  // 7. Confirm Payment via Mock Endpoint (Skip Stripe Gateway)
  const confirmRes = http.post(
    `${TARGET_URL}/orders/${orderId}/confirm-mock`,
    null,
    { headers: authHeaders }
  );

  const paymentConfirmed = check(confirmRes, {
    'payment confirmed without Stripe (200)': (r) => r.status === 200,
    'order status paid': (r) => {
      try {
        return r.json().status === 'paid';
      } catch (e) {
        return false;
      }
    },
  });

  if (paymentConfirmed) {
    ordersCompleted.add(1);
    orderDuration.add(Date.now() - orderStart);
  }

  // Small think-time between user actions
  sleep(1);
}
