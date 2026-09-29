#!/bin/sh
set -e

# Validate environment variable
if [ -z "$STRIPE_API_KEY" ]; then
  echo "Error: STRIPE_API_KEY environment variable is not set."
  exit 1
fi

# Start forwarding events to apiserver headlessly
echo "Forwarding Stripe events to $FORWARD_TO..."
exec stripe listen --api-key "$STRIPE_API_KEY" --all-snapshot --forward-to "$FORWARD_TO" --skip-update
