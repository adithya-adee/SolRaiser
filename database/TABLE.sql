CREATE TABLE IF NOT EXISTS blocks (
    slot BIGINT PRIMARY KEY,
    blockhash VARCHAR(88) NOT NULL,
    parent_slot BIGINT,
    block_time BIGINT,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    signature VARCHAR(88) UNIQUE NOT NULL,
    slot BIGINT NOT NULL,
    block_time BIGINT,
    success BOOLEAN NOT NULL,
    fee BIGINT,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (slot) REFERENCES blocks(slot)
);

CREATE TABLE IF NOT EXISTS account_updates (
    id SERIAL PRIMARY KEY,
    pubkey VARCHAR(44) NOT NULL,
    slot BIGINT NOT NULL,
    lamports BIGINT,
    owner VARCHAR(44),
    data TEXT,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
