-- Seed data for Kunjika test database (kunjika_test)
--
-- Run (YSQL, e.g. via podman exec ... bin/ysqlsh -h <host> -U yugabyte -d kunjika_test):
--   ysqlsh ... -f seed_data.sql
-- or paste inline with ysqlsh -c $'...'.
--
-- Schema facts this seed must respect (verified against backend/src/questions/dao.rs):
--   * Questions are posts with op_id = 0. The `questions_count` view counts
--     posts WHERE op_id = 0, and get_questions() filters t.op_id = 0.
--   * Answers are posts with op_id = <parent question id>, title/slug NULL.
--   * posts.reply_to_id is an FK to users(id) (the user being replied to),
--     NOT to another post.
--   * answer_count lives on the question row; accept_answer() sets
--     answer_accepted = true on BOTH the question and the accepted answer.
--   * post_tags links questions to tags; tags.post_count is derived from it.

BEGIN;

-- ---------------------------------------------------------------
-- Users (explicit IDs: users_id_seq is unreliable on reused DBs)
-- ---------------------------------------------------------------
INSERT INTO users (id, username, email, name, karma, email_verified, status, image_url, created_date, modified_date) VALUES
(1000, 'shiv',   'shiv@example.com',   'Shiv',      100, true,  'normal', 'https://example.com/avatars/shiv.pn',   NOW(), NOW()),
(1001, 'arjun',  'arjun@example.com',  'Arjun',      75, true,  'normal', 'https://example.com/avatars/arjun.png',  NOW(), NOW()),
(1002, 'priya',  'priya@example.com',  'Priya',      50, true,  'normal', 'https://example.com/avatars/priya.png',  NOW(), NOW()),
(1003, 'riya',   'riya@example.com',   'Riya',       30, true,  'normal', 'https://example.com/avatars/riya.png',   NOW(), NOW()),
(1004, 'dev_k',  'dev@example.com',    'Dev Kumar',  20, false, 'normal', 'https://example.com/avatars/devk.png',   NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ---------------------------------------------------------------
-- Tags (explicit IDs; 115 = ntex, 116 = svelte)
-- ---------------------------------------------------------------
INSERT INTO tags (id, name, info, post_count, last_updated_by_id, created_date, updated_date) VALUES
(111, 'database',    'Database related questions',         0, 1000, NOW(), NOW()),
(112, 'programming', 'Programming languages and concepts', 0, 1000, NOW(), NOW()),
(113, 'web',         'Web development',                   0, 1000, NOW(), NOW()),
(114, 'rust',        'Rust programming language',         0, 1000, NOW(), NOW()),
(115, 'ntex',        'Ntex web framework',                0, 1000, NOW(), NOW()),
(116, 'svelte',      'Svelte framework',                  0, 1000, NOW(), NOW()),
(117, 'yugabytedb',  'YugabyteDB',                        0, 1000, NOW(), NOW()),
(118, 'postgres',    'PostgreSQL',                        0, 1000, NOW(), NOW()),
(119, 'security',    'Security and cryptography',         0, 1000, NOW(), NOW()),
(120, 'ops',         'Operations and DevOps',             0, 1000, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ---------------------------------------------------------------
-- Questions: op_id = 0, reply_to_id NULL, answer_count = 1 each.
-- Question 1 has an accepted answer (mirrors accept_answer()).
-- ---------------------------------------------------------------
INSERT INTO posts (id, title, description, visible, op_id, votes, slug, views, answer_accepted, answer_count, posted_by_id, updated_by_id, updated_at, pinned) VALUES
(1,  'How do we handle secrets rotation without downtime?',            'We need to rotate secrets across multiple services without causing downtime. What is the best approach?', true, 0, 24, 'how-do-we-handle-secrets-rotation-without-downtime', 1, true,  1, 1000, 1000, NOW(), false),
(2,  'When should I prefer event sourcing over CRUD?',                 'We are designing a new microservice and debating between event sourcing and traditional CRUD. What are the trade-offs?', true, 0, 18, 'when-should-i-prefer-event-sourcing-over-crud', 1, false, 1, 1000, 1000, NOW(), false),
(3,  'Best practices for database indexing in YugabyteDB?',            'We have a large dataset and need to optimize query performance. What indexing strategies work best for YugabyteDB?', true, 0, 32, 'best-practices-for-database-indexing-in-yugabytedb', 1, false, 1, 1001, 1001, NOW(), false),
(4,  'How to implement rate limiting in a microservice architecture?', 'We need to protect our APIs from abuse. What are the best approaches for rate limiting?', true, 0, 45, 'how-to-implement-rate-limiting-in-microservice-architecture', 1, false, 1, 1002, 1002, NOW(), false),
(5,  'Svelte 5 runes: what changed from Svelte 4?',                    'We are migrating from Svelte 4 to Svelte 5 and need to understand the runes system.', true, 0, 28, 'svelte-5-runes-what-changed-from-svelte-4', 1, false, 1, 1003, 1003, NOW(), false),
(6,  'Ntex vs Actix: which web framework to choose for Rust?',         'Comparing Ntex and Actix for a new project. What are the key differences?', true, 0, 15, 'ntex-vs-actix-which-web-framework-to-choose-for-rust', 1, false, 1, 1000, 1000, NOW(), false),
(7,  'How to set up CI/CD for a Rust backend with PostgreSQL?',        'We need to automate our deployment pipeline. What tools and practices work best?', true, 0, 38, 'how-to-set-up-cicd-for-a-rust-backend-with-postgresql', 1, false, 1, 1001, 1001, NOW(), false),
(8,  'Understanding SQLx compile-time query checking',                 'We are using SQLx in our Rust backend. How does the compile-time query checking work?', true, 0, 22, 'understanding-sqlx-compile-time-query-checking', 1, false, 1, 1000, 1000, NOW(), false),
(9,  'YugabyteDB vs PostgreSQL: performance comparison',               'We are evaluating databases for a new project. How do they compare in terms of performance?', true, 0, 41, 'yugabytedb-vs-postgresql-performance-comparison', 1, false, 1, 1002, 1002, NOW(), false),
(10, 'Authentication best practices in 2026',                          'What are the latest best practices for authentication in web applications?', true, 0, 52, 'authentication-best-practices-in-2026', 1, false, 1, 1003, 1003, NOW(), false)
ON CONFLICT (id) DO NOTHING;

-- ---------------------------------------------------------------
-- Answers: op_id = parent question id, title/slug NULL,
-- reply_to_id = users(id) (FK to users, not posts),
-- answer_count = 0 on the answer row itself.
-- ---------------------------------------------------------------
INSERT INTO posts (id, title, description, visible, op_id, votes, slug, views, answer_accepted, answer_count, posted_by_id, updated_by_id, updated_at, reply_to_id) VALUES
(201, NULL, 'Vault dynamic secrets approach allows automatic rotation without downtime. Services get short-lived credentials they renew automatically.', true, 1,  24, NULL, 1, true,  0, 1001, 1001, NOW(), 1000),
(202, NULL, 'Event sourcing is great for audit trails and temporal queries, but adds complexity. Use it when you need temporal queries or distributed transactions.', true, 2,  18, NULL, 1, false, 0, 1002, 1002, NOW(), 1000),
(203, NULL, 'YugabyteDB supports both B-tree and hash indexes. B-tree is generally the best choice for most workloads.', true, 3,  32, NULL, 1, false, 0, 1000, 1000, NOW(), 1001),
(204, NULL, 'The token bucket algorithm is simple and effective. Redis+Lua scripts can implement it atomically.', true, 4,  45, NULL, 1, false, 0, 1003, 1003, NOW(), 1000),
(205, NULL, 'Runes are the core of Svelte 5. The $state rune replaces reactive declarations, $derived replaces computed properties, and run callbacks replace lifecycle methods.', true, 5,  28, NULL, 1, false, 0, 1004, 1004, NOW(), 1000),
(206, NULL, 'Both are great. Ntex is modern and actively maintained. Actix is mature but has had breaking changes. Choose based on ecosystem and team familiarity.', true, 6,  15, NULL, 1, false, 0, 1000, 1000, NOW(), 1002),
(207, NULL, 'GitHub Actions is easy to set up. Use cache for cargo, matrix for testing across versions, and artifacts for deployments.', true, 7,  38, NULL, 1, false, 0, 1001, 1001, NOW(), 1002),
(208, NULL, 'SQLx uses tokio-postgres connections. Configure pool size based on your workload. Too many connections can overwhelm the database.', true, 8,  22, NULL, 1, false, 0, 1002, 1002, NOW(), 1003),
(209, NULL, 'YugabyteDB supports vector indexes for similarity search. Use HNSW algorithm for approximate nearest neighbor search.', true, 9,  41, NULL, 1, false, 0, 1003, 1003, NOW(), 1004),
(210, NULL, 'OAuth 2.0 is the standard for authorization. OpenID Connect adds identity layer. Use libraries instead of rolling your own.', true, 10, 52, NULL, 1, false, 0, 1004, 1004, NOW(), 1003)
ON CONFLICT (id) DO NOTHING;

-- ---------------------------------------------------------------
-- Question <-> tag mapping (21 links), then derive post_count.
-- post_tags has no unique constraint, so ON CONFLICT cannot dedupe;
-- delete the seeded questions' links first to stay idempotent.
-- ---------------------------------------------------------------
DELETE FROM post_tags WHERE post_id BETWEEN 1 AND 10;
INSERT INTO post_tags (post_id, tag_id) VALUES
(1, 119), (1, 120),   -- secrets rotation -> security, ops
(2, 112), (2, 111),   -- event sourcing -> programming, database
(3, 111), (3, 118),   -- db indexing -> database, postgres
(4, 112), (4, 113),   -- rate limiting -> programming, web
(5, 116), (5, 113),   -- svelte 5 runes -> svelte, web
(6, 115), (6, 114),   -- ntex vs actix -> ntex, rust
(7, 120), (7, 114),   -- ci/cd -> ops, rust
(8, 114), (8, 118),   -- sqlx -> rust, postgres
(9, 117), (9, 118), (9, 111),  -- yugabytedb vs postgres -> yugabytedb, postgres, database
(10, 119), (10, 113)  -- auth best practices -> security, web
ON CONFLICT DO NOTHING;

UPDATE tags SET post_count = (SELECT COUNT(*) FROM post_tags pt WHERE pt.tag_id = tags.id);

-- ---------------------------------------------------------------
-- Reset sequences so future inserts don't collide with explicit IDs
-- ---------------------------------------------------------------
SELECT setval(pg_get_serial_sequence('users', 'id'), GREATEST((SELECT MAX(id) FROM users), 1000));
SELECT setval(pg_get_serial_sequence('posts', 'id'), GREATEST((SELECT MAX(id) FROM posts), 1));
SELECT setval(pg_get_serial_sequence('tags',  'id'), GREATEST((SELECT MAX(id) FROM tags), 1));

COMMIT;
