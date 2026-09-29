use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Attachment, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use repository::{OrderRepository, TicketRepository};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct EmailService {
    smtp_host: Option<String>,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_from_email: String,
}

impl EmailService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            smtp_host: config.smtp_host.clone(),
            smtp_port: config.smtp_port,
            smtp_username: config.smtp_username.clone(),
            smtp_password: config.smtp_password.clone(),
            smtp_from_email: config.smtp_from_email.clone(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.smtp_host
            .as_ref()
            .map(|h| !h.trim().is_empty())
            .unwrap_or(false)
    }

    /// Sends an order confirmation email with the generated order receipt & passes PDF attached
    pub async fn send_order_confirmation(
        &self,
        pool: &PgPool,
        order_id: Uuid,
        pdf_bytes: Vec<u8>,
    ) -> Result<(), String> {
        let order = OrderRepository::find_order_detail_by_id(pool, order_id)
            .await
            .map_err(|e| format!("Database error fetching order detail: {e}"))?
            .ok_or_else(|| format!("Order {order_id} not found"))?;

        let recipient_email = match &order.user_email {
            Some(email) if !email.trim().is_empty() => email.trim().to_string(),
            _ => {
                warn!("No recipient email found for order {order_id}. Skipping email delivery.");
                return Ok(());
            }
        };

        if !self.is_configured() {
            info!(
                order_id = %order_id,
                recipient = %recipient_email,
                "SMTP not configured (SMTP_HOST is empty). Skipping email delivery."
            );
            return Ok(());
        }

        let host = self.smtp_host.as_ref().unwrap();

        let seats = OrderRepository::find_order_seats_by_order_id(pool, order_id)
            .await
            .unwrap_or_default();

        let tickets = TicketRepository::find_by_order_id(pool, order_id)
            .await
            .unwrap_or_default();

        let starts_at_str = order.starts_at.format("%Y-%m-%d %H:%M UTC").to_string();
        let total_formatted = format!("${:.2}", (order.total_amount_cents as f64) / 100.0);

        // Build seat & ticket summary lines
        let mut seat_text_lines = Vec::new();
        let mut seat_html_cards = Vec::new();

        for s in &seats {
            let code = tickets
                .iter()
                .find(|t| t.seat_booking_id == s.seat_booking_id)
                .map(|t| t.code.clone())
                .unwrap_or_else(|| "—".to_string());

            let price_fmt = format!("${:.2}", (s.price_cents as f64) / 100.0);

            seat_text_lines.push(format!(
                "  • Row {} - Seat {} | Zone: {} | Price: {} | Ticket Code: {}",
                s.row_label, s.seat_number, s.zone_name, price_fmt, code
            ));

            seat_html_cards.push(format!(
                r#"<tr style="border-bottom: 1px solid #334155;">
                    <td style="padding: 12px 8px; color: #f8fafc; font-weight: 500;">Row {row} - Seat {seat}</td>
                    <td style="padding: 12px 8px; color: #94a3b8;">{zone}</td>
                    <td style="padding: 12px 8px; color: #f8fafc;">{price}</td>
                    <td style="padding: 12px 8px; text-align: right;">
                      <span style="display: inline-block; background-color: #f59e0b; color: #000000; font-family: 'Courier New', Courier, monospace; font-size: 14px; font-weight: 700; padding: 4px 10px; border-radius: 4px; letter-spacing: 2px;">{code}</span>
                    </td>
                  </tr>"#,
                row = s.row_label,
                seat = s.seat_number,
                zone = s.zone_name,
                price = price_fmt,
                code = code,
            ));
        }

        let plain_body = format!(
            r#"Dear Customer,

Thank you for your booking! Your payment has been confirmed.

==================================================
BOOKING CONFIRMATION & DIGITAL PASSES
==================================================
Order ID:    {order_id}
Event:       {event_title}
Venue:       {theater_name}
Showtime:    {starts_at}
Total Paid:  {total_formatted}

Booked Seats & Passes:
{seats_plain}

Your official digital ticket passes & order receipt PDF is attached to this email.
Please present your 6-character Ticket Code or QR code upon arrival at the venue.

Enjoy the show!
Theater Booking Team
"#,
            order_id = order.id,
            event_title = order.event_title,
            theater_name = order.theater_name,
            starts_at = starts_at_str,
            total_formatted = total_formatted,
            seats_plain = seat_text_lines.join("\n"),
        );

        let html_seats_table = seat_html_cards.join("\n");

        let html_body = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Booking Confirmation</title>
</head>
<body style="margin: 0; padding: 24px; background-color: #0b0f19; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #f8fafc;">
  <div style="max-width: 600px; margin: 0 auto; background-color: #151b28; border-radius: 12px; overflow: hidden; border: 1px solid #1e293b; box-shadow: 0 10px 25px rgba(0,0,0,0.5);">
    
    <!-- Header Banner -->
    <div style="background: linear-gradient(135deg, #d97706 0%, #f59e0b 50%, #fbbf24 100%); padding: 28px 24px; text-align: center;">
      <h1 style="margin: 0; color: #000000; font-size: 24px; font-weight: 800; letter-spacing: -0.5px;">🎟️ Booking Confirmed!</h1>
      <p style="margin: 6px 0 0 0; color: #1f2937; font-size: 14px; font-weight: 600;">Your digital tickets &amp; passes are ready</p>
    </div>

    <!-- Main Content -->
    <div style="padding: 24px;">
      
      <!-- Event Summary Card -->
      <div style="background-color: #1e2638; border: 1px solid #2d3748; border-radius: 8px; padding: 20px; margin-bottom: 24px;">
        <h2 style="margin: 0 0 16px 0; color: #f59e0b; font-size: 18px; font-weight: 700;">{event_title}</h2>
        
        <table style="width: 100%; border-collapse: collapse; font-size: 14px;">
          <tr>
            <td style="padding: 6px 0; color: #94a3b8; width: 120px;">Showtime:</td>
            <td style="padding: 6px 0; color: #f8fafc; font-weight: 600;">{starts_at}</td>
          </tr>
          <tr>
            <td style="padding: 6px 0; color: #94a3b8;">Venue:</td>
            <td style="padding: 6px 0; color: #f8fafc; font-weight: 600;">{theater_name}</td>
          </tr>
          <tr>
            <td style="padding: 6px 0; color: #94a3b8;">Order ID:</td>
            <td style="padding: 6px 0; color: #94a3b8; font-family: monospace; font-size: 13px;">{order_id}</td>
          </tr>
          <tr>
            <td style="padding: 6px 0; color: #94a3b8;">Total Paid:</td>
            <td style="padding: 6px 0; color: #10b981; font-weight: 700; font-size: 16px;">{total_formatted}</td>
          </tr>
        </table>
      </div>

      <!-- Passes & Seats Section -->
      <h3 style="margin: 0 0 12px 0; color: #f8fafc; font-size: 16px; font-weight: 700;">Your Seats &amp; Entry Codes</h3>
      <table style="width: 100%; border-collapse: collapse; margin-bottom: 24px; font-size: 14px;">
        <thead>
          <tr style="border-bottom: 2px solid #334155; text-align: left;">
            <th style="padding: 8px; color: #94a3b8; font-size: 12px; text-transform: uppercase;">Seat</th>
            <th style="padding: 8px; color: #94a3b8; font-size: 12px; text-transform: uppercase;">Zone</th>
            <th style="padding: 8px; color: #94a3b8; font-size: 12px; text-transform: uppercase;">Price</th>
            <th style="padding: 8px; color: #94a3b8; font-size: 12px; text-transform: uppercase; text-align: right;">Pass Code</th>
          </tr>
        </thead>
        <tbody>
          {html_seats_table}
        </tbody>
      </table>

      <!-- Note on PDF attachment -->
      <div style="background-color: #0f172a; border-left: 4px solid #f59e0b; padding: 14px 16px; border-radius: 4px; margin-bottom: 20px; font-size: 13px; color: #cbd5e1; line-height: 1.5;">
        📄 <strong>PDF Pass Attached:</strong> We have attached your official digital ticket passes and order receipt (PDF) to this email. You can present the PDF on your smartphone or print it out for entry.
      </div>
    </div>

    <!-- Footer -->
    <div style="background-color: #0f172a; padding: 16px 24px; text-align: center; border-top: 1px solid #1e293b; font-size: 12px; color: #64748b;">
      <p style="margin: 0 0 4px 0;">Need assistance? Reply to this email or contact theater staff at venue entry.</p>
      <p style="margin: 0;">Theater Seat Booking System &copy; 2026</p>
    </div>

  </div>
</body>
</html>"#,
            order_id = order.id,
            event_title = order.event_title,
            theater_name = order.theater_name,
            starts_at = starts_at_str,
            total_formatted = total_formatted,
            html_seats_table = html_seats_table,
        );

        let pdf_filename = format!("receipt_{}.pdf", order.id);

        let short_id = order.id.to_string();
        let fallback_label = format!("Order #{}", &short_id[..8]);
        let order_ref_label = order
            .tax_invoice_number
            .as_deref()
            .unwrap_or(&fallback_label);
        let email = Message::builder()
            .from(
                self.smtp_from_email
                    .parse()
                    .map_err(|e| format!("Invalid from address: {e}"))?,
            )
            .to(recipient_email
                .parse()
                .map_err(|e| format!("Invalid recipient address '{recipient_email}': {e}"))?)
            .subject(format!(
                "🎟️ Order Receipt & Tickets: {} ({})",
                order.event_title,
                order_ref_label
            ))
            .multipart(
                MultiPart::mixed()
                    .multipart(
                        MultiPart::alternative()
                            .singlepart(SinglePart::plain(plain_body))
                            .singlepart(SinglePart::html(html_body)),
                    )
                    .singlepart(
                        Attachment::new(pdf_filename).body(
                            pdf_bytes,
                            ContentType::parse("application/pdf")
                                .map_err(|e| format!("Invalid content type: {e}"))?,
                        ),
                    ),
            )
            .map_err(|e| format!("Failed to build email message: {e}"))?;

        // Build SMTP transport: use relay for port 465 (implicit TLS) and starttls_relay for 587 (STARTTLS)
        let mut transport_builder = if self.smtp_port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .map_err(|e| format!("Failed to create SMTPS relay for '{host}': {e}"))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                .map_err(|e| format!("Failed to create STARTTLS SMTP relay for '{host}': {e}"))?
        }
        .port(self.smtp_port);

        if let (Some(username), Some(password)) = (&self.smtp_username, &self.smtp_password) {
            if !username.is_empty() {
                transport_builder = transport_builder
                    .credentials(Credentials::new(username.clone(), password.clone()));
            }
        }

        let mailer = transport_builder.build();

        info!(
            order_id = %order_id,
            recipient = %recipient_email,
            smtp_host = %host,
            smtp_port = self.smtp_port,
            "Sending order confirmation email with attached PDF..."
        );

        mailer
            .send(email)
            .await
            .map_err(|e| format!("Failed to send SMTP email: {e}"))?;

        info!(
            order_id = %order_id,
            recipient = %recipient_email,
            "Successfully sent order confirmation email with attached PDF!"
        );

        Ok(())
    }
}
