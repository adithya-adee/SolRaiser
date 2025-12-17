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

CREATE TABLE IF NOT EXISTS campaign_events (
    id SERIAL PRIMARY KEY,
    signature VARCHAR(88) NOT NULL,
    slot BIGINT NOT NULL,
    event_type VARCHAR(20) NOT NULL,
    campaign_id BIGINT NOT NULL,
    user_pubkey VARCHAR(44) NOT NULL,
    amount BIGINT,
    goal_amount BIGINT,
    deadline BIGINT,
    metadata_url TEXT,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (signature) REFERENCES transactions(signature) ON DELETE CASCADE,
    FOREIGN KEY (slot) REFERENCES blocks(slot) ON DELETE CASCADE
);