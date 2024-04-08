
# ---------------------------------------------------------------
#
# ---------------------------------------------------------------
FROM vineiq_base AS builder
FROM bitnami/minideb:bookworm AS runner

# ---------------------------------------------------------------
#
# ---------------------------------------------------------------

RUN mkdir -p /opt/vineiq/etc /opt/vineiq/bin /opt/vineiq/scripts
WORKDIR /opt/vineiq

COPY ./scripts/entrypoint-yolink_logger.sh /opt/vineiq/scripts
COPY ./scripts/entrypoint-tempest_logger.sh /opt/vineiq/scripts

COPY --from=builder /builder/yolink_logger/target/release/yolink_logger /opt/vineiq/bin
COPY --from=builder /builder/tempest_logger/target/release/tempest_logger /opt/vineiq/bin

COPY --from=builder \
    /usr/lib/x86_64-linux-gnu/libssl.so.3 \
    /usr/lib/x86_64-linux-gnu/libcrypto.so.3 \
    /usr/lib/x86_64-linux-gnu/

