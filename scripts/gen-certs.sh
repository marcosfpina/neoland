#!/usr/bin/env bash
# ─── Neoland mTLS Certificate Generator ──────────────────────────────────
#
# Generates a complete PKI hierarchy for end-to-end mTLS:
#   CA → Server cert (Neoland gRPC + REST)
#      → Client cert (TUI/CLI)
#      → Bridge cert (SecureLLM Bridge)
#
# Output: secrets/tls/{ca,server,client,bridge}/
#
# Usage:
#   bash scripts/gen-certs.sh                    # interactive
#   bash scripts/gen-certs.sh --auto --days 365  # non-interactive

set -euo pipefail

OUT_DIR="${OUT_DIR:-secrets/tls}"
DAYS="${DAYS:-365}"
AUTO="${AUTO:-false}"
COMMON_NAME="${COMMON_NAME:-neoland.local}"
ORG="${ORG:-VoidNxSEC}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --auto) AUTO=true ;;
        --days) DAYS="$2"; shift ;;
        --cn) COMMON_NAME="$2"; shift ;;
        --out) OUT_DIR="$2"; shift ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
    shift
done

mkdir -p "$OUT_DIR"/{ca,server,client,bridge}
cd "$OUT_DIR"

echo "🔐 Neoland mTLS Certificate Generator"
echo "   CA CN:        Neoland Root CA"
echo "   Server CN:    $COMMON_NAME"
echo "   Client CN:    neoland-client"
echo "   Bridge CN:    securellm-bridge"
echo "   Validity:     $DAYS days"
echo "   Output:       $OUT_DIR"
echo ""

# ── Step 1: Root CA ──────────────────────────────────────────────────────

if [ ! -f ca/ca.key ]; then
    echo "→ Generating Root CA..."
    openssl genrsa -out ca/ca.key 4096
    openssl req -new -x509 -days "$DAYS" -key ca/ca.key -out ca/ca.crt \
        -subj "/O=${ORG}/CN=Neoland Root CA"
    echo "  ✓ ca/ca.key + ca/ca.crt"
else
    echo "  ⏭  CA already exists, skipping"
fi

# ── Step 2: Server certificate ────────────────────────────────────────────

if [ ! -f server/server.key ]; then
    echo "→ Generating Server certificate..."
    openssl genrsa -out server/server.key 2048

    cat > server/server.cnf <<EOF
[req]
default_bits = 2048
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = req_ext

[dn]
CN = ${COMMON_NAME}
O = ${ORG}

[req_ext]
subjectAltName = @alt_names

[alt_names]
DNS.1 = ${COMMON_NAME}
DNS.2 = localhost
DNS.3 = *.neoland.local
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

    openssl req -new -key server/server.key -out server/server.csr \
        -config server/server.cnf
    openssl x509 -req -in server/server.csr -CA ca/ca.crt -CAkey ca/ca.key \
        -CAcreateserial -out server/server.crt -days "$DAYS" -sha256 \
        -extfile server/server.cnf -extensions req_ext
    rm server/server.csr server/server.cnf
    echo "  ✓ server/server.key + server/server.crt"
else
    echo "  ⏭  Server cert already exists, skipping"
fi

# ── Step 3: Client certificate (TUI/CLI) ──────────────────────────────────

if [ ! -f client/client.key ]; then
    echo "→ Generating Client certificate..."
    openssl genrsa -out client/client.key 2048
    openssl req -new -key client/client.key -out client/client.csr \
        -subj "/O=${ORG}/CN=neoland-client"
    # Extensões explícitas: openssl 3.0 sem extfile emite cert v1 pelado,
    # que o webpki/rustls rejeita no client auth (alert certificate unknown).
    cat > client/client.cnf <<EOF
basicConstraints = CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer
EOF
    openssl x509 -req -in client/client.csr -CA ca/ca.crt -CAkey ca/ca.key \
        -CAcreateserial -out client/client.crt -days "$DAYS" -sha256 \
        -extfile client/client.cnf
    rm client/client.csr client/client.cnf

    # Create PKCS#12 bundle for import into TUI/browsers
    openssl pkcs12 -export -in client/client.crt -inkey client/client.key \
        -out client/client.p12 -passout pass:neoland \
        -name "Neoland Client Certificate"

    echo "  ✓ client/client.key + client/client.crt + client/client.p12"
else
    echo "  ⏭  Client cert already exists, skipping"
fi

# ── Step 4: Bridge certificate (SecureLLM Bridge) ─────────────────────────

if [ ! -f bridge/bridge.key ]; then
    echo "→ Generating Bridge certificate..."
    openssl genrsa -out bridge/bridge.key 2048

    cat > bridge/bridge.cnf <<EOF
[req]
default_bits = 2048
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = req_ext

[dn]
CN = securellm-bridge
O = ${ORG}

[req_ext]
subjectAltName = @alt_names

[alt_names]
DNS.1 = securellm-bridge
DNS.2 = localhost
IP.1 = 127.0.0.1
EOF

    openssl req -new -key bridge/bridge.key -out bridge/bridge.csr \
        -config bridge/bridge.cnf
    openssl x509 -req -in bridge/bridge.csr -CA ca/ca.crt -CAkey ca/ca.key \
        -CAcreateserial -out bridge/bridge.crt -days "$DAYS" -sha256 \
        -extfile bridge/bridge.cnf -extensions req_ext
    rm bridge/bridge.csr bridge/bridge.cnf
    echo "  ✓ bridge/bridge.key + bridge/bridge.crt"
else
    echo "  ⏭  Bridge cert already exists, skipping"
fi

# ── Summary ───────────────────────────────────────────────────────────────

echo ""
echo "✅ mTLS PKI generated successfully!"
echo ""
echo "  CA:           $OUT_DIR/ca/ca.crt"
echo "  Server cert:  $OUT_DIR/server/server.crt"
echo "  Server key:   $OUT_DIR/server/server.key"
echo "  Client cert:  $OUT_DIR/client/client.crt"
echo "  Client key:   $OUT_DIR/client/client.key"
echo "  Client PKCS12: $OUT_DIR/client/client.p12"
echo "  Bridge cert:  $OUT_DIR/bridge/bridge.crt"
echo "  Bridge key:   $OUT_DIR/bridge/bridge.key"
echo ""
echo "  Environment variables:"
echo "    export NEOLAND_TLS_CA_CERT=$OUT_DIR/ca/ca.crt"
echo "    export NEOLAND_TLS_SERVER_CERT=$OUT_DIR/server/server.crt"
echo "    export NEOLAND_TLS_SERVER_KEY=$OUT_DIR/server/server.key"
echo "    export NEOLAND_TLS_CLIENT_CERT=$OUT_DIR/client/client.crt"
echo "    export NEOLAND_TLS_CLIENT_KEY=$OUT_DIR/client/client.key"
echo ""
echo "  Test with:"
echo "    curl --cacert $OUT_DIR/ca/ca.crt https://localhost:3001/health"
echo "    curl --cacert $OUT_DIR/ca/ca.crt --cert $OUT_DIR/client/client.crt --key $OUT_DIR/client/client.key https://localhost:3001/v1/agents/sessions"
