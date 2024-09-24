-- Add migration script here
CREATE TABLE subscriptions (
  id INTEGER PRIMARY KEY NOT NULL AUTO_INCREMENT,
  name VARCHAR(128) NOT NULL,
  email VARCHAR(128) NOT NULL,
  subscribed_at TIMESTAMP NOT NULL
);
