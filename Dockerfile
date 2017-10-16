FROM archlinux/base AS build

RUN pacman -Sy --noconfirm --needed rust openssl-1.0 gcc
WORKDIR /usr/src/myapp
COPY . .

RUN OPENSSL_INCLUDE_DIR=/usr/include/openssl-1.0 OPENSSL_LIB_DIR=/usr/lib/openssl-1.0 cargo build --release

FROM archlinux/base

RUN pacman -Sy --noconfirm --needed openssl-1.0
COPY --from=build /usr/src/myapp/target/release/transporter /usr/local/bin/transporter

CMD ["transporter"]
