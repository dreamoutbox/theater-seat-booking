// Placeholder for stripe module
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentIntentResponse {
    pub client_secret: String,
    pub payment_intent_id: String,
}
