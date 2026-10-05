FROM grafana/k6:latest

WORKDIR /loadtest

COPY big-event-sale.js /loadtest/big-event-sale.js
COPY test-data.json /loadtest/test-data.json

ENTRYPOINT ["k6", "run", "/loadtest/big-event-sale.js"]
