#!/bin/bash
# Runs the schedule in cron. Times are UTC: 18:30 UTC is 03:30 KST.
set -euo pipefail
env | grep -E '^(PG|S3_|BACKUP_|ALERT_)' | sed 's/^/export /; s/=\(.*\)$/="\1"/' > /etc/acc-backup.env
cat > /etc/cron.d/acc-backup <<CRON
30 18 * * * root . /etc/acc-backup.env; /usr/local/bin/backup.sh >> /proc/1/fd/1 2>&1
0 19 1 * * root . /etc/acc-backup.env; /usr/local/bin/restore-test.sh >> /proc/1/fd/1 2>&1
CRON
chmod 0644 /etc/cron.d/acc-backup
if [[ "${1:-}" == "now" ]]; then exec /usr/local/bin/backup.sh; fi
if [[ "${1:-}" == "restore-test" ]]; then exec /usr/local/bin/restore-test.sh; fi
echo "acc-backup: cron started"
exec cron -f
