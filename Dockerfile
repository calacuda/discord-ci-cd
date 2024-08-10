FROM rust:latest
COPY . /root/dcicd-src
WORKDIR /root/dcicd-src
RUN cargo build -r --bin dcicd-runner
RUN cp ./target/release/dcicd-runner /usr/bin/dcicd-runner
RUN ls -l /usr/bin/dcicd-runner
RUN rm -rf ./target/
WORKDIR /root/
RUN rm -rf ./dcicd-src
