import { test, expect } from '@playwright/test';

test.describe('Admin Paid Upgrade & Order Seat Change Flow', () => {
  test('Charlie checks out D3, Admin upgrades to A3 with extra charge, verify updated total & seat history', async ({ page }) => {
    // -------------------------------------------------------------
    // Phase 1: Customer Charlie reserves and checks out Seat D3 ($10)
    // -------------------------------------------------------------

    // 1. Login as customer Charlie
    await page.goto('/login');
    await page.fill('input[type="email"]', 'charlie@customer.local');
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

    // 3. Click on Seat D3
    const seatD3 = page.locator('button[title^="Row D, Seat 3 •"]');
    await expect(seatD3).toBeVisible({ timeout: 10000 });
    await seatD3.click();

    // 4. Expect "Your Reservation" to have Seat D3
    const reservationPanel = page.locator('.v-card', { hasText: 'Your Reservation' });
    await expect(reservationPanel.getByText(/Row D,\s*Seat 3/i)).toBeVisible({ timeout: 10000 });

    // 5. Click on proceed to checkout button
    const checkoutBtn = page.getByRole('button', { name: /Proceed to Checkout/i });
    await expect(checkoutBtn).toBeEnabled({ timeout: 10000 });
    await checkoutBtn.click();

    // Wait for checkout page
    await expect(page).toHaveURL(/\/checkout\//);

    // 6. Fill attendee 1 name and phone
    const attendeeNameInput = page.getByLabel(/Attendee 1 Full Name/i);
    await expect(attendeeNameInput).toBeVisible({ timeout: 10000 });
    await attendeeNameInput.fill('Charlie');
    const attendeePhoneInput = page.getByLabel(/Attendee 1 Mobile Phone/i);
    await attendeePhoneInput.fill('+1 555-019-3333');

    // 7. Click on request tax invoice checkbox & fill info
    const taxCheckbox = page.getByLabel(/Request Tax Invoice/i);
    await taxCheckbox.check();
    await page.getByLabel(/Company \/ Individual Tax Name/i).fill('Charlie Enterprise');
    await page.getByLabel(/Tax ID \/ VAT Registration No\./i).fill('US-555666777');
    await page.getByLabel(/Registered Billing Address/i).fill('789 Fifth Ave, New York, NY 10022');

    // 8. Click Pay & Confirm button
    const payBtn = page.getByRole('button', { name: /Pay & Confirm/i });
    await expect(payBtn).toBeEnabled();
    await payBtn.click();

    // 9. Expect redirect to /orders/<ORDER-UUID> and save the URL
    await page.waitForURL(/\/orders\/[0-9a-fA-F-]+/, { timeout: 15000 });
    const orderUrl = page.url();

    // -------------------------------------------------------------
    // Phase 2: Login as Admin & Perform Paid Upgrade D3 -> A3 (VIP)
    // -------------------------------------------------------------

    // 10. Logout Charlie
    await page.getByRole('button', { name: 'Logout' }).click();
    await page.waitForURL('/login');

    // 11. Login as Admin
    await page.fill('input[type="email"]', 'admin@theater.local');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('http://localhost:5173/');

    // 12. Navigate to Charlie's order page
    await page.goto(orderUrl);
    await expect(page.getByText(/Tax Name:\s*Charlie Enterprise/i)).toBeVisible({ timeout: 10000 });

    // 13. Click on "Change Seat" on order item D3
    const changeSeatBtn = page.getByRole('button', { name: 'Change Seat' }).first();
    await expect(changeSeatBtn).toBeVisible({ timeout: 10000 });
    await changeSeatBtn.click();

    // 14. Expect Change Seat Assignment modal
    const changeSeatModal = page.locator('.v-dialog', { hasText: 'Change Seat Assignment' });
    await expect(changeSeatModal).toBeVisible({ timeout: 10000 });

    // 15. Click on seat A3 in the seat picker canvas
    const seatA3InPicker = changeSeatModal.locator('button[title^="Row A, Seat 3 ("]');
    await expect(seatA3InPicker).toBeVisible({ timeout: 10000 });
    await seatA3InPicker.click();

    // 16. Toggle OFF "Free Upgrade / Complimentary Move"
    const freeUpgradeToggle = changeSeatModal.locator('.v-switch input[type="checkbox"]');
    await freeUpgradeToggle.uncheck({ force: true });

    // Verify Paid Upgrade message (+$20.00 price difference, updated total $30.00)
    await expect(
      changeSeatModal.getByText(/Order total will be updated to.*\$30\.00.*\+\$20\.00 price difference/i)
    ).toBeVisible();

    // 17. Fill reason for seat change
    const reasonInput = changeSeatModal.getByLabel(/Reason for Seat Change/i);
    await reasonInput.fill('test paid upgrade');

    // 18. Click on confirm button
    const confirmChangeBtn = changeSeatModal.getByRole('button', { name: /Confirm Change to Row A3/i });
    await expect(confirmChangeBtn).toBeEnabled();
    await confirmChangeBtn.click();

    // -------------------------------------------------------------
    // Phase 3: Assertions after Paid Upgrade ($30.00 total)
    // -------------------------------------------------------------

    // 19. Expect modal to close
    await expect(changeSeatModal).not.toBeVisible({ timeout: 10000 });

    // 20. Expect Order Items table first row: Row A, Seat 3 (VIP) with $30.00 and Upgraded badge
    const orderItemsTable = page.locator('.v-table', { hasText: 'Seat & Attendee' });
    await expect(orderItemsTable.getByText('Row A, Seat 3', { exact: true })).toBeVisible({ timeout: 10000 });
    await expect(orderItemsTable.getByText(/Upgraded from Row D3 \(Standard · \$10\.00\)/)).toBeVisible();
    await expect(orderItemsTable.getByText('VIP', { exact: true })).toBeVisible();
    await expect(orderItemsTable.getByText('$30.00')).toBeVisible();

    // 21. Expect Seat Change History table first row
    const seatHistoryTable = page.locator('.v-table', { hasText: 'Previous Seat' });
    await expect(seatHistoryTable).toBeVisible({ timeout: 10000 });
    await expect(seatHistoryTable.getByText('admin@theater.local')).toBeVisible();
    await expect(seatHistoryTable.getByText('Row D, Seat 3', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText('Standard · $10.00', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText('Row A, Seat 3', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText('VIP · $30.00', { exact: true })).toBeVisible();
    await expect(seatHistoryTable.getByText(/Upgraded from Row D3 \(Standard · \$10\.00\)/)).toBeVisible();
    await expect(seatHistoryTable.getByText(/Reason:\s*test paid upgrade/i)).toBeVisible();

    // 22. Expect Financial Summary to be updated to $30.00
    const subtotalSection = page.locator('text=Seats Subtotal:').locator('..');
    await expect(subtotalSection.getByText('$30.00')).toBeVisible();

    const netTotalSection = page.locator('text=Net Total:').locator('..');
    await expect(netTotalSection.getByText('$30.00')).toBeVisible();
  });
});
