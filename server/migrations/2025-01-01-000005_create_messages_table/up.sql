CREATE TABLE messages (
    id UUID PRIMARY KEY NOT NULL,
    chat_id UUID NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB,
    parent_message_id UUID,
    sequence_number INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    tokens_used INTEGER,
    model_used TEXT,
    CONSTRAINT fk_messages_chat_id FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    CONSTRAINT fk_messages_parent_message_id FOREIGN KEY (parent_message_id) REFERENCES messages(id) ON DELETE SET NULL
);

CREATE INDEX idx_messages_chat_id_sequence ON messages(chat_id, sequence_number);
CREATE INDEX idx_messages_parent_message_id ON messages(parent_message_id);









