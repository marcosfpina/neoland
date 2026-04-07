-- Merkle chain das decisões do Tech-Leader assinadas com secp256k1.
--
-- Cada linha representa um veredicto final (accepted | rejected) encadeado
-- criptograficamente. A integridade é verificada recalculando:
--   chain_hash = sha256(prev_hash || payload_hash)
-- e verificando a assinatura DER secp256k1 sobre chain_hash.
--
-- Ciclo 1 — Fase C

CREATE TABLE IF NOT EXISTS adr_merkle_chain (
    id           BIGSERIAL    PRIMARY KEY,
    -- Hash do nó anterior (32 bytes; zero para o nó gênesis)
    prev_hash    BYTEA        NOT NULL CHECK (octet_length(prev_hash) = 32),
    -- SHA-256 do payload canônico do evento
    payload_hash BYTEA        NOT NULL CHECK (octet_length(payload_hash) = 32),
    -- SHA-256(prev_hash || payload_hash)
    chain_hash   BYTEA        NOT NULL CHECK (octet_length(chain_hash) = 32),
    -- Assinatura DER secp256k1 sobre chain_hash
    signature    BYTEA        NOT NULL,
    -- Chave pública comprimida SEC1 (33 bytes)
    pubkey       BYTEA        NOT NULL CHECK (octet_length(pubkey) = 33),
    -- Timestamp de inserção
    timestamp    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Índice para busca do último nó (last_chain_hash query)
CREATE INDEX IF NOT EXISTS idx_adr_merkle_chain_id_desc
    ON adr_merkle_chain (id DESC);

-- Constraint: chain_hash deve ser único (sem bifurcações)
CREATE UNIQUE INDEX IF NOT EXISTS idx_adr_merkle_chain_unique_hash
    ON adr_merkle_chain (chain_hash);
