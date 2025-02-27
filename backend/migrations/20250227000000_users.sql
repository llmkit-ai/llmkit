PRAGMA foreign_keys = ON;

-- Create enum-like tables for roles and registration states
CREATE TABLE user_role (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE registration_state (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Insert predefined roles
INSERT INTO user_role (name) VALUES ('admin');
INSERT INTO user_role (name) VALUES ('standard');

-- Insert predefined registration states
INSERT INTO registration_state (name) VALUES ('pending');
INSERT INTO registration_state (name) VALUES ('approved');
INSERT INTO registration_state (name) VALUES ('rejected');

CREATE TABLE user (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    role_id INTEGER NOT NULL DEFAULT 2, -- Default to standard role (id=2)
    registration_state_id INTEGER NOT NULL DEFAULT 1, -- Default to pending state (id=1)
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES user_role(id),
    FOREIGN KEY (registration_state_id) REFERENCES registration_state(id)
);

-- Create a default admin user with password "admin"
-- The password_hash is pre-computed using the hash_password function
-- In production, you would generate this securely and not include a default admin
INSERT INTO user (
    email, 
    name, 
    password_hash, 
    role_id, 
    registration_state_id
) VALUES (
    'admin@llmkit.com',
    'Admin User',
    '08c9db112b347d1a87dd19a5f891ca7a9d38af418766298db9389103fb9bb7d3', -- Hash for "admin"
    1, -- admin role
    2  -- approved state
);

CREATE INDEX idx_user_email ON user(email);