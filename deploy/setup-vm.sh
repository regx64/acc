#!/usr/bin/env bash
# One-time setup of the Oracle Cloud A1 VM (Ubuntu 24.04, arm64).
#
#   scp deploy/setup-vm.sh ubuntu@<vm>:~ && ssh ubuntu@<vm> 'sudo bash setup-vm.sh'
#
# Afterwards: put .env in /opt/acc (see deploy/.env.example) and let the
# Deploy workflow ship compose.yml and start the stack.
set -euo pipefail

DEPLOY_USER=${DEPLOY_USER:-ubuntu}

echo "== packages"
apt-get update
apt-get -y upgrade
apt-get install -y ca-certificates curl ufw fail2ban unattended-upgrades

echo "== automatic security updates"
dpkg-reconfigure -f noninteractive unattended-upgrades

echo "== docker"
if ! command -v docker > /dev/null; then
  curl -fsSL https://get.docker.com | sh
fi
usermod -aG docker "$DEPLOY_USER"
cat > /etc/docker/daemon.json <<'JSON'
{
  "log-driver": "local",
  "log-opts": { "max-size": "20m", "max-file": "5" }
}
JSON
systemctl restart docker

echo "== cgroup v2 (go-judge needs it)"
if [[ "$(stat -fc %T /sys/fs/cgroup)" != "cgroup2fs" ]]; then
  echo "WARNING: cgroup v2 is not active. Add systemd.unified_cgroup_hierarchy=1 to the kernel command line and reboot." >&2
fi

echo "== firewall: SSH only; the site arrives through the Cloudflare tunnel"
ufw default deny incoming
ufw default allow outgoing
ufw allow OpenSSH
ufw --force enable
# Oracle images also ship iptables rules; make sure nothing else is open.
if [[ -f /etc/iptables/rules.v4 ]]; then
  echo "Check /etc/iptables/rules.v4 and the VCN security list: only 22/tcp should be allowed in." >&2
fi

echo "== SSH: keys only"
sed -i 's/^#\?PasswordAuthentication .*/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/^#\?PermitRootLogin .*/PermitRootLogin no/' /etc/ssh/sshd_config
systemctl reload ssh || systemctl reload sshd

echo "== fail2ban for SSH"
systemctl enable --now fail2ban

echo "== app directory"
install -d -o "$DEPLOY_USER" -g "$DEPLOY_USER" -m 750 /opt/acc
echo "Done. Next: create /opt/acc/.env (chmod 600) from deploy/.env.example."
