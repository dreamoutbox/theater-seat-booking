use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StripePaymentIntentResult {
    pub id: String,
    pub client_secret: String,
    pub amount: i64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StripeCheckoutSessionResult {
    pub id: String,
    pub url: String,
    pub payment_intent: Option<String>,
}

pub struct StripeService {
    secret_key: String,
    client: Client,
}

impl StripeService {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            client: Client::new(),
        }
    }

    pub async fn create_payment_intent(
        &self,
        amount_cents: i64,
        currency: &str,
        order_id: &str,
    ) -> Result<StripePaymentIntentResult, String> {
        // If the key is a mock placeholder, generate mock client secret
        if self.secret_key.is_empty() || self.secret_key.starts_with("sk_test_mock") {
            let mock_id = format!("pi_mock_{}", &order_id[..8.min(order_id.len())]);
            let mock_secret = format!("{}_secret_{}", mock_id, uuid::Uuid::new_v4());
            return Ok(StripePaymentIntentResult {
                id: mock_id,
                client_secret: mock_secret,
                amount: amount_cents,
                status: "requires_payment_method".to_string(),
            });
        }

        // Call real Stripe API via HTTP POST
        let params = [
            ("amount", amount_cents.to_string()),
            ("currency", currency.to_string()),
            ("metadata[order_id]", order_id.to_string()),
            ("automatic_payment_methods[enabled]", "true".to_string()),
        ];

        let response = self
            .client
            .post("https://api.stripe.com/v1/payment_intents")
            .bearer_auth(&self.secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Stripe: {e}"))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Stripe error".to_string());
            return Err(format!("Stripe API error: {error_text}"));
        }

        #[derive(Deserialize)]
        struct StripePiResponse {
            id: String,
            client_secret: Option<String>,
            amount: i64,
            status: String,
        }

        let pi: StripePiResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Stripe response: {e}"))?;

        Ok(StripePaymentIntentResult {
            id: pi.id.clone(),
            client_secret: pi.client_secret.unwrap_or_default(),
            amount: pi.amount,
            status: pi.status,
        })
    }

    pub async fn create_checkout_session(
        &self,
        amount_cents: i64,
        currency: &str,
        order_id: &str,
        event_title: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<StripeCheckoutSessionResult, String> {
        // Mock mode: return a fake checkout URL pointing back to our mock-confirm endpoint
        if self.secret_key.is_empty() || self.secret_key.starts_with("sk_test_mock") {
            let mock_session_id = format!("cs_mock_{}", &order_id[..8.min(order_id.len())]);
            return Ok(StripeCheckoutSessionResult {
                id: mock_session_id,
                url: format!("{cancel_url}?mock_order={order_id}"),
                payment_intent: None,
            });
        }

        let amount_str = amount_cents.to_string();
        let quantity_str = "1";
        let params = [
            ("mode", "payment"),
            ("line_items[0][price_data][currency]", currency),
            ("line_items[0][price_data][unit_amount]", &amount_str),
            ("line_items[0][price_data][product_data][name]", event_title),
            ("line_items[0][quantity]", quantity_str),
            ("metadata[order_id]", order_id),
            ("payment_intent_data[metadata][order_id]", order_id),
            ("success_url", success_url),
            ("cancel_url", cancel_url),
        ];

        let response = self
            .client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .bearer_auth(&self.secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Stripe Checkout: {e}"))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Stripe error".to_string());
            return Err(format!("Stripe Checkout API error: {error_text}"));
        }

        #[derive(Deserialize)]
        struct StripeSessionResponse {
            id: String,
            url: Option<String>,
            payment_intent: Option<String>,
        }

        let session: StripeSessionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Stripe Checkout response: {e}"))?;

        Ok(StripeCheckoutSessionResult {
            id: session.id,
            url: session.url.unwrap_or_default(),
            payment_intent: session.payment_intent,
        })
    }

    pub async fn create_refund(
        &self,
        payment_intent_id: &str,
        amount_cents: i64,
        reason: Option<&str>,
    ) -> Result<String, String> {
        // If mock mode or mock payment intent ID, generate mock refund ID
        if self.secret_key.is_empty()
            || self.secret_key.starts_with("sk_test_mock")
            || payment_intent_id.starts_with("pi_mock_")
        {
            let mock_refund_id = format!("re_mock_{}", &uuid::Uuid::new_v4().to_string()[..8]);
            return Ok(mock_refund_id);
        }

        let mut params = vec![
            ("payment_intent", payment_intent_id.to_string()),
            ("amount", amount_cents.to_string()),
        ];
        if let Some(r) = reason {
            params.push(("reason", r.to_string()));
        }

        let response = self
            .client
            .post("https://api.stripe.com/v1/refunds")
            .bearer_auth(&self.secret_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to send refund request to Stripe: {e}"))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown Stripe error".to_string());
            return Err(format!("Stripe refund error: {error_text}"));
        }

        #[derive(Deserialize)]
        struct StripeRefundResponse {
            id: String,
        }

        let refund_res: StripeRefundResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Stripe refund response: {e}"))?;

        Ok(refund_res.id)
    }
}
