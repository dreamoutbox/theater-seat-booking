#!/bin/sh
set -e

# Validate environment variable
if [ -z "$STRIPE_API_KEY" ]; then
  echo "Error: STRIPE_API_KEY environment variable is not set."
  exit 1
fi

# Login using the provided API key
echo "Logging in to Stripe CLI..."
stripe login --api-key "$STRIPE_API_KEY"

# Start forwarding events to apiserver
echo "Forwarding Stripe events to $FORWARD_TO"
exec stripe listen --forward-to $FORWARD_TO
