FROM ubuntu:14.04

RUN apt-get -qq update
RUN apt-get install -qqy curl libssl-dev
RUN curl https://static.rust-lang.org/dist/rust-1.0.0-beta.3-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /opt
RUN apt-get install -qqy gem ruby-dev build-essential
RUN gem install fpm
RUN /opt/rust-1.0.0-beta.3-x86_64-unknown-linux-gnu/install.sh
COPY deb/pack.sh /usr/local/bin/pack.sh
RUN chmod +x /usr/local/bin/pack.sh

CMD pack.sh
