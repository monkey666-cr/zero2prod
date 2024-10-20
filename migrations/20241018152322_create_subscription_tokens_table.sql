-- Add migration script here
CREATE TABLE subscription_tokens (
    id INTEGER PRIMARY KEY NOT NULL AUTO_INCREMENT,
    subscription_id INTEGER NOT NULL,
    token TEXT NOT NULL,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id)
);