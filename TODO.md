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

- [ ] loadtest. simulate 1000+ users sale for big event. skip stripe payment (to prevent rate limit).

- [x] frontend remove test logins info

---

# BACKLOG

- [ ] send email use outbox with send email worker

- [ ] remove Auth Dual-Mode. use only `Authorization: Bearer` header.
