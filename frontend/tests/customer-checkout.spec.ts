import { test, expect } from '@playwright/test';

test.describe('Customer Checkout Flow', () => {
  test('Alice reserves seat D1, fills attendee & tax info, and completes checkout', async ({ page }) => {
    // 1. Login as customer Alice
    await page.goto('http://localhost:5173/login');
    await page.fill('input[type="email"]', 'alice@customer.local');
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

    // 3. Click on Seat D1
    const seatD1 = page.locator('button[title^="Row D, Seat 1 •"]');
    await expect(seatD1).toBeVisible({ timeout: 10000 });
    await seatD1.click();

    // 4. Expect "Your Reservation" to have Seat D1
    const reservationPanel = page.locator('.v-card', { hasText: 'Your Reservation' });
    await expect(reservationPanel.getByText(/Row D,\s*Seat 1/i)).toBeVisible({ timeout: 10000 });

    // 5. Click on proceed to checkout button
    const checkoutBtn = page.getByRole('button', { name: /Proceed to Checkout/i });
    await expect(checkoutBtn).toBeEnabled({ timeout: 10000 });
    await checkoutBtn.click();

    // Wait for checkout page
    await expect(page).toHaveURL(/\/checkout\//);

    // 6. Fill attendee 1 name and phone
    const attendeeNameInput = page.getByLabel(/Attendee 1 Full Name/i);
    await expect(attendeeNameInput).toBeVisible({ timeout: 10000 });
    await attendeeNameInput.fill('Alice');
    const attendeePhoneInput = page.getByLabel(/Attendee 1 Mobile Phone/i);
    await attendeePhoneInput.fill('+1 555-019-2834');

    // 7. Click on request tax invoice checkbox
    const taxCheckbox = page.getByLabel(/Request Tax Invoice/i);
    await taxCheckbox.check();

    // 8. Fill tax name, tax id, address
    await page.getByLabel(/Company \/ Individual Tax Name/i).fill('Alice Enterprise');
    await page.getByLabel(/Tax ID \/ VAT Registration No\./i).fill('US-123456789');
    await page.getByLabel(/Registered Billing Address/i).fill('123 Broadway, New York, NY 10001');

    // 9. Click Pay & Confirm button
    const payBtn = page.getByRole('button', { name: /Pay & Confirm/i });
    await expect(payBtn).toBeEnabled();
    await payBtn.click();

    // 10. Expect to redirect to http://localhost:5173/orders/<ORDER-UUID>
    await page.waitForURL(/\/orders\/[0-9a-fA-F-]+/, { timeout: 15000 });

    // Assert customer info & tax details on the order details card
    await expect(page.getByText(/Tax Name:\s*Alice Enterprise/i)).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(/Tax ID:\s*US-123456789/i)).toBeVisible({ timeout: 10000 });

    // Assert order item to be seat D1 with price $10.00
    await expect(page.getByText('Row D, Seat 1', { exact: true })).toBeVisible();
    await expect(page.getByText('D1', { exact: true }).first()).toBeVisible();

    // Assert net total to be $10.00
    const netTotalSection = page.locator('text=Net Total:').locator('..');
    await expect(netTotalSection.getByText('$10.00')).toBeVisible();
  });
});
