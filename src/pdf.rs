use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream, dictionary};
use repository::{EventFinancialReportDetail, ShowtimeDashboardRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct PdfService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSeatReceiptItem {
    pub seat_label: String,
    pub zone_name: String,
    pub ticket_code: Option<String>,
    pub price_cents: i64,
}

impl PdfService {
    fn escape_pdf_text(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
    }

    /// Generates an A4 Financial & Tax Statement PDF using lopdf
    pub fn generate_financial_report_pdf(
        report: &EventFinancialReportDetail,
        event_title: &str,
        showtimes: &[ShowtimeDashboardRow],
    ) -> Result<Vec<u8>, String> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();

        let font_regular_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });

        let font_bold_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica-Bold",
        });

        let font_mono_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });

        let mut operations: Vec<Operation> = Vec::new();

        // 1. Header Banner background
        operations.push(Operation::new("q", vec![]));
        // Amber top bar
        operations.push(Operation::new("rg", vec![0.96.into(), 0.62.into(), 0.04.into()]));
        operations.push(Operation::new("re", vec![0.into(), 832.into(), 595.into(), 10.into()]));
        operations.push(Operation::new("f", vec![]));

        // Company Logo & Title
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 18.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![40.into(), 800.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("DREAMCINEMA SETTLEMENT").into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.into()]));
        operations.push(Operation::new("rg", vec![0.4.into(), 0.45.into(), 0.55.into()]));
        operations.push(Operation::new("Td", vec![40.into(), 788.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Box Office Settlement & Tax Authority Document (Reg: TAX-8842-TSB)").into()]));
        operations.push(Operation::new("ET", vec![]));

        // Statement Title & Ref on Right
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 12.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![340.into(), 800.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("FINANCIAL & TAX STATEMENT").into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_MONO".into(), 9.into()]));
        operations.push(Operation::new("rg", vec![0.85.into(), 0.45.into(), 0.0.into()]));
        operations.push(Operation::new("Td", vec![340.into(), 788.into()]));
        let ref_text = format!("Ref: {}", report.statement_ref);
        operations.push(Operation::new("Tj", vec![Object::string_literal(ref_text).into()]));
        operations.push(Operation::new("ET", vec![]));

        // Horizontal Line
        operations.push(Operation::new("RG", vec![0.8.into(), 0.85.into(), 0.9.into()]));
        operations.push(Operation::new("w", vec![1.into()]));
        operations.push(Operation::new("m", vec![40.into(), 775.into()]));
        operations.push(Operation::new("l", vec![555.into(), 775.into()]));
        operations.push(Operation::new("S", vec![]));

        // 2. Event Specification Table Box
        operations.push(Operation::new("rg", vec![0.97.into(), 0.98.into(), 0.99.into()]));
        operations.push(Operation::new("re", vec![40.into(), 715.into(), 515.into(), 50.into()]));
        operations.push(Operation::new("f", vec![]));
        operations.push(Operation::new("RG", vec![0.85.into(), 0.88.into(), 0.92.into()]));
        operations.push(Operation::new("re", vec![40.into(), 715.into(), 515.into(), 50.into()]));
        operations.push(Operation::new("S", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 9.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![48.into(), 750.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Event Title:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 9.into()]));
        operations.push(Operation::new("Td", vec![70.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(event_title)).into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 9.into()]));
        operations.push(Operation::new("Td", vec![48.into(), 732.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Generated:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 9.into()]));
        operations.push(Operation::new("Td", vec![70.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(report.generated_at.format("%Y-%m-%d %H:%M UTC").to_string()).into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 9.into()]));
        operations.push(Operation::new("Td", vec![340.into(), 750.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Currency:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 9.into()]));
        operations.push(Operation::new("Td", vec![60.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("USD ($)").into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 9.into()]));
        operations.push(Operation::new("Td", vec![340.into(), 732.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Organizer:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 9.into()]));
        operations.push(Operation::new("Td", vec![60.into(), 0.into()]));
        let org_email = report.generated_by_email.as_deref().unwrap_or("Cinema Admin");
        operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(org_email)).into()]));
        operations.push(Operation::new("ET", vec![]));

        // 3. Section 1: Executive Financial Summary Table
        let sec1_top = 690.0;
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 11.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![40.into(), sec1_top.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("1. EXECUTIVE FINANCIAL & TAX SUMMARY").into()]));
        operations.push(Operation::new("ET", vec![]));

        // Table Header
        let tbl1_y = sec1_top - 20.0;
        operations.push(Operation::new("rg", vec![0.12.into(), 0.16.into(), 0.24.into()]));
        operations.push(Operation::new("re", vec![40.into(), (tbl1_y - 4.0).into(), 515.into(), 18.into()]));
        operations.push(Operation::new("f", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![1.0.into(), 1.0.into(), 1.0.into()]));
        operations.push(Operation::new("Td", vec![48.into(), tbl1_y.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Revenue & Tax Line Item Description").into()]));
        operations.push(Operation::new("Td", vec![280.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Volume / Units").into()]));
        operations.push(Operation::new("Td", vec![110.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Amount (USD)").into()]));
        operations.push(Operation::new("ET", vec![]));

        // Rows
        let row_height = 20.0;
        let mut curr_y = tbl1_y - 20.0;

        let gross_str = format!("${:.2}", (report.gross_sales_cents as f64) / 100.0);
        let refund_str = format!("-${:.2}", (report.total_refunds_cents as f64) / 100.0);
        let net_profit_str = format!("${:.2}", (report.net_profit_cents as f64) / 100.0);
        let tax_str = format!("-${:.2}", (report.tax_withholding_cents as f64) / 100.0);
        let net_settle_str = format!("${:.2}", (report.net_settlement_cents as f64) / 100.0);

        let lines = vec![
            ("Gross Ticket Sales Revenue", format!("{} Issued", report.tickets_issued), gross_str, false),
            ("Less: Customer Refunds & Cancelations", format!("{} Refunded", report.tickets_refunded), refund_str, true),
            ("Net Operating Ticket Profit (Before Tax)", format!("{} Active", report.tickets_issued - report.tickets_refunded), net_profit_str, false),
            ("Statutory Tax Withholding (10.0%)", "Standard Rate".to_string(), tax_str, false),
            ("FINAL DISBURSABLE SETTLEMENT", "100% Cleared".to_string(), net_settle_str, false),
        ];

        for (desc, volume, amount, is_red) in lines {
            operations.push(Operation::new("RG", vec![0.9.into(), 0.92.into(), 0.95.into()]));
            operations.push(Operation::new("m", vec![40.into(), (curr_y - 4.0).into()]));
            operations.push(Operation::new("l", vec![555.into(), (curr_y - 4.0).into()]));
            operations.push(Operation::new("S", vec![]));

            operations.push(Operation::new("BT", vec![]));
            operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.5.into()]));
            operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
            operations.push(Operation::new("Td", vec![48.into(), curr_y.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(desc).into()]));

            operations.push(Operation::new("Tf", vec!["F_MONO".into(), 8.0.into()]));
            operations.push(Operation::new("Td", vec![280.into(), 0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(volume).into()]));

            if is_red {
                operations.push(Operation::new("rg", vec![0.85.into(), 0.15.into(), 0.15.into()]));
            } else {
                operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
            }
            operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
            operations.push(Operation::new("Td", vec![110.into(), 0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(amount).into()]));
            operations.push(Operation::new("ET", vec![]));

            curr_y -= row_height;
        }

        // 4. Section 2: Showtime Sessions Breakdown
        curr_y -= 10.0;
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 11.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![40.into(), curr_y.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("2. SHOWTIME SESSIONS & ATTENDANCE AUDIT").into()]));
        operations.push(Operation::new("ET", vec![]));

        curr_y -= 18.0;
        operations.push(Operation::new("rg", vec![0.12.into(), 0.16.into(), 0.24.into()]));
        operations.push(Operation::new("re", vec![40.into(), (curr_y - 4.0).into(), 515.into(), 16.into()]));
        operations.push(Operation::new("f", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.0.into()]));
        operations.push(Operation::new("rg", vec![1.0.into(), 1.0.into(), 1.0.into()]));
        operations.push(Operation::new("Td", vec![48.into(), curr_y.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Showtime Session").into()]));
        operations.push(Operation::new("Td", vec![150.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Venue / Hall").into()]));
        operations.push(Operation::new("Td", vec![110.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Seats").into()]));
        operations.push(Operation::new("Td", vec![60.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Scanned").into()]));
        operations.push(Operation::new("Td", vec![70.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Net Sales").into()]));
        operations.push(Operation::new("ET", vec![]));

        curr_y -= 16.0;
        for st in showtimes.iter().take(6) {
            operations.push(Operation::new("RG", vec![0.9.into(), 0.92.into(), 0.95.into()]));
            operations.push(Operation::new("m", vec![40.into(), (curr_y - 3.0).into()]));
            operations.push(Operation::new("l", vec![555.into(), (curr_y - 3.0).into()]));
            operations.push(Operation::new("S", vec![]));

            operations.push(Operation::new("BT", vec![]));
            operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.0.into()]));
            operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
            operations.push(Operation::new("Td", vec![48.into(), curr_y.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(st.starts_at.format("%Y-%m-%d %H:%M").to_string()).into()]));

            operations.push(Operation::new("Td", vec![150.into(), 0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(&st.theater_name)).into()]));

            operations.push(Operation::new("Td", vec![110.into(), 0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(format!("{}/{}", st.sold_seats, st.total_seats)).into()]));

            operations.push(Operation::new("Td", vec![60.into(), 0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(format!("{}", st.used_tickets)).into()]));

            operations.push(Operation::new("Td", vec![70.into(), 0.into()]));
            operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(format!("${:.2}", (st.net_cents as f64) / 100.0)).into()]));
            operations.push(Operation::new("ET", vec![]));

            curr_y -= 15.0;
        }

        // 5. Section 3: Admission & Verification Summary
        curr_y -= 10.0;
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 11.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![40.into(), curr_y.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("3. GATE ENTRY & ADMISSION SUMMARY").into()]));
        operations.push(Operation::new("ET", vec![]));

        curr_y -= 25.0;
        operations.push(Operation::new("rg", vec![0.97.into(), 0.98.into(), 0.99.into()]));
        operations.push(Operation::new("re", vec![40.into(), curr_y.into(), 515.into(), 22.into()]));
        operations.push(Operation::new("f", vec![]));
        operations.push(Operation::new("RG", vec![0.85.into(), 0.88.into(), 0.92.into()]));
        operations.push(Operation::new("re", vec![40.into(), curr_y.into(), 515.into(), 22.into()]));
        operations.push(Operation::new("S", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![48.into(), (curr_y + 7.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(format!("Issued: {}  |  Scanned: {}  |  Attendance Rate: {}%", report.tickets_issued, report.tickets_scanned, report.attendance_rate_percent)).into()]));
        operations.push(Operation::new("ET", vec![]));

        // 6. Signatures & Footer Seal
        operations.push(Operation::new("RG", vec![0.8.into(), 0.85.into(), 0.9.into()]));
        operations.push(Operation::new("m", vec![40.into(), 70.into()]));
        operations.push(Operation::new("l", vec![555.into(), 70.into()]));
        operations.push(Operation::new("S", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 7.5.into()]));
        operations.push(Operation::new("rg", vec![0.5.into(), 0.55.into(), 0.65.into()]));
        operations.push(Operation::new("Td", vec![40.into(), 55.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Certified Authentic by DreamCinema System • Automated Digital Audit Stamp ID: DC-TSB-SYS-2026").into()]));
        operations.push(Operation::new("Td", vec![0.into(), (-10.0_f64).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("This statement represents point-in-time financial transaction records stored in immutable object storage.").into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("Q", vec![]));

        let content = Content { operations };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.28.into(), 841.89.into()],
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F_REG" => font_regular_id,
                    "F_BOLD" => font_bold_id,
                    "F_MONO" => font_mono_id,
                }
            }
        });

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let catalog = dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        };
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set("Root", catalog_id);

        let mut output = Vec::new();
        doc.save_to(&mut output)
            .map_err(|e| format!("Failed to render PDF: {e}"))?;

        Ok(output)
    }

    /// Generates an Order Booking Receipt & Passes PDF using lopdf
    pub fn generate_order_receipt_pdf(
        order_id: Uuid,
        event_title: &str,
        showtime_starts_at: &str,
        theater_name: &str,
        user_email: &str,
        total_price_cents: i64,
        status: &str,
        seats: &[OrderSeatReceiptItem],
    ) -> Result<Vec<u8>, String> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();

        let font_regular_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });

        let font_bold_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica-Bold",
        });

        let font_mono_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });

        let mut operations: Vec<Operation> = Vec::new();

        // 1. Formal Header
        operations.push(Operation::new("q", vec![]));
        // Amber top bar accent
        operations.push(Operation::new("rg", vec![0.96.into(), 0.62.into(), 0.04.into()]));
        operations.push(Operation::new("re", vec![0.into(), 832.into(), 595.into(), 10.into()]));
        operations.push(Operation::new("f", vec![]));

        // Company Logo & Title (Left)
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 18.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![40.into(), 800.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("DREAMCINEMA").into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.0.into()]));
        operations.push(Operation::new("rg", vec![0.4.into(), 0.45.into(), 0.55.into()]));
        operations.push(Operation::new("Td", vec![40.into(), 788.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Official Order Receipt & Digital Ticket Confirmation Pass").into()]));
        operations.push(Operation::new("ET", vec![]));

        // Receipt Title & Status (Right)
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 12.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![340.into(), 800.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("OFFICIAL ORDER RECEIPT").into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_MONO".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.85.into(), 0.45.into(), 0.0.into()]));
        operations.push(Operation::new("Td", vec![340.into(), 788.into()]));
        let short_id = if order_id.to_string().len() >= 8 {
            &order_id.to_string()[..8]
        } else {
            "CONFIRMED"
        };
        let ref_text = format!("Order: #{}", short_id.to_uppercase());
        operations.push(Operation::new("Tj", vec![Object::string_literal(ref_text).into()]));
        operations.push(Operation::new("ET", vec![]));

        // Horizontal Line
        operations.push(Operation::new("RG", vec![0.8.into(), 0.85.into(), 0.9.into()]));
        operations.push(Operation::new("w", vec![1.into()]));
        operations.push(Operation::new("m", vec![40.into(), 775.into()]));
        operations.push(Operation::new("l", vec![555.into(), 775.into()]));
        operations.push(Operation::new("S", vec![]));

        // 2. Order Specification Grid Table Box
        let box_y = 695.0;
        let box_h = 68.0;
        operations.push(Operation::new("rg", vec![0.97.into(), 0.98.into(), 0.99.into()]));
        operations.push(Operation::new("re", vec![40.into(), box_y.into(), 515.into(), box_h.into()]));
        operations.push(Operation::new("f", vec![]));
        operations.push(Operation::new("RG", vec![0.85.into(), 0.88.into(), 0.92.into()]));
        operations.push(Operation::new("re", vec![40.into(), box_y.into(), 515.into(), box_h.into()]));
        operations.push(Operation::new("S", vec![]));

        // Internal grid lines
        operations.push(Operation::new("m", vec![40.into(), (box_y + 45.0).into()]));
        operations.push(Operation::new("l", vec![555.into(), (box_y + 45.0).into()]));
        operations.push(Operation::new("m", vec![40.into(), (box_y + 23.0).into()]));
        operations.push(Operation::new("l", vec![555.into(), (box_y + 23.0).into()]));
        operations.push(Operation::new("m", vec![295.into(), box_y.into()]));
        operations.push(Operation::new("l", vec![295.into(), (box_y + box_h).into()]));
        operations.push(Operation::new("S", vec![]));

        // Row 1: Order Ref & Status
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![48.into(), (box_y + 51.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Order Reference:").into()]));
        operations.push(Operation::new("Tf", vec!["F_MONO".into(), 8.0.into()]));
        operations.push(Operation::new("Td", vec![85.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(format!("#{}", order_id)).into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![305.into(), (box_y + 51.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Payment Status:").into()]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.6.into(), 0.3.into()]));
        operations.push(Operation::new("Td", vec![80.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(status.to_uppercase()).into()]));
        operations.push(Operation::new("ET", vec![]));

        // Row 2: Customer Email & Showtime Date
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![48.into(), (box_y + 29.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Customer Email:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.5.into()]));
        operations.push(Operation::new("Td", vec![85.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(user_email)).into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![305.into(), (box_y + 29.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Showtime:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.5.into()]));
        operations.push(Operation::new("Td", vec![80.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(showtime_starts_at)).into()]));
        operations.push(Operation::new("ET", vec![]));

        // Row 3: Event Title & Venue
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![48.into(), (box_y + 8.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Event Title:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.5.into()]));
        operations.push(Operation::new("Td", vec![85.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(event_title)).into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
        operations.push(Operation::new("Td", vec![305.into(), (box_y + 8.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Venue / Hall:").into()]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.5.into()]));
        operations.push(Operation::new("Td", vec![80.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(theater_name)).into()]));
        operations.push(Operation::new("ET", vec![]));

        // 3. Seats & Ticket Table
        let mut curr_y = box_y - 20.0;
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 10.5.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![40.into(), curr_y.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("BOOKED SEATS & DIGITAL ADMISSION PASSES").into()]));
        operations.push(Operation::new("ET", vec![]));

        curr_y -= 18.0;
        operations.push(Operation::new("rg", vec![0.12.into(), 0.16.into(), 0.24.into()]));
        operations.push(Operation::new("re", vec![40.into(), (curr_y - 4.0).into(), 515.into(), 18.into()]));
        operations.push(Operation::new("f", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 8.5.into()]));
        operations.push(Operation::new("rg", vec![1.0.into(), 1.0.into(), 1.0.into()]));
        operations.push(Operation::new("Td", vec![48.into(), curr_y.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Seat Location").into()]));
        operations.push(Operation::new("Td", vec![130.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Zone").into()]));
        operations.push(Operation::new("Td", vec![140.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Digital Ticket Code").into()]));
        operations.push(Operation::new("Td", vec![110.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Price (USD)").into()]));
        operations.push(Operation::new("ET", vec![]));

        curr_y -= 20.0;
        for seat in seats {
            operations.push(Operation::new("RG", vec![0.9.into(), 0.92.into(), 0.95.into()]));
            operations.push(Operation::new("m", vec![40.into(), (curr_y - 4.0).into()]));
            operations.push(Operation::new("l", vec![555.into(), (curr_y - 4.0).into()]));
            operations.push(Operation::new("S", vec![]));

            operations.push(Operation::new("BT", vec![]));
            operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 9.0.into()]));
            operations.push(Operation::new("rg", vec![0.1.into(), 0.15.into(), 0.25.into()]));
            operations.push(Operation::new("Td", vec![48.into(), curr_y.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(seat.seat_label.as_str()).into()]));

            operations.push(Operation::new("Tf", vec!["F_REG".into(), 9.0.into()]));
            operations.push(Operation::new("Td", vec![130.into(), 0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(Self::escape_pdf_text(&seat.zone_name)).into()]));

            operations.push(Operation::new("Tf", vec!["F_MONO".into(), 9.5.into()]));
            operations.push(Operation::new("rg", vec![0.85.into(), 0.45.into(), 0.0.into()]));
            operations.push(Operation::new("Td", vec![140.into(), 0.into()]));
            let code_str = seat.ticket_code.as_deref().unwrap_or("PENDING");
            operations.push(Operation::new("Tj", vec![Object::string_literal(code_str).into()]));

            operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 9.0.into()]));
            operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
            operations.push(Operation::new("Td", vec![110.into(), 0.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(format!("${:.2}", (seat.price_cents as f64) / 100.0)).into()]));
            operations.push(Operation::new("ET", vec![]));

            curr_y -= 20.0;
        }

        // 4. Total Paid Box with Generous Y-Spacing
        curr_y -= 28.0;
        operations.push(Operation::new("rg", vec![0.95.into(), 0.97.into(), 1.0.into()]));
        operations.push(Operation::new("re", vec![320.into(), curr_y.into(), 235.into(), 30.into()]));
        operations.push(Operation::new("f", vec![]));
        operations.push(Operation::new("RG", vec![0.8.into(), 0.85.into(), 0.95.into()]));
        operations.push(Operation::new("re", vec![320.into(), curr_y.into(), 235.into(), 30.into()]));
        operations.push(Operation::new("S", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 10.into()]));
        operations.push(Operation::new("rg", vec![0.06.into(), 0.09.into(), 0.16.into()]));
        operations.push(Operation::new("Td", vec![335.into(), (curr_y + 10.0).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("TOTAL PAID:").into()]));
        operations.push(Operation::new("Tf", vec!["F_BOLD".into(), 12.into()]));
        operations.push(Operation::new("rg", vec![0.85.into(), 0.45.into(), 0.0.into()]));
        operations.push(Operation::new("Td", vec![95.into(), 0.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(format!("${:.2} USD", (total_price_cents as f64) / 100.0)).into()]));
        operations.push(Operation::new("ET", vec![]));

        // 5. Footer Notes
        operations.push(Operation::new("RG", vec![0.85.into(), 0.88.into(), 0.92.into()]));
        operations.push(Operation::new("m", vec![40.into(), 80.into()]));
        operations.push(Operation::new("l", vec![555.into(), 80.into()]));
        operations.push(Operation::new("S", vec![]));

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new("Tf", vec!["F_REG".into(), 8.into()]));
        operations.push(Operation::new("rg", vec![0.5.into(), 0.55.into(), 0.65.into()]));
        operations.push(Operation::new("Td", vec![40.into(), 65.into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("Show the 6-character Ticket Code or digital QR pass to the gatekeeper at venue entrance for check-in.").into()]));
        operations.push(Operation::new("Td", vec![0.into(), (-12.0_f64).into()]));
        operations.push(Operation::new("Tj", vec![Object::string_literal("DreamCinema Box Office Platform • Auto-generated confirmation pass.").into()]));
        operations.push(Operation::new("ET", vec![]));

        operations.push(Operation::new("Q", vec![]));

        let content = Content { operations };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.28.into(), 841.89.into()],
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F_REG" => font_regular_id,
                    "F_BOLD" => font_bold_id,
                    "F_MONO" => font_mono_id,
                }
            }
        });

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let catalog = dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        };
        let catalog_id = doc.add_object(catalog);
        doc.trailer.set("Root", catalog_id);

        let mut output = Vec::new();
        doc.save_to(&mut output)
            .map_err(|e| format!("Failed to render PDF: {e}"))?;

        Ok(output)
    }
}
