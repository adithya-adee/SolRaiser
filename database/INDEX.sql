CREATE INDEX idx_transactions_slot ON transactions(slot);
CREATE INDEX idx_transactions_signature ON transactions(signature);
CREATE INDEX idx_account_updates_pubkey ON account_updates(pubkey);
CREATE INDEX idx_account_updates_slot ON account_updates(slot);