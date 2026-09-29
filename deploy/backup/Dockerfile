# Nightly Postgres dump to R2 (30-day retention) and the monthly restore test.
FROM postgres:16-bookworm
ARG MC_VERSION=RELEASE.2025-08-13T08-35-41Z
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl cron \
 && rm -rf /var/lib/apt/lists/* \
 && ARCH=$(dpkg --print-architecture) \
 && curl -sSfL -o /usr/local/bin/mc "https://dl.min.io/client/mc/release/linux-${ARCH}/archive/mc.${MC_VERSION}" \
 && chmod +x /usr/local/bin/mc
COPY backup.sh restore-test.sh entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/*.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
