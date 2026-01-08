-- Add up migration script here
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS $$
  BEGIN
    new.updated_at := CURRENT_TIMESTAMP;
    return new;
  END;
$$ LANGUAGE plpgsql;
