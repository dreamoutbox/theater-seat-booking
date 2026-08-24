import { test, expect } from '@playwright/test';

test.describe('Admin Free Upgrade & Order Seat Change Flow', () => {
  test('Bob checks out D2, Admin free-upgrades to A2, verify order items & seat history', async ({ page }) => {
    // -------------------------------------------------------------
    // Phase 1: Customer Bob reserves and checks out Seat D2 ($10)
    // -------------------------------------------------------------

    // 1. Login as customer Bob
    await page.goto('/login');
    await page.fill('input[type="email"]', 'bob@customer.local');
    await page.fill('input[type="password"]', 'customer123');
    await page.click('button[type="submit"]');

    // Wait for login and navigation to home
    await page.waitForURL('http://localhost:5173/');

    // 2. Select showtime for "Interstellar Live Symphony & Screening"
    const eventCard = page.locator('.event-card', { hasText: 'Interstellar Live Symphony & Screening' });
    await expect(eventCard).toBeVisible({ timeout: 10000 });
    await eventCard.getByRole('link', { name: 'Select Seats' }).first().click();

    // Wait for booking page
    await expect(page).toHaveURL(/\/booking\//);

    // 3. Click on Seat D2
    const seatD2 = page.locator('button[title^="Row D, Seat 2 •"]');
    await expect(seatD2).toBeVisible({ timeout: 10000 });
    await seatD2.click();

    // 4. Expect "Your Reservation" to have Seat D2
    const reservationPanel = page.locator('.v-card', { hasText: 'Your Reservation' });
    await expect(reservationPanel.getByText(/Row D,\s*Seat 2/i)).toBeVisible({ timeout: 10000 });

    // 5. Click on proceed to checkout button
    const checkoutBtn = page.getByRole('button', { name: /Proceed to Checkout/i });
    await expect(checkoutBtn).toBeEnabled({ timeout: 10000 });
    await checkoutBtn.click();

    // Wait for checkout page
    await expect(page).toHaveURL(/\/checkout\//);

    // 6. Fill attendee 1 name and phone
    const attendeeNameInput = page.getByLabel(/Attendee 1 Full Name/i);
    await expect(attendeeNameInput).toBeVisible({ timeout: 10000 });
    await attendeeNameInput.fill('Bob');
    const attendeePhoneInput = page.getByLabel(/Attendee 1 Mobile Phone/i);
    await attendeePhoneInput.fill('+1 555-019-9999');

    // 7. Click on request tax invoice checkbox & fill info
    const taxCheckbox = page.getByLabel(/Request Tax Invoice/i);
    await taxCheckbox.check();
    await page.getByLabel(/Company \/ Individual Tax Name/i).fill('Bob Enterprise');
    await page.getByLabel(/Tax ID \/ VAT Registration No\./i).fill('US-987654321');
    await page.getByLabel(/Registered Billing Address/i).fill('456 Market St, San Francisco, CA 94103');

    // 8. Click Pay & Confirm button (redirects to Stripe Checkout)
    const payBtn = page.getByRole('button', { name: /Pay & Confirm/i });
    await expect(payBtn).toBeEnabled();
    await payBtn.click();

    // Complete Payment on Stripe Hosted Checkout Page
    await page.waitForURL(/checkout\.stripe\.com/, { timeout: 30000 });

    const emailInput = page.locator('#email, input[type="email"], input[name="email"]').first();
    if (await emailInput.isVisible({ timeout: 10000 }).catch(() => false)) {
      await emailInput.fill('bob@customer.local');
    }

    const cardNumberInput = page.locator('#cardNumber, input[name="cardNumber"], input[autocomplete="cc-number"]').first();
    await expect(cardNumberInput).toBeVisible({ timeout: 20000 });
    await cardNumberInput.fill('4242424242424242');

    const cardExpiryInput = page.locator('#cardExpiry, input[name="cardExpiry"], input[autocomplete="cc-exp"]').first();
    await expect(cardExpiryInput).toBeVisible({ timeout: 10000 });
    await cardExpiryInput.fill('12/34');

    const cardCvcInput = page.locator('#cardCvc, input[name="cardCvc"], input[autocomplete="cc-csc"]').first();
    await expect(cardCvcInput).toBeVisible({ timeout: 10000 });
    await cardCvcInput.fill('123');

    const billingNameInput = page.locator('#billingName, input[name="billingName"], input[autocomplete="cc-name"]').first();
    if (await billingNameInput.isVisible().catch(() => false)) {
      await billingNameInput.fill('Bob Doe');
    }

    const postalCodeInput = page.locator('#billingPostalCode, input[name="billingPostalCode"], input[autocomplete="postal-code"]').first();
    if (await postalCodeInput.isVisible().catch(() => false)) {
      await postalCodeInput.fill('94103');
    }

    const submitPayBtn = page.locator('button[data-testid="hosted-payment-submit-button"], button[type="submit"], .SubmitButton').first();
    await expect(submitPayBtn).toBeEnabled({ timeout: 15000 });
    await submitPayBtn.click();

    // 9. Expect redirect to /orders/<ORDER-UUID> and save the URL
    await page.waitForURL(/\/orders\/[0-9a-fA-F-]+/, { timeout: 30000 });
    const orderUrl = page.url();

    // -------------------------------------------------------------
    // Phase 2: Login as Admin & Perform Free Upgrade D2 -> A2 (VIP)
    // -------------------------------------------------------------

    // 10. Logout Bob
    await page.getByRole('button', { name: 'Logout' }).click();
    await page.waitForURL('/login');

    // 11. Login as Admin
    await page.fill('input[type="email"]', 'admin@theater.local');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('http://localhost:5173/');

    // 12. Navigate to Bob's order page
    await page.goto(orderUrl);
    await expect(page.getByText(/Tax Name:\s*Bob Enterprise/i)).toBeVisible({ timeout: 10000 });

    // 13. Click on "Change Seat" on order item D2
    const changeSeatBtn = page.getByRole('button', { name: 'Change Seat' }).first();
    await expect(changeSeatBtn).toBeVisible({ timeout: 10000 });
    await changeSeatBtn.click();

    // 14. Expect Change Seat Assignment modal
    const changeSeatModal = page.locator('.v-dialog', { hasText: 'Change Seat Assignment' });
    await expect(changeSeatModal).toBeVisible({ timeout: 10000 });

    // 15. Click on seat A2 in the seat picker canvas
    const seatA2InPicker = changeSeatModal.locator('button[title^="Row A, Seat 2 ("]');
    await expect(seatA2InPicker).toBeVisible({ timeout: 10000 });
    await seatA2InPicker.click();

    // 16. Verify Free Upgrade message
    await expect(
      changeSeatModal.getByText(/Order total remains.*\$10\.00.*No extra charge will be billed to the customer/i)
    ).toBeVisible();

    // 17. Fill reason for seat change
    const reasonInput = changeSeatModal.getByLabel(/Reason for Seat Change/i);
    await reasonInput.fill('test free upgrade');

    // 18. Click on confirm button
    const confirmChangeBtn = changeSeatModal.getByRole('button', { name: /Confirm Change to Row A2/i });
    await expect(confirmChangeBtn).toBeEnabled();
    await confirmChangeBtn.click();

    // -------------------------------------------------------------
    // Phase 3: Assertions after Seat Upgrade
    // -------------------------------------------------------------

    // 19. Expect modal to close
    await expect(changeSeatModal).not.toBeVisible({ timeout: 10000 });

    // 20. Expect Order Items table first row: Row A, Seat 2 (VIP) with $10.00 and Free Upgrade badge
    const orderItemsTable = page.locator('.v-table', { hasText: 'Seat & Attendee' });
    await expect(orderItemsTable.getByText('Row A, Seat 2', { exact: true })).toBeVisible({ timeout: 10000 });
    await expect(orderItemsTable.getByText(/🎁 Free Upgrade from Row D2 \(Standard · \$10\.00\)/)).toBeVisible();
    await expect(orderItemsTable.getByText('VIP', { exact: true })).toBeVisible();
    await expect(orderItemsTable.getByText('Free Upgrade', { exact: true })).toBeVisible();

    // 21. Expect Seat Change History table first row
    const seatHistoryTable = page.locator('.v-table', { hasText: 'Previous Seat' });
    await expect(seatHistoryTable).toBeVisible({ timeout: 10000 });
    await expect(seatHistoryTable.getByText('admin@theater.local')).toBeVisible();
    await expect(seatHistoryTable.getByText('Row D, Seat 2', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText('Standard · $10.00', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText('Row A, Seat 2', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText('VIP · $10.00', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText(/🎁 Free Upgrade from Row D2 \(Standard · \$10\.00\)/)).toBeVisible();
    await expect(seatHistoryTable.getByText(/Reason:\s*test free upgrade/i)).toBeVisible();

    // 22. Expect Financial Summary to remain at $10.00
    const subtotalSection = page.locator('text=Seats Subtotal:').locator('..');
    await expect(subtotalSection.getByText('$10.00')).toBeVisible();

    const netTotalSection = page.locator('text=Net Total:').locator('..');
    await expect(netTotalSection.getByText('$10.00')).toBeVisible();
  });
});
