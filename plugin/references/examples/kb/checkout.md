# checkout

## How it works now

- Cart lives in `src/checkout/cart.ts`; totals are computed on the server, never in the browser.
- Payment goes through `src/api/payments/charge.ts`, one call per order.

## Watch out

- Prices are integer pence. Formatting happens only in `formatPrice`.
- The charge call is not idempotent: retry only on network errors.

## Why it's like this

- Client totals drifted from server totals with discounts (WEB-08), so the server owns them.

last touched: WEB-11, 2026-11-15
