<template>
  <v-container class="py-8 max-w-4xl">
    <!-- Back Button & Actions -->
    <div class="d-flex justify-space-between align-center mb-4">
      <v-btn variant="text" prepend-icon="mdi-arrow-left" to="/orders" class="text-none font-weight-bold">
        Back to Orders
      </v-btn>

      <div class="d-flex align-center gap-2">
        <!-- Download S3 PDF Receipt -->
        <v-btn v-if="hasTickets" variant="outlined" color="amber-darken-1" size="small" prepend-icon="mdi-file-pdf-box"
          class="text-none font-weight-bold" :href="`/api/orders/${order?.id}/pdf`" target="_blank">
          PDF Receipt
        </v-btn>

        <!-- Elevated Refund Action -->
        <v-btn v-if="canRefundOrder" color="red-accent-2" variant="tonal" size="small" prepend-icon="mdi-cash-refund"
          class="text-none font-weight-bold" @click="openRefundDialog()">
          Refund Tickets ({{ activeTickets.length }})
        </v-btn>
      </div>
    </div>

    <!-- Loading Skeleton -->
    <v-skeleton-loader v-if="loading" type="article, table, actions" theme="dark"
      class="rounded-xl"></v-skeleton-loader>

    <!-- Error State -->
    <v-alert v-else-if="error" type="error" variant="tonal" class="rounded-xl mb-6">
      {{ error }}
    </v-alert>

    <!-- Order Detail Card -->
    <v-card v-else-if="order" theme="dark" class="rounded-xl pa-6 border bg-surface" elevation="4">
      <!-- Order Header -->
      <div class="d-flex flex-wrap justify-space-between align-start gap-4 mb-6">
        <div>
          <div class="d-flex align-center gap-2 mb-2">
            <span class="text-caption text-grey font-weight-mono text-uppercase">Order Reference</span>
            <span class="text-caption font-weight-mono text-amber-darken-1 font-weight-bold">#{{ order.id }}</span>
          </div>
          <div class="d-flex align-center gap-5 flex-wrap mb-2">
            <h1 class="text-h4 font-weight-bold text-white mb-0 mr-5">
              {{ order.event_title }}
            </h1>
            <v-chip size="small" :color="getStatusColor(order.status)" variant="flat"
              class="font-weight-bold text-uppercase">
              {{ formatStatus(order.status) }}
            </v-chip>
          </div>
          <div class="d-flex align-center text-body-2 text-grey">
            <v-icon icon="mdi-map-marker" size="16" class="mr-1"></v-icon>
            <span>{{ order.theater_name }}</span>
            <span class="mx-2">•</span>
            <v-icon icon="mdi-calendar-clock" size="16" class="mr-1"></v-icon>
            <span>{{ formatDate(order.starts_at) }}</span>
          </div>
        </div>

        <div class="d-flex flex-column align-end">
          <span class="text-caption text-grey">Placed on {{ formatDate(order.created_at) }}</span>
        </div>
      </div>

      <v-divider class="mb-6"></v-divider>

      <!-- Customer & Organizer Info Grid -->
      <div class="row-info mb-6 pa-4 rounded-lg bg-surface-variant border">
        <v-row dense>
          <v-col cols="12" sm="6">
            <div class="text-caption text-grey font-weight-bold text-uppercase mb-1">Customer / Buyer</div>
            <div class="text-body-1 font-weight-medium text-white">{{ order.user_email || 'You' }}</div>
            <div v-if="order.billing_name" class="text-caption text-grey-lighten-1 mt-1 font-weight-bold">
              Tax Name: {{ order.billing_name }}
            </div>
            <div v-if="order.tax_id" class="text-caption font-weight-mono text-teal-accent-3">
              Tax ID: {{ order.tax_id }}
            </div>
            <div v-if="order.billing_address" class="text-caption text-grey">
              {{ order.billing_address }}
            </div>
          </v-col>
          <v-col cols="12" sm="6">
            <div class="text-caption text-grey font-weight-bold text-uppercase mb-1">Event Organizer & Tax Identity
            </div>
            <div class="text-body-2 font-weight-bold text-white">
              {{ order.organizer_tax_business_name || 'DreamCinema Live Productions Ltd.' }}
            </div>
            <div class="text-caption font-weight-mono text-grey-lighten-1">
              Tax ID: {{ order.organizer_tax_id || 'TAX-8842-TSB-2026' }}
            </div>
            <div v-if="order.organizer_tax_address" class="text-caption text-grey">
              {{ order.organizer_tax_address }}
            </div>
            <div class="text-caption font-weight-mono text-grey-lighten-2 mt-1">
              Payment Ref: {{ order.stripe_payment_intent_id || 'N/A' }}
            </div>
          </v-col>
        </v-row>
      </div>

      <!-- Seats List -->
      <div class="mb-6">
        <div class="d-flex justify-space-between align-center mb-3">
          <h2 class="text-h6 font-weight-bold text-white">Order Items ({{ order.seats?.length || 0 }})</h2>
          <div v-if="order.refunded_amount_cents > 0" class="text-caption text-red-accent-2 font-weight-bold">
            Refunded: ${{ (order.refunded_amount_cents / 100).toFixed(2) }}
          </div>
        </div>
        <v-table theme="dark" class="rounded-lg border">
          <thead>
            <tr>
              <th class="text-left font-weight-bold">Seat & Attendee</th>
              <th class="text-left font-weight-bold">Zone Tier</th>
              <th v-if="hasTickets" class="text-left font-weight-bold" style="min-width: 240px;">
                Ticket & QR Pass
              </th>
              <th class="text-right font-weight-bold">Price</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in order.seats" :key="s.seat_booking_id">
              <td class="font-weight-bold text-white py-4 align-middle">
                <div class="d-flex align-center gap-2 mb-1 flex-wrap">
                  <v-chip size="small" :color="getZoneColor(s)" variant="flat" class="font-weight-bold">
                    {{ s.row_label }}{{ s.seat_number }}
                  </v-chip>
                  <span>Row {{ s.row_label }}, Seat {{ s.seat_number }}</span>
                  <v-btn
                    v-if="['paid', 'partially_refunded'].includes(order.status) && (!hasTickets || getTicket(s.seat_booking_id)?.status === 'active')"
                    size="x-small" color="amber-darken-1" variant="tonal" prepend-icon="mdi-swap-horizontal"
                    class="text-none font-weight-bold ml-1" @click="openChangeSeatModal(s)">
                    Change Seat
                  </v-btn>
                </div>
                <!-- Upgrade / Seat Change Origin Badge -->
                <div v-if="getLatestSeatChange(s.seat_booking_id)" class="d-flex align-center gap-1 mt-1 flex-wrap">
                  <v-chip size="x-small" :color="getSeatChangeColor(getLatestSeatChange(s.seat_booking_id))"
                    variant="tonal" class="font-weight-bold"
                    :prepend-icon="getSeatChangeIcon(getLatestSeatChange(s.seat_booking_id))">
                    {{ getSeatChangeLabel(getLatestSeatChange(s.seat_booking_id)) }}
                  </v-chip>
                </div>
                <div v-if="s.attendee_name || s.attendee_phone"
                  class="text-caption text-grey-lighten-1 font-weight-regular mt-1">
                  <span v-if="s.attendee_name">👤 {{ s.attendee_name }}</span>
                  <span v-if="s.attendee_phone" class="ml-2">📞 {{ s.attendee_phone }}</span>
                </div>
              </td>
              <td class="align-middle">
                <span class="text-body-2 text-grey-lighten-1">{{ s.zone_name }}</span>
              </td>
              <td v-if="hasTickets" class="py-3 align-middle">
                <div v-if="getTicket(s.seat_booking_id)" class="d-flex align-center gap-3">
                  <!-- QR wrapper -->
                  <div class="qr-cell-wrapper pa-1 rounded bg-white elevation-2 d-flex align-center justify-center"
                    :class="{ 'opacity-40': getTicket(s.seat_booking_id).status === 'refunded' }">
                    <qrcode-vue :value="getTicket(s.seat_booking_id).code" :size="64" level="H" />
                  </div>
                  <div class="d-flex flex-column flex-grow-1">
                    <div class="d-flex align-center gap-2 mb-1">
                      <span class="text-caption text-grey text-uppercase font-weight-bold">Code</span>
                      <span class="text-subtitle-2 font-weight-black font-weight-mono letter-spacing-code"
                        :class="getTicket(s.seat_booking_id).status === 'refunded' ? 'text-grey text-decoration-line-through' : 'text-amber-darken-1'">
                        {{ getTicket(s.seat_booking_id).code }}
                      </span>
                    </div>
                    <div class="d-flex align-center gap-2 flex-wrap">
                      <v-chip size="x-small" :color="getTicketStatusColor(getTicket(s.seat_booking_id).status)"
                        variant="flat" class="font-weight-bold text-uppercase">
                        {{ getTicket(s.seat_booking_id).status }}
                      </v-chip>
                      <span v-if="getTicket(s.seat_booking_id).status === 'used'"
                        class="text-caption text-grey text-truncate">
                        Scanned {{ formatDate(getTicket(s.seat_booking_id).used_at) }}
                      </span>
                      <span v-else-if="getTicket(s.seat_booking_id).status === 'refunded'"
                        class="text-caption text-red-accent-2 text-truncate">
                        Refunded {{ formatDate(getTicket(s.seat_booking_id).refunded_at) }}
                      </span>
                      <!-- Individual ticket refund button for staff/organizers -->
                      <v-btn v-else-if="canRefundOrder && getTicket(s.seat_booking_id).status === 'active'"
                        size="x-small" color="red-accent-2" variant="text" class="text-none font-weight-bold px-1"
                        @click="openRefundDialog(getTicket(s.seat_booking_id).id)">
                        Refund
                      </v-btn>
                    </div>
                  </div>
                </div>
                <span v-else class="text-caption text-grey">Generating ticket...</span>
              </td>
              <td class="text-right font-weight-mono font-weight-bold text-white align-middle">
                <div
                  :class="{ 'text-decoration-line-through text-grey': getTicket(s.seat_booking_id)?.status === 'refunded' }">
                  ${{ (s.price_cents / 100).toFixed(2) }}
                </div>
                <div v-if="getLatestSeatChange(s.seat_booking_id)?.metadata?.is_free_upgrade"
                  class="text-caption text-purple-accent-2 font-weight-bold" title="Complimentary Upgrade">
                  <span>Free Upgrade</span>
                </div>
                <div v-else-if="s.original_price_cents && s.original_price_cents !== s.price_cents"
                  class="text-caption text-grey font-weight-regular font-mono"
                  title="Original purchase price before seat change">
                  <span class="text-grey-lighten-1">Orig: ${{ (s.original_price_cents / 100).toFixed(2) }}</span>
                </div>
              </td>
            </tr>
          </tbody>
        </v-table>
      </div>

      <!-- Refund Records / History Section -->
      <div v-if="refunds.length > 0" class="mb-6">
        <div class="d-flex justify-space-between align-center mb-3">
          <div class="d-flex align-center gap-2">
            <v-icon icon="mdi-cash-refund" color="red-accent-2" size="20"></v-icon>
            <h2 class="text-h6 font-weight-bold text-white mb-0">
              Refund Records ({{ refunds.length }})
            </h2>
          </div>
          <span class="text-caption text-red-accent-2 font-weight-mono font-weight-bold">
            Total Refunded: ${{ (order.refunded_amount_cents / 100).toFixed(2) }}
          </span>
        </div>

        <v-table theme="dark" class="rounded-lg border">
          <thead>
            <tr>
              <th class="text-left font-weight-bold">Refund ID & Time</th>
              <th class="text-left font-weight-bold">Processed By</th>
              <th class="text-left font-weight-bold">Refunded Seats & Tickets</th>
              <th class="text-left font-weight-bold">Reason / Gateway Ref</th>
              <th class="text-right font-weight-bold">Amount</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="r in refunds" :key="r.id">
              <td class="py-3 align-middle">
                <div class="font-weight-mono text-caption text-red-accent-2 font-weight-bold">
                  #{{ r.id.slice(0, 8) }}
                </div>
                <div class="text-caption text-grey">
                  {{ formatDate(r.created_at) }}
                </div>
              </td>
              <td class="align-middle">
                <div class="text-body-2 text-white">
                  {{ r.refunded_by_email || 'System' }}
                </div>
              </td>
              <td class="py-3 align-middle">
                <div v-if="r.items && r.items.length > 0" class="d-flex flex-wrap gap-1">
                  <v-chip v-for="item in r.items" :key="item.ticket_id" size="x-small" color="red-accent-2"
                    variant="tonal" class="font-weight-bold">
                    Row {{ item.row_label }}{{ item.seat_number }}
                    <span v-if="item.ticket_code" class="font-weight-mono ml-1 text-grey-lighten-1">[{{ item.ticket_code
                    }}]</span>
                    <span class="ml-1 font-weight-mono">(${{ (item.price_cents / 100).toFixed(2) }})</span>
                  </v-chip>
                </div>
                <span v-else class="text-caption text-grey">—</span>
              </td>
              <td class="py-3 align-middle">
                <div v-if="r.reason" class="text-body-2 text-white">
                  {{ r.reason }}
                </div>
                <div v-if="r.stripe_refund_id" class="text-caption text-grey font-weight-mono">
                  Ref: {{ r.stripe_refund_id }}
                </div>
                <span v-if="!r.reason && !r.stripe_refund_id" class="text-caption text-grey">—</span>
              </td>
              <td class="text-right font-weight-mono font-weight-bold text-red-accent-2 align-middle">
                -${{ (r.amount_cents / 100).toFixed(2) }}
              </td>
            </tr>
          </tbody>
        </v-table>
      </div>

      <!-- Seat Change History Section -->
      <div v-if="seatChanges.length > 0" class="mb-6">
        <div class="d-flex justify-space-between align-center mb-3">
          <div class="d-flex align-center gap-2">
            <v-icon icon="mdi-swap-horizontal-bold" color="amber-darken-1" size="20"></v-icon>
            <h2 class="text-h6 font-weight-bold text-white mb-0">
              Seat Change History ({{ seatChanges.length }})
            </h2>
          </div>
        </div>

        <v-table theme="dark" class="rounded-lg border">
          <thead>
            <tr>
              <th class="text-left font-weight-bold">Time & Processor</th>
              <th class="text-left font-weight-bold">Ticket Code</th>
              <th class="text-left font-weight-bold">Previous Seat</th>
              <th class="text-left font-weight-bold" style="width: 36px;"></th>
              <th class="text-left font-weight-bold">New Assigned Seat</th>
              <th class="text-left font-weight-bold">Change Type & Reason</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="sc in seatChanges" :key="sc.id">
              <td class="py-3 align-middle">
                <div class="text-caption text-white font-weight-bold">
                  {{ formatDate(sc.created_at) }}
                </div>
                <div class="text-caption text-grey">
                  {{ sc.changed_by_email || 'Customer Self-service' }}
                </div>
              </td>
              <td class="align-middle font-weight-mono font-weight-bold text-amber-darken-1">
                {{ sc.ticket_code || '—' }}
              </td>
              <td class="py-3 align-middle">
                <div class="d-flex align-center gap-2">
                  <v-chip size="small" color="grey-darken-1" variant="flat" class="font-weight-bold">
                    {{ sc.old_row_label }}{{ sc.old_seat_number }}
                  </v-chip>
                  <div>
                    <div class="text-caption font-weight-bold text-white">
                      Row {{ sc.old_row_label }}, Seat {{ sc.old_seat_number }}
                    </div>
                    <div class="text-caption text-grey">
                      {{ sc.old_zone_name }} · ${{ (sc.old_price_cents / 100).toFixed(2) }}
                    </div>
                  </div>
                </div>
              </td>
              <td class="text-center align-middle px-1">
                <v-icon icon="mdi-arrow-right" color="amber-darken-1" size="18"></v-icon>
              </td>
              <td class="py-3 align-middle">
                <div class="d-flex align-center gap-2">
                  <v-chip size="small" color="amber-darken-1" variant="flat" class="font-weight-bold">
                    {{ sc.new_row_label }}{{ sc.new_seat_number }}
                  </v-chip>
                  <div>
                    <div class="text-caption font-weight-bold text-amber-darken-1">
                      Row {{ sc.new_row_label }}, Seat {{ sc.new_seat_number }}
                    </div>
                    <div class="text-caption text-grey">
                      {{ sc.new_zone_name }} · ${{ (sc.new_price_cents / 100).toFixed(2) }}
                    </div>
                  </div>
                </div>
              </td>
              <td class="py-3 align-middle">
                <div class="d-flex align-center gap-1 flex-wrap mb-1">
                  <v-chip size="x-small" :color="getSeatChangeColor(sc)" variant="tonal" class="font-weight-bold"
                    :prepend-icon="getSeatChangeIcon(sc)">
                    {{ getSeatChangeLabel(sc) }}
                  </v-chip>
                </div>
                <div v-if="sc.reason" class="text-caption text-grey-lighten-1">
                  <strong>Reason:</strong> {{ sc.reason }}
                </div>
              </td>
            </tr>
          </tbody>
        </v-table>
      </div>

      <v-divider class="mb-6"></v-divider>

      <!-- Financial Summary -->
      <div class="d-flex flex-column gap-2 max-w-sm ml-auto mb-6">
        <div class="d-flex justify-space-between text-body-2 text-grey">
          <span>Seats Subtotal:</span>
          <span class="font-weight-mono text-white">${{ (order.total_amount_cents / 100).toFixed(2) }}</span>
        </div>
        <div v-if="order.refunded_amount_cents > 0"
          class="d-flex justify-space-between text-body-2 text-red-accent-2 font-weight-bold">
          <span>Total Refunded:</span>
          <span class="font-weight-mono">-${{ (order.refunded_amount_cents / 100).toFixed(2) }}</span>
        </div>
        <div class="d-flex justify-space-between text-body-2 text-grey">
          <span>Booking & Processing Fee:</span>
          <span class="text-success font-weight-bold">$0.00 (Free)</span>
        </div>
        <v-divider class="my-2"></v-divider>
        <div class="d-flex justify-space-between align-center">
          <span class="text-h6 font-weight-bold text-white">Net Total:</span>
          <span class="text-h4 font-weight-black text-amber-darken-1 font-weight-mono">
            ${{ ((order.total_amount_cents - (order.refunded_amount_cents || 0)) / 100).toFixed(2) }}
          </span>
        </div>
      </div>

      <!-- Pending Payment Action -->
      <div v-if="order.status === 'pending_payment'"
        class="pa-4 rounded-xl border bg-surface-variant d-flex justify-space-between align-center">
        <div>
          <div class="font-weight-bold text-white mb-1">Awaiting Payment</div>
          <div class="text-caption text-grey">Complete checkout to permanently confirm your reserved seats.</div>
        </div>
        <v-btn color="amber-darken-1" variant="flat" size="large" prepend-icon="mdi-credit-card-check"
          class="text-none font-weight-bold px-6" :loading="paying" @click="handlePayNow">
          Pay Now (${{ (order.total_amount_cents / 100).toFixed(2) }})
        </v-btn>
      </div>
    </v-card>

    <!-- Refund Dialog -->
    <v-dialog v-model="refundDialog.show" max-width="520" theme="dark">
      <v-card theme="dark" class="rounded-xl pa-6 border bg-surface">
        <div class="d-flex align-center gap-3 mb-4">
          <v-avatar color="red-accent-2" variant="tonal" size="44">
            <v-icon icon="mdi-cash-refund" size="24"></v-icon>
          </v-avatar>
          <div>
            <h3 class="text-h6 font-weight-bold text-white">Refund Tickets</h3>
            <span class="text-caption text-grey">Select the active tickets to refund and release back to
              inventory.</span>
          </div>
        </div>

        <v-divider class="mb-4"></v-divider>

        <!-- Ticket Selection Checkboxes -->
        <div class="mb-4">
          <div class="text-caption text-grey font-weight-bold text-uppercase mb-2">Select Tickets to Refund</div>
          <div class="d-flex flex-column gap-2 max-h-60 overflow-y-auto">
            <v-card v-for="t in activeTickets" :key="t.id" variant="outlined"
              class="pa-3 rounded-lg border bg-surface-variant cursor-pointer" @click="toggleTicketSelection(t.id)">
              <div class="d-flex justify-space-between align-center">
                <div class="d-flex align-center gap-3">
                  <v-checkbox-btn :model-value="refundDialog.selectedTicketIds.includes(t.id)" color="red-accent-2"
                    hide-details density="compact" @click.stop="toggleTicketSelection(t.id)"></v-checkbox-btn>
                  <div>
                    <div class="font-weight-bold text-white text-body-2">
                      Row {{ t.row_label }}, Seat {{ t.seat_number }}
                    </div>
                    <div class="text-caption text-grey">
                      Code: <strong class="font-weight-mono text-amber-darken-1">{{ t.code }}</strong> • {{ t.zone_name
                      }}
                    </div>
                  </div>
                </div>
                <div class="font-weight-mono font-weight-bold text-white">
                  ${{ (t.price_cents / 100).toFixed(2) }}
                </div>
              </div>
            </v-card>
          </div>
        </div>

        <!-- Refund Reason -->
        <v-text-field v-model="refundDialog.reason" label="Refund Reason (Optional)"
          placeholder="e.g., Customer requested cancellation" variant="outlined" density="compact" class="mb-4"
          hide-details></v-text-field>

        <!-- Calculated Refund Total -->
        <div class="pa-4 rounded-lg bg-surface-variant border mb-6 d-flex justify-space-between align-center">
          <span class="text-body-2 text-grey">Total Refund Amount:</span>
          <span class="text-h6 font-weight-bold text-red-accent-2 font-weight-mono">
            ${{ (calculatedRefundAmount / 100).toFixed(2) }}
          </span>
        </div>

        <div class="d-flex justify-end gap-3">
          <v-btn variant="text" class="text-none font-weight-bold" :disabled="refundDialog.submitting"
            @click="refundDialog.show = false">
            Cancel
          </v-btn>
          <v-btn color="red-accent-2" variant="flat" class="text-none font-weight-bold px-6"
            :loading="refundDialog.submitting" :disabled="refundDialog.selectedTicketIds.length === 0"
            @click="handleRefundSubmit">
            Confirm Refund ({{ refundDialog.selectedTicketIds.length }})
          </v-btn>
        </div>
      </v-card>
    </v-dialog>

    <!-- Change Seat Modal Dialog -->
    <v-dialog v-model="changeSeatDialog.show" max-width="880" theme="dark">
      <v-card theme="dark" class="rounded-xl pa-6 border bg-surface">
        <div class="d-flex justify-space-between align-center mb-4">
          <div class="d-flex align-center gap-3">
            <v-avatar color="amber-darken-1" variant="tonal" size="44">
              <v-icon icon="mdi-swap-horizontal" size="24"></v-icon>
            </v-avatar>
            <div>
              <h3 class="text-h6 font-weight-bold text-white">Change Seat Assignment</h3>
              <span class="text-caption text-grey">Select an available seat for {{ order?.event_title }}</span>
            </div>
          </div>
          <v-btn icon="mdi-close" variant="text" size="small" @click="changeSeatDialog.show = false"></v-btn>
        </div>

        <v-divider class="mb-4"></v-divider>

        <!-- Seat Comparison Card -->
        <div class="pa-4 rounded-xl bg-surface-variant border mb-4">
          <v-row dense align="center">
            <v-col cols="12" sm="5">
              <div class="text-caption text-grey font-weight-bold text-uppercase mb-1">Current Seat</div>
              <div class="d-flex align-center gap-2">
                <v-chip size="small" color="grey-darken-1" variant="flat" class="font-weight-bold">
                  {{ changeSeatDialog.oldSeat?.row_label }}{{ changeSeatDialog.oldSeat?.seat_number }}
                </v-chip>
                <span class="text-body-2 font-weight-bold text-white">
                  Row {{ changeSeatDialog.oldSeat?.row_label }}, Seat {{ changeSeatDialog.oldSeat?.seat_number }}
                </span>
                <span class="text-caption text-grey">({{ changeSeatDialog.oldSeat?.zone_name }})</span>
              </div>
            </v-col>
            <v-col cols="12" sm="2" class="text-center">
              <v-icon icon="mdi-arrow-right-bold" color="amber-darken-1" size="24"></v-icon>
            </v-col>
            <v-col cols="12" sm="5">
              <div class="text-caption text-grey font-weight-bold text-uppercase mb-1">New Selected Seat</div>
              <div v-if="changeSeatDialog.newSeat" class="d-flex align-center gap-2">
                <v-chip size="small" color="amber-darken-1" variant="flat" class="font-weight-bold">
                  {{ changeSeatDialog.newSeat.row_label }}{{ changeSeatDialog.newSeat.seat_number }}
                </v-chip>
                <span class="text-body-2 font-weight-bold text-amber-darken-1">
                  Row {{ changeSeatDialog.newSeat.row_label }}, Seat {{ changeSeatDialog.newSeat.seat_number }}
                </span>
                <span class="text-caption text-grey">({{ changeSeatDialog.newSeat.zone_name }} - ${{
                  (changeSeatDialog.newSeat.price_cents / 100).toFixed(2) }})</span>
              </div>
              <div v-else class="text-caption text-grey-lighten-1 font-italic">
                Click any available (colored) seat below to select
              </div>
            </v-col>
          </v-row>
        </div>

        <!-- Interactive Visual Seat Map Grid -->
        <div v-if="changeSeatDialog.loadingSeats" class="py-12 text-center">
          <v-progress-circular indeterminate color="amber-darken-1" size="48"></v-progress-circular>
          <div class="text-caption text-grey mt-3">Loading theater seat layout...</div>
        </div>
        <div v-else class="seat-picker-canvas pa-4 rounded-xl border bg-black mb-4 overflow-x-auto">
          <!-- Screen Banner -->
          <div class="picker-screen mb-6 mx-auto">
            <span class="picker-screen-text">STAGE / SCREEN</span>
          </div>

          <!-- Seating Grid -->
          <div class="d-flex flex-column gap-1 align-center">
            <div v-for="row in changeSeatGridRows" :key="row.label" class="d-flex align-center gap-1">
              <div class="picker-row-header">{{ row.label }}</div>
              <div class="d-flex align-center gap-1">
                <template v-for="slot in row.slots" :key="`${row.label}-${slot.seatNumber}`">
                  <!-- Seat exists -->
                  <button v-if="slot.seat" type="button" class="picker-seat-node" :class="{
                    'picker-seat-selected': changeSeatDialog.newSeat?.id === slot.seat.id,
                    'picker-seat-current': slot.seat.id === changeSeatDialog.oldSeat?.seat_id,
                    'picker-seat-unavailable': isSeatUnavailableInPicker(slot.seat),
                  }" :disabled="isSeatUnavailableInPicker(slot.seat)" :style="getPickerSeatStyle(slot.seat)"
                    :title="`Row ${slot.seat.row_label}, Seat ${slot.seat.seat_number} (${slot.seat.zone_name}) - ${slot.seat.status === 'available' ? 'Available' : 'Unavailable'}`"
                    @click="selectNewSeat(slot.seat)">
                    <span class="picker-seat-label">{{ slot.seat.seat_number }}</span>
                    <v-icon v-if="changeSeatDialog.newSeat?.id === slot.seat.id" icon="mdi-check" size="10"
                      color="white" class="picker-check"></v-icon>
                    <v-icon v-else-if="isSeatUnavailableInPicker(slot.seat)" icon="mdi-lock" size="10"
                      color="grey"></v-icon>
                  </button>
                  <!-- Empty walkway slot -->
                  <div v-else class="picker-empty-slot"></div>
                </template>
              </div>
              <div class="picker-row-header">{{ row.label }}</div>
            </div>
          </div>
        </div>

        <!-- Free Upgrade Option & Reason Field for Elevated Staff/Admin -->
        <div v-if="changeSeatDialog.newSeat" class="mb-4">
          <div
            v-if="authStore.isElevated && changeSeatDialog.newSeat.price_cents > (changeSeatDialog.oldSeat?.price_cents || 0)"
            class="pa-3 rounded-lg border bg-surface-variant mb-3">
            <div class="d-flex align-center justify-space-between flex-wrap gap-2">
              <div>
                <div class="d-flex align-center gap-2">
                  <v-icon :icon="changeSeatDialog.isFreeUpgrade ? 'mdi-gift-outline' : 'mdi-currency-usd'"
                    :color="changeSeatDialog.isFreeUpgrade ? 'purple-accent-2' : 'amber-darken-1'"></v-icon>
                  <span class="font-weight-bold text-white text-body-2">Free Upgrade / Complimentary Move</span>
                </div>
                <div class="text-caption text-grey mt-1">
                  <span v-if="changeSeatDialog.isFreeUpgrade">
                    Order total remains <strong>${{ ((changeSeatDialog.oldSeat?.price_cents || 0) / 100).toFixed(2)
                    }}</strong>.
                    No extra charge will be billed to the customer.
                  </span>
                  <span v-else>
                    Order total will be updated to <strong>${{ (changeSeatDialog.newSeat.price_cents / 100).toFixed(2)
                    }}</strong> (+${{ (((changeSeatDialog.newSeat.price_cents || 0) -
                        (changeSeatDialog.oldSeat?.price_cents
                          || 0)) / 100).toFixed(2) }} price difference).
                  </span>
                </div>
              </div>
              <v-switch v-model="changeSeatDialog.isFreeUpgrade" color="purple-accent-2" hide-details density="compact"
                inset></v-switch>
            </div>
          </div>

          <v-text-field v-model="changeSeatDialog.reason" label="Reason for Seat Change (Optional)"
            placeholder="e.g. Complimentary VIP upgrade, Operational reseating, Customer request" density="compact"
            variant="outlined" hide-details></v-text-field>
        </div>

        <div class="d-flex justify-end gap-3">
          <v-btn variant="text" class="text-none font-weight-bold" :disabled="changeSeatDialog.submitting"
            @click="changeSeatDialog.show = false">
            Cancel
          </v-btn>
          <v-btn color="amber-darken-1" variant="flat" class="text-none font-weight-bold px-6"
            :loading="changeSeatDialog.submitting" :disabled="!changeSeatDialog.newSeat"
            @click="handleConfirmChangeSeat">
            Confirm Change to Row {{ changeSeatDialog.newSeat?.row_label }}{{ changeSeatDialog.newSeat?.seat_number }}
          </v-btn>
        </div>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="snackbar.show" :color="snackbar.color" timeout="3000">
      {{ snackbar.text }}
    </v-snackbar>

    <!-- Printable Official Tax Receipt (Shown in print view) -->
    <div v-if="order" class="print-only-receipt">
      <div class="receipt-header pb-4 mb-4 border-bottom">
        <div class="d-flex justify-space-between align-start">
          <div>
            <div class="company-logo-text mb-1">
              DREAM<span class="text-amber">CINEMA</span>
            </div>
            <div class="text-small font-bold text-dark">{{ order.organizer_tax_business_name || 'DreamCinema Live Productions Ltd.' }}</div>
            <div class="text-small text-muted">Tax ID / VAT Reg: {{ order.organizer_tax_id || 'TAX-8842-TSB-2026' }}</div>
            <div v-if="order.organizer_tax_address" class="text-small text-muted">{{ order.organizer_tax_address }}</div>
            <div v-if="order.organizer_tax_email" class="text-small text-muted">{{ order.organizer_tax_email }} • {{ order.organizer_tax_phone }}</div>
          </div>
          <div class="text-right">
            <h2 class="receipt-title text-uppercase font-bold text-dark mb-1">OFFICIAL TAX RECEIPT</h2>
            <div class="font-mono text-small font-bold">Receipt No: REC-{{ order.id.substring(0, 8).toUpperCase() }}-{{ new Date(order.created_at).getFullYear() }}</div>
            <div class="text-small text-muted">Date: {{ formatDate(order.created_at) }}</div>
            <div class="text-small text-muted">Ref: {{ order.stripe_payment_intent_id || 'PAID' }}</div>
          </div>
        </div>
      </div>

      <!-- Customer & Event Specification Table -->
      <table class="receipt-table mb-4">
        <tbody>
          <tr>
            <th style="width: 25%;" class="bg-light-gray text-left">Billed To (Customer):</th>
            <td style="width: 25%;" class="font-bold">{{ order.billing_name || order.user_email || 'Direct Admissions'
            }}
            </td>
            <th style="width: 25%;" class="bg-light-gray text-left">Customer Tax ID / VAT:</th>
            <td style="width: 25%;" class="font-mono">{{ order.tax_id || 'N/A' }}</td>
          </tr>
          <tr>
            <th class="bg-light-gray text-left">Event & Show:</th>
            <td class="font-bold">{{ order.event_title }}</td>
            <th class="bg-light-gray text-left">Venue & Showtime:</th>
            <td>{{ order.theater_name }} ({{ formatDate(order.starts_at) }})</td>
          </tr>
        </tbody>
      </table>

      <!-- Items Table -->
      <table class="receipt-table mb-4">
        <thead>
          <tr>
            <th class="text-left">Seat & Zone Tier</th>
            <th class="text-left">Ticket Holder (Attendee)</th>
            <th class="text-center">Pass Code</th>
            <th class="text-right">Price (USD)</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="s in order.seats" :key="s.seat_booking_id">
            <td class="font-bold">
              Row {{ s.row_label }}, Seat {{ s.seat_number }} ({{ s.zone_name }})
              <div v-if="getLatestSeatChange(s.seat_booking_id)" class="text-muted"
                style="font-size: 11px; font-weight: normal;">
                ↳ {{ getSeatChangeLabel(getLatestSeatChange(s.seat_booking_id)) }}
              </div>
            </td>
            <td>
              <span class="font-bold">{{ s.attendee_name || order.billing_name || order.user_email }}</span>
              <span v-if="s.attendee_phone" class="text-muted ml-2">({{ s.attendee_phone }})</span>
            </td>
            <td class="text-center font-mono font-bold">{{ getTicket(s.seat_booking_id)?.code || '—' }}</td>
            <td class="text-right font-mono font-bold">
              <div>${{ (s.price_cents / 100).toFixed(2) }}</div>
              <div v-if="s.original_price_cents && s.original_price_cents !== s.price_cents"
                style="font-size: 10px; font-weight: normal; color: #666;">
                (Orig: ${{ (s.original_price_cents / 100).toFixed(2) }})
              </div>
            </td>
          </tr>
        </tbody>
        <tfoot>
          <tr>
            <td colspan="3" class="text-right font-bold">Subtotal:</td>
            <td class="text-right font-mono font-bold">${{ (order.total_amount_cents / 100).toFixed(2) }}</td>
          </tr>
          <tr v-if="order.refunded_amount_cents > 0">
            <td colspan="3" class="text-right font-bold text-danger">Refunds Deducted:</td>
            <td class="text-right font-mono font-bold text-danger">-${{ (order.refunded_amount_cents / 100).toFixed(2)
            }}
            </td>
          </tr>
          <tr class="row-total">
            <td colspan="3" class="text-right font-bold text-large">TOTAL PAID:</td>
            <td class="text-right font-mono font-bold text-large">
              ${{ ((order.total_amount_cents - (order.refunded_amount_cents || 0)) / 100).toFixed(2) }}
            </td>
          </tr>
        </tfoot>
      </table>

      <!-- Footer Stamp -->
      <div class="d-flex justify-space-between align-end pt-4 border-top">
        <div class="text-small text-muted">
          Official Electronic Tax Receipt • System Reconciled & Paid
        </div>
        <div class="text-center">
          <div class="stamp-paid">PAID & SETTLED</div>
        </div>
      </div>
    </div>
  </v-container>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import QrcodeVue from 'qrcode.vue';
import api from '../api/client';
import { useAuthStore } from '../stores/auth';

const props = defineProps({
  id: { type: String, required: true },
});

const route = useRoute();
const authStore = useAuthStore();
const order = ref(null);
const tickets = ref([]);
const refunds = ref([]);
const seatChanges = ref([]);
const loading = ref(true);
const error = ref(null);
const paying = ref(false);

const refundDialog = ref({
  show: false,
  selectedTicketIds: [],
  reason: '',
  submitting: false,
});

function printReceipt() {
  window.print();
}

const hasTickets = computed(() => {
  if (!order.value) return false;
  return ['paid', 'partially_refunded', 'refunded'].includes(order.value.status);
});

const ticketMap = computed(() => {
  const map = new Map();
  for (const t of tickets.value) {
    map.set(t.seat_booking_id, t);
  }
  return map;
});

function getTicket(seatBookingId) {
  return ticketMap.value.get(seatBookingId);
}

const activeTickets = computed(() => {
  return tickets.value.filter((t) => t.status === 'active');
});

const canRefundOrder = computed(() => {
  if (!order.value || !authStore.isElevated) return false;
  return (
    ['paid', 'partially_refunded'].includes(order.value.status) &&
    activeTickets.value.length > 0
  );
});

const calculatedRefundAmount = computed(() => {
  return refundDialog.value.selectedTicketIds.reduce((sum, tid) => {
    const t = tickets.value.find((tk) => tk.id === tid);
    return sum + (t?.price_cents || 0);
  }, 0);
});

function toggleTicketSelection(ticketId) {
  const idx = refundDialog.value.selectedTicketIds.indexOf(ticketId);
  if (idx > -1) {
    refundDialog.value.selectedTicketIds.splice(idx, 1);
  } else {
    refundDialog.value.selectedTicketIds.push(ticketId);
  }
}

function openRefundDialog(singleTicketId = null) {
  if (singleTicketId) {
    refundDialog.value.selectedTicketIds = [singleTicketId];
  } else {
    refundDialog.value.selectedTicketIds = activeTickets.value.map((t) => t.id);
  }
  refundDialog.value.reason = '';
  refundDialog.value.show = true;
}

async function handleRefundSubmit() {
  if (refundDialog.value.selectedTicketIds.length === 0) {
    showToast('Select at least one ticket to refund', 'warning');
    return;
  }

  refundDialog.value.submitting = true;
  try {
    const res = await api.post(`/orders/${order.value.id}/refund`, {
      ticket_ids: refundDialog.value.selectedTicketIds,
      reason: refundDialog.value.reason.trim() || undefined,
    });

    showToast(`Successfully refunded ${res.data.refunded_count} ticket(s)`, 'success');
    refundDialog.value.show = false;
    await fetchOrderDetail();
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to process refund', 'error');
  } finally {
    refundDialog.value.submitting = false;
  }
}

const snackbar = ref({
  show: false,
  text: '',
  color: 'info',
});

function showToast(text, color = 'info') {
  snackbar.value = { show: true, text, color };
}

function formatDate(isoStr) {
  if (!isoStr) return '—';
  const d = new Date(isoStr);
  return d.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  });
}

function getStatusColor(status) {
  switch (status) {
    case 'paid': return 'emerald-darken-1';
    case 'partially_refunded': return 'amber-darken-2';
    case 'refunded': return 'red-accent-2';
    case 'pending_payment': return 'orange-darken-2';
    case 'cancelled': return 'grey-darken-1';
    case 'expired': return 'amber-darken-2';
    default: return 'info';
  }
}

function getTicketStatusColor(status) {
  switch (status) {
    case 'active': return 'emerald-darken-1';
    case 'used': return 'blue-grey';
    case 'refunded': return 'red-accent-2';
    case 'cancelled': return 'grey-darken-1';
    case 'expired': return 'amber-darken-2';
    default: return 'info';
  }
}

function formatStatus(status) {
  if (!status) return '';
  return status.replace('_', ' ');
}

function getZoneColor(seatOrName) {
  if (!seatOrName) return '#14b8a6';
  if (typeof seatOrName === 'object') {
    if (seatOrName.zone_color) return seatOrName.zone_color;
    seatOrName = seatOrName.zone_name;
  }
  if (!seatOrName) return '#14b8a6';
  if (seatOrName.toLowerCase().includes('vip')) return '#f59e0b';
  if (seatOrName.toLowerCase().includes('premium')) return '#6366f1';
  return '#14b8a6';
}

const seatChangesByBookingId = computed(() => {
  const map = new Map();
  for (const sc of seatChanges.value) {
    if (sc.seat_booking_id) {
      if (!map.has(sc.seat_booking_id)) {
        map.set(sc.seat_booking_id, []);
      }
      map.get(sc.seat_booking_id).push(sc);
    }
  }
  return map;
});

function getSeatChanges(seatBookingId) {
  return seatChangesByBookingId.value.get(seatBookingId) || [];
}

function getLatestSeatChange(seatBookingId) {
  const list = getSeatChanges(seatBookingId);
  return list.length > 0 ? list[0] : null;
}

function getSeatChangeLabel(sc) {
  if (!sc) return '';
  if (sc.metadata?.is_free_upgrade) {
    return `🎁 Free Upgrade from Row ${sc.old_row_label}${sc.old_seat_number} (${sc.old_zone_name} · $${(sc.old_price_cents / 100).toFixed(2)})`;
  }
  const priceDiff = sc.new_price_cents - sc.old_price_cents;
  if (priceDiff > 0) {
    return `Upgraded from Row ${sc.old_row_label}${sc.old_seat_number} (${sc.old_zone_name} · $${(sc.old_price_cents / 100).toFixed(2)})`;
  } else if (priceDiff < 0) {
    return `Downgraded from Row ${sc.old_row_label}${sc.old_seat_number} (${sc.old_zone_name} · $${(sc.old_price_cents / 100).toFixed(2)})`;
  } else {
    return `Relocated from Row ${sc.old_row_label}${sc.old_seat_number} (${sc.old_zone_name})`;
  }
}

function getSeatChangeColor(sc) {
  if (!sc) return 'grey';
  if (sc.metadata?.is_free_upgrade) return 'purple-accent-2';
  const priceDiff = sc.new_price_cents - sc.old_price_cents;
  if (priceDiff > 0) return 'amber-darken-1';
  if (priceDiff < 0) return 'cyan-accent-3';
  return 'blue-grey';
}

function getSeatChangeIcon(sc) {
  if (!sc) return 'mdi-swap-horizontal';
  if (sc.metadata?.is_free_upgrade) return 'mdi-gift-outline';
  const priceDiff = sc.new_price_cents - sc.old_price_cents;
  if (priceDiff > 0) return 'mdi-arrow-up-bold-circle-outline';
  if (priceDiff < 0) return 'mdi-arrow-down-bold-circle-outline';
  return 'mdi-swap-horizontal';
}

async function fetchOrderDetail() {
  const orderId = props.id || route.params.id;
  loading.value = true;
  error.value = null;
  try {
    const res = await api.get(`/orders/${orderId}`);
    order.value = res.data;

    if (['paid', 'partially_refunded', 'refunded'].includes(order.value.status)) {
      await Promise.all([
        fetchTickets(orderId),
        fetchRefunds(orderId),
        fetchSeatChanges(orderId),
      ]);
    }
  } catch (err) {
    error.value = err.response?.data?.error || 'Failed to load order details';
    console.error('Fetch order error:', err);
  } finally {
    loading.value = false;
  }
}

async function fetchTickets(orderId) {
  try {
    const res = await api.get(`/orders/${orderId}/tickets`);
    tickets.value = res.data || [];
  } catch (err) {
    console.error('Failed to load tickets', err);
  }
}

async function fetchRefunds(orderId) {
  try {
    const res = await api.get(`/orders/${orderId}/refunds`);
    refunds.value = res.data || [];
  } catch (err) {
    console.error('Failed to load refunds', err);
  }
}

async function fetchSeatChanges(orderId) {
  try {
    const res = await api.get(`/orders/${orderId}/seat-changes`);
    seatChanges.value = res.data || [];
  } catch (err) {
    console.error('Failed to load seat changes', err);
  }
}

async function handlePayNow() {
  if (!order.value) return;
  paying.value = true;
  try {
    const res = await api.post(`/orders/${order.value.id}/checkout-session`);
    const checkoutUrl = res.data.checkout_url;
    showToast('Redirecting to secure payment...', 'info');
    window.location.href = checkoutUrl;
  } catch (err) {
    const errMsg = err.response?.data?.error || 'Payment failed';
    showToast(errMsg, 'error');
    paying.value = false;
  }
}

// Change Seat State & Logic
const changeSeatDialog = ref({
  show: false,
  oldSeat: null,
  newSeat: null,
  loadingSeats: false,
  submitting: false,
  isFreeUpgrade: true,
  reason: '',
  seats: [],
});

const changeSeatGridRows = computed(() => {
  const seats = changeSeatDialog.value.seats;
  if (!seats.length) return [];

  const map = new Map();
  let maxCol = 0;
  let maxRowIdx = 0;

  for (const s of seats) {
    map.set(`${s.row_label}-${s.seat_number}`, s);
    if (s.seat_number > maxCol) maxCol = s.seat_number;
    const rIdx = rowLabelToIndex(s.row_label);
    if (rIdx > maxRowIdx) maxRowIdx = rIdx;
  }

  const rows = [];
  for (let r = 0; r <= maxRowIdx; r++) {
    const label = indexToRowLabel(r);
    const slots = [];
    for (let c = 1; c <= maxCol; c++) {
      slots.push({
        seatNumber: c,
        seat: map.get(`${label}-${c}`) || null,
      });
    }
    if (slots.some((sl) => sl.seat)) {
      rows.push({ label, slots });
    }
  }
  return rows;
});

function rowLabelToIndex(label) {
  if (!label) return 0;
  let res = 0;
  for (let i = 0; i < label.length; i++) {
    res = res * 26 + (label.charCodeAt(i) - 64);
  }
  return res - 1;
}

function indexToRowLabel(idx) {
  let label = '';
  let n = idx;
  while (n >= 0) {
    label = String.fromCharCode((n % 26) + 65) + label;
    n = Math.floor(n / 26) - 1;
  }
  return label;
}

function isSeatUnavailableInPicker(seat) {
  if (!seat) return true;
  if (seat.id === changeSeatDialog.value.oldSeat?.seat_id) return true;
  return seat.status !== 'available';
}

function hexToRgba(hex, alpha = 0.2) {
  if (!hex || typeof hex !== 'string') return `rgba(20, 184, 166, ${alpha})`;
  let c = hex.replace('#', '').trim();
  if (c.length === 3) c = c.split('').map((x) => x + x).join('');
  const num = parseInt(c, 16);
  if (isNaN(num)) return `rgba(20, 184, 166, ${alpha})`;
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function getPickerSeatStyle(seat) {
  if (changeSeatDialog.value.newSeat?.id === seat.id) {
    return {
      borderColor: '#f59e0b',
      background: 'rgba(245, 158, 11, 0.4)',
    };
  }
  if (seat.id === changeSeatDialog.value.oldSeat?.seat_id) {
    return {
      borderColor: '#6b7280',
      background: '#374151',
    };
  }
  if (seat.status !== 'available') {
    return {
      borderColor: '#374151',
      background: '#1f2937',
    };
  }
  const color = getZoneColor(seat);
  return {
    borderColor: color,
    background: hexToRgba(color, 0.2),
  };
}

function selectNewSeat(seat) {
  if (isSeatUnavailableInPicker(seat)) return;
  changeSeatDialog.value.newSeat = seat;
}

async function openChangeSeatModal(seatItem) {
  changeSeatDialog.value = {
    show: true,
    oldSeat: seatItem,
    newSeat: null,
    loadingSeats: true,
    submitting: false,
    isFreeUpgrade: true,
    reason: '',
    seats: [],
  };

  try {
    const res = await api.get(`/showtimes/${order.value.showtime_id}/seats`);
    changeSeatDialog.value.seats = res.data || [];
  } catch (err) {
    showToast('Failed to load showtime seats', 'error');
  } finally {
    changeSeatDialog.value.loadingSeats = false;
  }
}

async function handleConfirmChangeSeat() {
  if (!changeSeatDialog.value.oldSeat || !changeSeatDialog.value.newSeat) return;

  changeSeatDialog.value.submitting = true;
  try {
    const res = await api.post(`/orders/${order.value.id}/change-seat`, {
      old_seat_booking_id: changeSeatDialog.value.oldSeat.seat_booking_id,
      new_seat_id: changeSeatDialog.value.newSeat.id,
      is_free_upgrade: changeSeatDialog.value.isFreeUpgrade,
      reason: changeSeatDialog.value.reason?.trim() || undefined,
    });

    showToast(`Seat changed to Row ${changeSeatDialog.value.newSeat.row_label}, Seat ${changeSeatDialog.value.newSeat.seat_number} successfully!`, 'success');
    changeSeatDialog.value.show = false;
    await fetchOrderDetail();
  } catch (err) {
    showToast(err.response?.data?.error || 'Failed to change seat', 'error');
  } finally {
    changeSeatDialog.value.submitting = false;
  }
}

onMounted(() => {
  fetchOrderDetail();
});
</script>

<style scoped>
.max-w-4xl {
  max-width: 900px;
}

.max-w-sm {
  max-width: 350px;
}

.bg-surface {
  background: #181c27 !important;
  border-color: #2a3142 !important;
}

.bg-surface-variant {
  background: #1e2330 !important;
  border-color: #2a3142 !important;
}

.qr-cell-wrapper {
  background: #ffffff !important;
  width: 72px;
  height: 72px;
  min-width: 72px;
}

.letter-spacing-code {
  letter-spacing: 2px;
}

.align-middle {
  vertical-align: middle !important;
}

.opacity-40 {
  opacity: 0.4;
  filter: grayscale(1);
}

.gap-2 {
  gap: 8px;
}

.gap-3 {
  gap: 12px;
}

.gap-4 {
  gap: 16px;
}

/* Seat Picker Styles */
.seat-picker-canvas {
  background: #0f131c !important;
  border-color: #2a3142 !important;
  max-height: 400px;
}

.picker-screen {
  height: 18px;
  max-width: 320px;
  background: linear-gradient(180deg, #f59e0b 0%, rgba(245, 158, 11, 0.1) 100%);
  border-radius: 50% 50% 0 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.picker-screen-text {
  font-size: 8px;
  letter-spacing: 3px;
  color: #fff;
  font-weight: 800;
  opacity: 0.8;
}

.picker-row-header {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: bold;
  color: #64748b;
}

.picker-seat-node {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px solid #4b5563;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: relative;
  transition: transform 0.1s ease, border-color 0.15s ease;
  flex-shrink: 0;
}

.picker-seat-node:hover:not(:disabled) {
  transform: scale(1.15);
  border-color: #f59e0b !important;
  z-index: 2;
}

.picker-seat-selected {
  border-color: #f59e0b !important;
  border-width: 2px !important;
  transform: scale(1.1);
  box-shadow: 0 0 8px rgba(245, 158, 11, 0.6);
  z-index: 3;
}

.picker-seat-current {
  opacity: 0.5;
  cursor: not-allowed !important;
}

.picker-seat-unavailable {
  opacity: 0.35;
  cursor: not-allowed !important;
}

.picker-seat-label {
  font-size: 9px;
  font-weight: bold;
  color: #fff;
}

.picker-check {
  position: absolute;
  top: -2px;
  right: -2px;
  background: #f59e0b;
  border-radius: 50%;
}

.picker-empty-slot {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}

/* Print Only Receipt Styles */
.print-only-receipt {
  display: none;
}

@media print {
  .print-only-receipt {
    display: block !important;
    background: #ffffff;
    color: #1e293b;
    padding: 10mm;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  }

  .v-card,
  .v-btn,
  .v-table,
  .v-divider,
  .row-info {
    display: none !important;
  }

  .receipt-header {
    border-bottom: 2px solid #cbd5e1;
  }

  .company-logo-text {
    font-size: 16pt;
    font-weight: 900;
    color: #0f172a;
    letter-spacing: -0.5px;
  }

  .text-amber {
    color: #d97706;
  }

  .text-dark {
    color: #0f172a !important;
  }

  .receipt-title {
    font-size: 13pt;
    font-weight: 800;
    letter-spacing: 0.5px;
  }

  .receipt-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 9.5pt;
  }

  .receipt-table th,
  .receipt-table td {
    border: 1px solid #cbd5e1;
    padding: 6px 10px;
  }

  .receipt-table th {
    background-color: #f1f5f9;
    font-weight: 700;
    color: #334155;
  }

  .bg-light-gray {
    background-color: #f8fafc !important;
  }

  .row-total {
    background-color: #f8fafc;
    border-top: 2px solid #0f172a;
  }

  .text-small {
    font-size: 8.5pt;
  }

  .text-large {
    font-size: 11pt;
  }

  .text-muted {
    color: #64748b;
  }

  .text-danger {
    color: #dc2626 !important;
  }

  .font-bold {
    font-weight: 700;
  }

  .font-mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .border-top {
    border-top: 1.5px solid #cbd5e1;
  }

  .stamp-paid {
    display: inline-block;
    border: 2px solid #16a34a;
    color: #16a34a;
    font-weight: 900;
    font-size: 9pt;
    padding: 3px 8px;
    border-radius: 4px;
    letter-spacing: 1px;
  }
}
</style>
