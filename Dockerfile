FROM ekidd/rust-musl-builder AS build

WORKDIR /home/rust/src
COPY . .

RUN cargo build --release

FROM alpine

COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/transporter /usr/local/bin/transporter
RUN transporter --version

CMD ["transporter"]
