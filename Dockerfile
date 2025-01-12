ARG IMPLEMENTATION=java

# Build stage for Java implementation
FROM maven:3.9.6-amazoncorretto-21 AS java-builder
WORKDIR /app
ADD pom.xml .
RUN mvn clean verify --fail-never
COPY . .
RUN mvn clean package

# Build stage for Rust implementation
FROM rust:1.72 AS rust-builder
WORKDIR /app
COPY rust/degressly-core .
RUN cargo build --release

# Final stage - Java
FROM amazoncorretto:21-alpine AS java-runtime
WORKDIR /app
COPY --from=java-builder /app/target/core-0.0.1-SNAPSHOT.jar .
ARG diff_publisher_bootstrap_servers=false
ARG diff_publisher_topic_name=diff_stream
ARG primary_host=http://host.docker.internal:9000
ARG secondary_host=http://host.docker.internal:9001
ARG candidate_host=http://host.docker.internal:9002
ARG return_response_from=PRIMARY
ENV diff_publisher_bootstrap_servers=${diff_publisher_bootstrap_servers} \
    diff_publisher_topic_name=${diff_publisher_topic_name} \
    primary_host=$primary_host \
    secondary_host=$secondary_host \
    candidate_host=$candidate_host \
    return_response_from=$return_response_from
EXPOSE 8000
ENTRYPOINT ["java", "-jar", \
    "-Ddiff.publisher.bootstrap-servers=${diff_publisher_bootstrap_servers}", \
    "-Ddiff.publisher.topic-name=${diff_publisher_topic_name}", \
    "-Dprimary.host=${primary_host}", \
    "-Dsecondary.host=${secondary_host}", \
    "-Dcandidate.host=${candidate_host}", \
    "-Dreturn.response.from=${return_response_from}", \
    "core-0.0.1-SNAPSHOT.jar"]

# Final stage - Rust
FROM debian:stable-slim AS rust-runtime
RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /app/target/release/degressly-core-rust /usr/local/bin/
COPY --from=rust-builder /app/config /etc/degressly-core/config
ENV RUN_MODE=production \
    APP_CONFIG_DIR=/etc/degressly-core/config
EXPOSE 8000
CMD ["degressly-core-rust"]

# Select final image based on IMPLEMENTATION arg
FROM ${IMPLEMENTATION}-runtime AS final
