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

- [ ] event 2 can edit event 1 seat map editor. should separate it.

- [ ] send email use outbox with send email worker

- [ ] frontend remove test logins info

- [ ] use THB instead of USD

- [ ] loadtest. simulate 1000+ users sale for big event. 
      skip stripe payment to prevent rate limit.
