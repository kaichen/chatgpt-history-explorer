-- ChatGPT History Explorer - Optimized SQLite Schema with Assets Separation
-- 优化设计：分离多媒体资产和文本内容

-- 会话表：存储对话基本信息
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,           -- ChatGPT conversation ID
    title TEXT NOT NULL,           -- 对话标题
    create_time INTEGER NOT NULL,  -- Unix timestamp
    update_time INTEGER NOT NULL,  -- Unix timestamp
    model_slug TEXT,               -- 使用的模型 (o3, gpt-4o, etc.)
    is_archived BOOLEAN DEFAULT 0  -- 是否归档
);

-- 消息表：存储消息的核心信息和文本内容
CREATE TABLE messages (
    id TEXT PRIMARY KEY,              -- Message ID
    conversation_id TEXT NOT NULL,    -- 关联的对话ID
    parent_id TEXT,                   -- 父消息ID (用于分支对话)
    author_role TEXT NOT NULL,        -- 'user' | 'assistant' | 'system' | 'tool'
    content_type TEXT NOT NULL,       -- 'text' | 'multimodal_text' | 'code' | 'thoughts' | 'user_editable_context'
    text_content TEXT,                -- 提取的纯文本内容 (multimodal时为最后的文本指令)
    create_time INTEGER,              -- Unix timestamp
    model_slug TEXT,                  -- 助手消息使用的模型
    message_order INTEGER,            -- 在对话中的顺序
    has_assets BOOLEAN DEFAULT 0,     -- 是否包含多媒体资产
    
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES messages(id) ON DELETE SET NULL
);

-- 资产表：存储多媒体文件的元数据和内容
CREATE TABLE assets (
    id TEXT PRIMARY KEY,              -- Asset ID (从 asset_pointer 提取)
    message_id TEXT NOT NULL,         -- 关联的消息ID
    asset_pointer TEXT NOT NULL,      -- 完整的 asset_pointer URL
    content_type TEXT NOT NULL,       -- 'image_asset_pointer' | 'file_asset_pointer' 等
    size_bytes INTEGER,               -- 文件大小
    width INTEGER,                    -- 图片宽度 (如果适用)
    height INTEGER,                   -- 图片高度 (如果适用)
    metadata TEXT,                    -- JSON格式的元数据
    asset_order INTEGER,              -- 在消息中的顺序
    file_content BLOB,                -- 实际的文件内容 (二进制数据)
    file_name TEXT,                   -- 原始文件名 (从zip中提取)
    mime_type TEXT,                   -- MIME类型 (从文件扩展名推断)
    
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

-- 索引优化查询性能
CREATE INDEX idx_conversations_create_time ON conversations(create_time DESC);
CREATE INDEX idx_conversations_title ON conversations(title);
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_create_time ON messages(create_time DESC);
CREATE INDEX idx_messages_content_type ON messages(content_type);
CREATE INDEX idx_messages_author_role ON messages(author_role);
CREATE INDEX idx_messages_has_assets ON messages(has_assets);
CREATE INDEX idx_assets_message_id ON assets(message_id);
CREATE INDEX idx_assets_content_type ON assets(content_type);

-- 全文搜索支持 (FTS5) - 仅索引文本内容
CREATE VIRTUAL TABLE messages_fts USING fts5(
    text_content,
    conversation_title
);

-- 触发器：同步全文搜索
CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, text_content, conversation_title)
    SELECT 
        NEW.rowid, 
        NEW.text_content,
        (SELECT title FROM conversations WHERE id = NEW.conversation_id)
    WHERE NEW.text_content IS NOT NULL AND NEW.text_content != '';
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages BEGIN
    DELETE FROM messages_fts WHERE rowid = OLD.rowid;
END;

CREATE TRIGGER messages_fts_update AFTER UPDATE ON messages BEGIN
    DELETE FROM messages_fts WHERE rowid = OLD.rowid;
    INSERT INTO messages_fts(rowid, text_content, conversation_title)
    SELECT 
        NEW.rowid, 
        NEW.text_content,
        (SELECT title FROM conversations WHERE id = NEW.conversation_id)
    WHERE NEW.text_content IS NOT NULL AND NEW.text_content != '';
END;