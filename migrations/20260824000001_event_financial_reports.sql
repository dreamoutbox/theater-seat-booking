-- Create event_financial_reports table for storing historical statement snapshots with 30-min cooldown
CREATE TABLE IF NOT EXISTS event_financial_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    statement_ref VARCHAR(64) NOT NULL,
    generated_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    gross_sales_cents BIGINT NOT NULL DEFAULT 0,
    total_refunds_cents BIGINT NOT NULL DEFAULT 0,
    net_profit_cents BIGINT NOT NULL DEFAULT 0,
    tax_withholding_cents BIGINT NOT NULL DEFAULT 0,
    net_settlement_cents BIGINT NOT NULL DEFAULT 0,
    tickets_issued INT NOT NULL DEFAULT 0,
    tickets_scanned INT NOT NULL DEFAULT 0,
    tickets_refunded INT NOT NULL DEFAULT 0,
    attendance_rate_percent INT NOT NULL DEFAULT 0,
    report_data JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_event_financial_reports_event_id_date 
    ON event_financial_reports(event_id, generated_at DESC);
