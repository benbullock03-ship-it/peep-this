---
id: WEB-14
title: Refund button on the order page
status: blocked
depends: [WEB-12]
reason: Payments API has no refund endpoint yet
wake: ticket:WEB-12
---

# WEB-14 — Refund button on the order page

- [ ] Button on `/orders/:id`, shown only for paid orders under 30 days old
- [ ] Confirm dialog, then call the refund endpoint
- [ ] Order status updates without a reload

**Done when:** a paid order can be refunded from its page and shows "Refunded".
