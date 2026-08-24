import { test, expect } from '@playwright/test';

test.describe('Customer Checkout Flow with Stripe Integration', () => {
  test('Alice reserves seat D1, fills attendee & tax info, completes Stripe checkout with card 4242, and verifies paid order & QR ticket pass', async ({ page }) => {
    // -------------------------------------------------------------
    // Phase 1: Login as Customer Alice
    // -------------------------------------------------------------
    await page.goto('/login');
    await page.fill('input[type="email"]', 'alice@customer.local');
    await page.fill('input[type="password"]', 'customer123');
    await page.click('button[type="submit"]');

    // Wait for login and navigation to home
    await page.waitForURL('http://localhost:5173/');

    // -------------------------------------------------------------
    // Phase 2: Select Showtime & Reserve Seat D1
    // -------------------------------------------------------------
    const eventCard = page.locator('.event-card', { hasText: 'Interstellar Live Symphony & Screening' });
    await expect(eventCard).toBeVisible({ timeout: 10000 });
    await eventCard.getByRole('link', { name: 'Select Seats' }).first().click();

    // Wait for booking page
    await expect(page).toHaveURL(/\/booking\//);

    // Click on Seat D1
    const seatD1 = page.locator('button[title^="Row D, Seat 1 •"]');
    await expect(seatD1).toBeVisible({ timeout: 10000 });
    await seatD1.click();

    // Expect "Your Reservation" panel to show Seat D1
    const reservationPanel = page.locator('.v-card', { hasText: 'Your Reservation' });
    await expect(reservationPanel.getByText(/Row D,\s*Seat 1/i)).toBeVisible({ timeout: 10000 });

    // Proceed to checkout
    const checkoutBtn = page.getByRole('button', { name: /Proceed to Checkout/i });
    await expect(checkoutBtn).toBeEnabled({ timeout: 10000 });
    await checkoutBtn.click();

    // Wait for checkout page
    await expect(page).toHaveURL(/\/checkout\//);

    // -------------------------------------------------------------
    // Phase 3: Fill Attendee Details & Tax Invoice Information
    // -------------------------------------------------------------
    const attendeeNameInput = page.getByLabel(/Attendee 1 Full Name/i);
    await expect(attendeeNameInput).toBeVisible({ timeout: 10000 });
    await attendeeNameInput.fill('Alice Doe');
    const attendeePhoneInput = page.getByLabel(/Attendee 1 Mobile Phone/i);
    await attendeePhoneInput.fill('+1 555-019-2834');

    // Request tax invoice and fill billing details
    const taxCheckbox = page.getByLabel(/Request Tax Invoice/i);
    await taxCheckbox.check();

    await page.getByLabel(/Company \/ Individual Tax Name/i).fill('Alice Enterprise');
    await page.getByLabel(/Tax ID \/ VAT Registration No\./i).fill('US-123456789');
    await page.getByLabel(/Registered Billing Address/i).fill('123 Broadway, New York, NY 10001');

    // Click Pay & Confirm to create order and redirect to Stripe Checkout
    const payBtn = page.getByRole('button', { name: /Pay & Confirm/i });
    await expect(payBtn).toBeEnabled();
    await payBtn.click();

    // -------------------------------------------------------------
    // Phase 4: Complete Payment on Stripe Hosted Checkout Page
    // -------------------------------------------------------------
    // Wait for redirection to Stripe Checkout
    await page.waitForURL(/checkout\.stripe\.com/, { timeout: 30000 });

    // Fill Contact Information Email if required/present
    const emailInput = page.locator('#email, input[type="email"], input[name="email"]').first();
    if (await emailInput.isVisible({ timeout: 10000 }).catch(() => false)) {
      await emailInput.fill('alice@customer.local');
    }

    // Fill Card Number (4242 4242 4242 4242)
    const cardNumberInput = page.locator('#cardNumber, input[name="cardNumber"], input[autocomplete="cc-number"]').first();
    await expect(cardNumberInput).toBeVisible({ timeout: 20000 });
    await cardNumberInput.fill('4242424242424242');

    // Fill Expiration Date
    const cardExpiryInput = page.locator('#cardExpiry, input[name="cardExpiry"], input[autocomplete="cc-exp"]').first();
    await expect(cardExpiryInput).toBeVisible({ timeout: 10000 });
    await cardExpiryInput.fill('12/34');

    // Fill Security Code (CVC)
    const cardCvcInput = page.locator('#cardCvc, input[name="cardCvc"], input[autocomplete="cc-csc"]').first();
    await expect(cardCvcInput).toBeVisible({ timeout: 10000 });
    await cardCvcInput.fill('123');

    // Fill Cardholder Name / Billing Name if present
    const billingNameInput = page.locator('#billingName, input[name="billingName"], input[autocomplete="cc-name"]').first();
    if (await billingNameInput.isVisible().catch(() => false)) {
      await billingNameInput.fill('Alice Doe');
    }

    // Fill Billing Postal Code if present
    const postalCodeInput = page.locator('#billingPostalCode, input[name="billingPostalCode"], input[autocomplete="postal-code"]').first();
    if (await postalCodeInput.isVisible().catch(() => false)) {
      await postalCodeInput.fill('10001');
    }

    // Submit payment on Stripe
    const submitPayBtn = page.locator('button[data-testid="hosted-payment-submit-button"], button[type="submit"], .SubmitButton').first();
    await expect(submitPayBtn).toBeEnabled({ timeout: 15000 });
    await submitPayBtn.click();

    // -------------------------------------------------------------
    // Phase 5: Verify Order Confirmation & Digital Ticket Pass
    // -------------------------------------------------------------
    // Expect redirect back to application order detail page
    await page.waitForURL(/\/orders\/[0-9a-fA-F-]+/, { timeout: 30000 });

    // Assert Order Status is Paid / Confirmed
    await expect(page.locator('.v-chip', { hasText: /paid/i })).toBeVisible({ timeout: 20000 });

    // Assert Customer and Tax Details
    await expect(page.getByText(/Tax Name:\s*Alice Enterprise/i)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(/Tax ID:\s*US-123456789/i)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(/123 Broadway, New York, NY 10001/i)).toBeVisible({ timeout: 10000 });

    // Assert Order Item Details: Seat D1, Standard zone, $10.00, Attendee Alice Doe
    await expect(page.getByText('Row D, Seat 1', { exact: true })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('D1', { exact: true }).first()).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(/👤\s*Alice Doe/i)).toBeVisible({ timeout: 10000 });

    // Assert Financial Net Total
    const netTotalSection = page.locator('text=Net Total:').locator('..');
    await expect(netTotalSection.getByText('$10.00')).toBeVisible({ timeout: 10000 });

    // Assert Digital QR Ticket Pass & Download PDF button
    await expect(page.locator('.qr-cell-wrapper')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.v-chip', { hasText: /active/i }).first()).toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('link', { name: /PDF Receipt/i })).toBeVisible({ timeout: 10000 });
  });
});
