#!/usr/bin/env bash
# Keeps trying to create the Always Free A1 instance until Oracle has capacity.
#
# Run it in OCI Cloud Shell (already signed in; the console's >_ icon):
#
#   curl -fsSLO https://raw.githubusercontent.com/regx64/acc/claude/great-darwin-la8ip5/deploy/oracle-retry.sh
#   bash oracle-retry.sh
#
# Optional settings (environment variables):
#   OCPUS=2 MEMORY_GB=12   size (up to 4 / 24 on Always Free; smaller is easier to get)
#   BOOT_GB=100            boot volume size
#   INTERVAL=60            seconds between attempts
#   SUBNET_NAME=public-subnet  or SUBNET_ID=ocid1.subnet...
#   SSH_KEY=~/.ssh/acc_oracle.pub  public key to install (created if missing)
set -uo pipefail

OCPUS=${OCPUS:-2}
MEMORY_GB=${MEMORY_GB:-12}
BOOT_GB=${BOOT_GB:-100}
INTERVAL=${INTERVAL:-60}
NAME=${NAME:-acc}
SUBNET_NAME=${SUBNET_NAME:-public-subnet}
SSH_KEY=${SSH_KEY:-$HOME/.ssh/acc_oracle.pub}

log() { echo "[$(date '+%H:%M:%S')] $*"; }
die() { log "ERROR: $*"; exit 1; }

command -v oci > /dev/null || die "oci CLI가 없습니다. OCI Cloud Shell에서 실행하세요."

# Root compartment = tenancy.
COMPARTMENT=${COMPARTMENT:-${OCI_TENANCY:-}}
if [[ -z "$COMPARTMENT" && -f ~/.oci/config ]]; then
  COMPARTMENT=$(sed -n 's/^tenancy *= *//p' ~/.oci/config | head -1)
fi
[[ -n "$COMPARTMENT" ]] || die "compartment를 찾지 못했습니다. COMPARTMENT=ocid1.tenancy... 로 지정하세요."

AD=${AD:-$(oci iam availability-domain list --compartment-id "$COMPARTMENT" --query 'data[0].name' --raw-output)}
[[ -n "$AD" ]] || die "availability domain을 찾지 못했습니다."

SUBNET_ID=${SUBNET_ID:-$(oci network subnet list --compartment-id "$COMPARTMENT" --all \
  --query "data[?\"display-name\"=='$SUBNET_NAME'].id | [0]" --raw-output)}
[[ -n "$SUBNET_ID" && "$SUBNET_ID" != "null" ]] || die "서브넷 '$SUBNET_NAME'을 찾지 못했습니다. SUBNET_ID=... 로 지정하세요."

IMAGE_ID=${IMAGE_ID:-$(oci compute image list --compartment-id "$COMPARTMENT" \
  --operating-system "Canonical Ubuntu" --operating-system-version "24.04" \
  --shape VM.Standard.A1.Flex --sort-by TIMECREATED --sort-order DESC \
  --query 'data[0].id' --raw-output)}
[[ -n "$IMAGE_ID" && "$IMAGE_ID" != "null" ]] || die "Ubuntu 24.04 ARM 이미지를 찾지 못했습니다."

if [[ ! -f "$SSH_KEY" ]]; then
  key="${SSH_KEY%.pub}"
  mkdir -p "$(dirname "$key")"
  ssh-keygen -t ed25519 -N "" -f "$key" -C acc-oracle > /dev/null
  log "SSH 키를 새로 만들었습니다: $key (개인키) — 인스턴스가 생기면 Cloud Shell 메뉴의 Download로 받아 두세요."
fi

existing=$(oci compute instance list --compartment-id "$COMPARTMENT" --display-name "$NAME" \
  --lifecycle-state RUNNING --query 'data[0].id' --raw-output 2> /dev/null)
if [[ -n "$existing" && "$existing" != "null" ]]; then
  log "이미 실행 중인 '$NAME' 인스턴스가 있습니다: $existing"
  exit 0
fi

log "시도 설정: ${OCPUS} OCPU / ${MEMORY_GB}GB, 부트 ${BOOT_GB}GB, $AD, ${INTERVAL}초 간격"
attempt=0
while true; do
  attempt=$((attempt + 1))
  out=$(oci compute instance launch \
    --availability-domain "$AD" \
    --compartment-id "$COMPARTMENT" \
    --shape VM.Standard.A1.Flex \
    --shape-config "{\"ocpus\": $OCPUS, \"memoryInGBs\": $MEMORY_GB}" \
    --image-id "$IMAGE_ID" \
    --subnet-id "$SUBNET_ID" \
    --assign-public-ip true \
    --boot-volume-size-in-gbs "$BOOT_GB" \
    --display-name "$NAME" \
    --ssh-authorized-keys-file "$SSH_KEY" \
    --query 'data.id' --raw-output 2>&1)

  if [[ "$out" == ocid1.instance.* ]]; then
    log "성공 ($attempt번째 시도). 인스턴스: $out"
    log "RUNNING이 될 때까지 기다립니다..."
    oci compute instance get --instance-id "$out" --wait-for-state RUNNING > /dev/null 2>&1
    ip=$(oci compute instance list-vnics --instance-id "$out" --query 'data[0]."public-ip"' --raw-output)
    log "공인 IP: $ip"
    log "접속: ssh -i ${SSH_KEY%.pub} ubuntu@$ip"
    exit 0
  fi

  case "$out" in
    *"Out of host capacity"*|*"OutOfCapacity"*|*"InternalError"*)
      log "#$attempt 자리 없음. ${INTERVAL}초 뒤 다시 시도합니다." ;;
    *"TooManyRequests"*|*"429"*)
      log "#$attempt 요청이 너무 잦습니다. 5분 쉽니다."; sleep 300; continue ;;
    *"LimitExceeded"*|*"QuotaExceeded"*)
      die "무료 한도를 넘습니다. 이미 A1 인스턴스가 있거나 OCPUS/MEMORY_GB가 너무 큽니다.\n$out" ;;
    *)
      log "#$attempt 알 수 없는 오류:"; echo "$out" | tail -5 ;;
  esac
  sleep "$INTERVAL"
done
