use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Staff,
    EventOrganizer,
    Customer,
    Gatekeeper,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Staff => "staff",
            UserRole::EventOrganizer => "event_organizer",
            UserRole::Customer => "customer",
            UserRole::Gatekeeper => "gatekeeper",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

impl From<User> for UserSummary {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            role: u.role,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Theater {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Zone {
    pub id: Uuid,
    pub theater_id: Uuid,
    pub name: String,
    pub price_cents: i64,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Seat {
    pub id: Uuid,
    pub theater_id: Uuid,
    pub zone_id: Uuid,
    pub row_label: String,
    pub seat_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterLayout {
    pub theater: Theater,
    pub zones: Vec<Zone>,
    pub seats: Vec<Seat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveZoneItem {
    pub id: Option<Uuid>,
    pub name: String,
    pub price_cents: i64,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSeatItem {
    pub id: Option<Uuid>,
    pub zone_id: Uuid,
    pub row_label: String,
    pub seat_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveLayoutPayload {
    pub zones: Vec<SaveZoneItem>,
    pub seats: Vec<SaveSeatItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeatWithStatus {
    pub id: Uuid,
    pub theater_id: Uuid,
    pub zone_id: Uuid,
    pub zone_name: String,
    pub zone_color: Option<String>,
    pub price_cents: i64,
    pub row_label: String,
    pub seat_number: i32,
    pub status: String, // "available", "held", "confirmed"
    pub is_mine: bool,
    pub held_expires_at: Option<DateTime<Utc>>,
    pub seat_booking_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub max_seats_per_booking: i32,
    pub max_seats_per_user: i32,
    pub owner_id: Option<Uuid>,
    pub event_number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Showtime {
    pub id: Uuid,
    pub event_id: Uuid,
    pub theater_id: Uuid,
    pub starts_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShowtimeDetail {
    pub id: Uuid,
    pub event_id: Uuid,
    pub event_title: String,
    pub theater_id: Uuid,
    pub theater_name: String,
    pub starts_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BookingQueueItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub showtime_id: Uuid,
    pub zone_id: Uuid,
    pub seat_id: Uuid,
    pub status: String, // "pending", "processing", "success", "failed"
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeatBooking {
    pub id: Uuid,
    pub showtime_id: Uuid,
    pub seat_id: Uuid,
    pub user_id: Uuid,
    pub booking_queue_id: Option<Uuid>,
    pub status: String, // "held", "confirmed", "released"
    pub held_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HoldItem {
    pub id: Uuid,
    pub showtime_id: Uuid,
    pub price_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub showtime_id: Uuid,
    pub total_amount_cents: i64,
    pub refunded_amount_cents: i64,
    pub status: String, // "pending_payment", "paid", "partially_refunded", "refunded", "cancelled", "expired"
    pub stripe_payment_intent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderSeat {
    pub order_id: Uuid,
    pub seat_booking_id: Uuid,
    pub price_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderSeatDetail {
    pub seat_booking_id: Uuid,
    pub seat_id: Uuid,
    pub row_label: String,
    pub seat_number: i32,
    pub zone_name: String,
    pub price_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderDetailRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: Option<String>,
    pub showtime_id: Uuid,
    pub event_title: String,
    pub theater_name: String,
    pub starts_at: DateTime<Utc>,
    pub total_amount_cents: i64,
    pub refunded_amount_cents: i64,
    pub status: String,
    pub stripe_payment_intent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDetail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: Option<String>,
    pub showtime_id: Uuid,
    pub event_title: String,
    pub theater_name: String,
    pub starts_at: DateTime<Utc>,
    pub total_amount_cents: i64,
    pub refunded_amount_cents: i64,
    pub status: String,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_client_secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub seats: Vec<OrderSeatDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentEvent {
    pub id: Uuid,
    pub stripe_event_id: String,
    pub order_id: Option<Uuid>,
    pub r#type: String,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartitionCandidate {
    pub showtime_id: Uuid,
    pub zone_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ticket {
    pub id: Uuid,
    pub code: String,
    pub order_id: Uuid,
    pub seat_booking_id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub showtime_id: Uuid,
    pub status: String, // "active", "used", "cancelled", "expired", "refunded"
    pub used_at: Option<DateTime<Utc>>,
    pub used_by: Option<Uuid>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub refunded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TicketDetail {
    pub id: Uuid,
    pub code: String,
    pub order_id: Uuid,
    pub seat_booking_id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub event_title: String,
    pub showtime_id: Uuid,
    pub theater_name: String,
    pub starts_at: DateTime<Utc>,
    pub row_label: String,
    pub seat_number: i32,
    pub zone_name: String,
    pub price_cents: i64,
    pub status: String,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by: Option<Uuid>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub refunded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventGatekeeper {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GatekeeperDetail {
    pub user_id: Uuid,
    pub email: String,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Refund {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount_cents: i64,
    pub stripe_refund_id: Option<String>,
    pub refunded_by: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundDetail {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount_cents: i64,
    pub stripe_refund_id: Option<String>,
    pub refunded_by: Option<Uuid>,
    pub refunded_by_email: Option<String>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub ticket_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventDashboardMetrics {
    pub gross_sales_cents: i64,
    pub total_refunds_cents: i64,
    pub net_profit_cents: i64,
    pub total_tickets_created: i64,
    pub active_tickets: i64,
    pub used_tickets: i64,
    pub refunded_tickets: i64,
    pub attendance_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShowtimeDashboardRow {
    pub showtime_id: Uuid,
    pub event_id: Uuid,
    pub event_title: String,
    pub theater_id: Uuid,
    pub theater_name: String,
    pub starts_at: DateTime<Utc>,
    pub total_seats: i64,
    pub sold_seats: i64,
    pub used_tickets: i64,
    pub refunded_tickets: i64,
    pub gross_cents: i64,
    pub refunded_cents: i64,
    pub net_cents: i64,
}
