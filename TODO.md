# TODO

- [x] assert order/report use running number

- [x] fix can't refund

- [x] playwright tests
    - [ ] register test
    - [ ] login test
    - [x] checkout test with stripe
    - [x] view order test
    - [ ] refund test

- [x] redesign manage event page left sidebar panel

- [x] fix page at `http://localhost:5173/orders/<order-uuid>` missing left sidebar panel

- [x] move Scheduled Showtimes to new page & add link to left sidebar panel.

- [x] event 2 can edit event 1 seat map editor. should separate it.

- [x] environment variable to set currency to use in stripe.

- [x] add Dockerfile and docker compose. use cargo chef.

- [x] loadtest. simulate 1000+ users sale for big event. skip stripe payment (to prevent rate limit).

- [x] frontend remove test logins info

- [ ] seat update poll only update necessary data to prevent large payloads send from server.
    ```
    data_received: 7.5 GB in 64s (117 MB/s) is unusually high for a booking app. 
    You may be returning large payloads (seat maps? full order objects?) 
    in hot-path responses. 
    Worth checking response sizes on the high-traffic endpoints.
    ```

---

# BACKLOG

- [ ] send email use outbox with send email worker

- [ ] remove Auth Dual-Mode. use only `Authorization: Bearer` header.

---

# Phase 2:

- [ ] upgrade to event ticket selling website. custom zone & custom seats layout.

- [ ] custom seat layout. add gap between seats. don't skip number.
