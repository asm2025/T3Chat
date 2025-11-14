CREATE TABLE chats (
    id UUID PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    model_provider TEXT NOT NULL,
    model_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT fk_chats_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_chats_user_id ON chats(user_id);
CREATE INDEX idx_chats_deleted_at ON chats(deleted_at);
CREATE INDEX idx_chats_updated_at ON chats(updated_at);























