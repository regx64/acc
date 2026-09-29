-- acc initial schema. Every record hangs off users.id; handles are display only.

CREATE TABLE users (
    id                BIGSERIAL PRIMARY KEY,
    -- NULL after account deletion.
    handle            TEXT UNIQUE,
    email             TEXT UNIQUE,
    pw_hash           TEXT,
    role              TEXT NOT NULL DEFAULT 'USER' CHECK (role IN ('USER', 'ADMIN')),
    email_verified    BOOLEAN NOT NULL DEFAULT FALSE,
    suspended         BOOLEAN NOT NULL DEFAULT FALSE,
    suspend_reason    TEXT,
    deleted           BOOLEAN NOT NULL DEFAULT FALSE,
    rating            INTEGER NOT NULL DEFAULT 0,
    tier              SMALLINT NOT NULL DEFAULT 0,
    default_language  TEXT,
    handle_changed_at TIMESTAMPTZ,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX users_rating_idx ON users (rating DESC, id) WHERE NOT deleted;

-- Handles freed by a change (locked 180 days, redirecting) or deletion (forever).
CREATE TABLE reserved_handles (
    handle  TEXT PRIMARY KEY,
    -- Redirect target while the lock lasts; NULL for deleted accounts.
    user_id BIGINT REFERENCES users(id),
    until   TIMESTAMPTZ NOT NULL
);

CREATE SEQUENCE problems_id_seq START WITH 1000;
CREATE TABLE problems (
    id                 INTEGER PRIMARY KEY DEFAULT nextval('problems_id_seq'),
    title              TEXT NOT NULL,
    status             TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'REVIEW', 'PUBLIC')),
    author_id          BIGINT NOT NULL REFERENCES users(id),
    source             TEXT NOT NULL CHECK (source IN ('OFFICIAL', 'USER')),
    time_limit_ms      INTEGER NOT NULL CHECK (time_limit_ms BETWEEN 500 AND 10000),
    memory_limit_kb    INTEGER NOT NULL CHECK (memory_limit_kb BETWEEN 32768 AND 1048576),
    level              SMALLINT NOT NULL DEFAULT 0 CHECK (level BETWEEN 0 AND 30),
    proposed_level     SMALLINT CHECK (proposed_level BETWEEN 0 AND 30),
    -- {"legend","input","output","hint","samples":[{"input","output"}]}
    statement          JSONB NOT NULL,
    -- Reference solution, required before review.
    solution_language  TEXT,
    solution_code      TEXT,
    -- Result of the last reference-solution check.
    validation_status  TEXT CHECK (validation_status IN ('PENDING', 'PASSED', 'FAILED')),
    validation_message TEXT,
    validated_at       TIMESTAMPTZ,
    -- Bumped whenever the test data changes, so worker caches invalidate.
    testcase_version   INTEGER NOT NULL DEFAULT 0,
    solved_count       INTEGER NOT NULL DEFAULT 0,
    submission_count   INTEGER NOT NULL DEFAULT 0,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at       TIMESTAMPTZ
);
ALTER SEQUENCE problems_id_seq OWNED BY problems.id;
CREATE INDEX problems_public_idx ON problems (status, id);
CREATE INDEX problems_author_idx ON problems (author_id);

-- Test data lives in object storage; the DB keeps keys only.
CREATE TABLE testcases (
    id          BIGSERIAL PRIMARY KEY,
    problem_id  INTEGER NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    idx         INTEGER NOT NULL,
    input_key   TEXT NOT NULL,
    output_key  TEXT NOT NULL,
    input_size  BIGINT NOT NULL,
    output_size BIGINT NOT NULL,
    is_sample   BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (problem_id, idx)
);

CREATE TABLE submissions (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES users(id),
    problem_id  INTEGER NOT NULL REFERENCES problems(id),
    language    TEXT NOT NULL,
    code        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'PENDING'
                CHECK (status IN ('PENDING', 'JUDGING', 'AC', 'WA', 'TLE', 'MLE', 'RE', 'CE', 'SE')),
    time_ms     INTEGER,
    memory_kb   INTEGER,
    -- Compile output (CE), shown to the author only.
    message     TEXT,
    code_length INTEGER NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    judged_at   TIMESTAMPTZ
);
CREATE INDEX submissions_problem_idx ON submissions (problem_id, id DESC);
CREATE INDEX submissions_user_idx ON submissions (user_id, id DESC);
CREATE INDEX submissions_user_problem_idx ON submissions (user_id, problem_id, status);

CREATE TABLE user_problem (
    user_id     BIGINT NOT NULL REFERENCES users(id),
    problem_id  INTEGER NOT NULL REFERENCES problems(id),
    state       TEXT NOT NULL CHECK (state IN ('TRIED', 'SOLVED')),
    first_ac_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, problem_id)
);
CREATE INDEX user_problem_problem_idx ON user_problem (problem_id, state);

CREATE TABLE problem_reviews (
    id          BIGSERIAL PRIMARY KEY,
    problem_id  INTEGER NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    reviewer_id BIGINT REFERENCES users(id),
    decision    TEXT NOT NULL CHECK (decision IN ('SUBMITTED', 'APPROVED', 'REJECTED')),
    comment     TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX problem_reviews_problem_idx ON problem_reviews (problem_id, id);

-- Per-problem board.
CREATE TABLE posts (
    id            BIGSERIAL PRIMARY KEY,
    problem_id    INTEGER NOT NULL REFERENCES problems(id),
    user_id       BIGINT NOT NULL REFERENCES users(id),
    kind          TEXT NOT NULL CHECK (kind IN ('QUESTION', 'COUNTEREXAMPLE', 'TYPO')),
    title         TEXT NOT NULL,
    body          TEXT NOT NULL,
    -- Attached code, shown only to users who solved the problem.
    code          TEXT,
    code_language TEXT,
    hidden        BOOLEAN NOT NULL DEFAULT FALSE,
    -- Set on typo reports once an admin has handled them.
    resolved      BOOLEAN NOT NULL DEFAULT FALSE,
    comment_count INTEGER NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ
);
CREATE INDEX posts_problem_idx ON posts (problem_id, id DESC);

CREATE TABLE comments (
    id         BIGSERIAL PRIMARY KEY,
    post_id    BIGINT NOT NULL REFERENCES posts(id),
    user_id    BIGINT NOT NULL REFERENCES users(id),
    body       TEXT NOT NULL,
    code       TEXT,
    code_language TEXT,
    hidden     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);
CREATE INDEX comments_post_idx ON comments (post_id, id);

CREATE TABLE reports (
    id          BIGSERIAL PRIMARY KEY,
    reporter_id BIGINT NOT NULL REFERENCES users(id),
    target_type TEXT NOT NULL CHECK (target_type IN ('POST', 'COMMENT')),
    target_id   BIGINT NOT NULL,
    reason      TEXT NOT NULL,
    resolved    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (reporter_id, target_type, target_id)
);
CREATE INDEX reports_open_idx ON reports (resolved, id DESC);

-- Supporting tables.

CREATE TABLE sessions (
    -- sha256 of the cookie token.
    id         TEXT PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX sessions_user_idx ON sessions (user_id);

CREATE TABLE email_tokens (
    -- sha256 of the token sent by mail.
    id         TEXT PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users(id),
    kind       TEXT NOT NULL CHECK (kind IN ('VERIFY', 'RESET')),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at    TIMESTAMPTZ
);

-- Key/value site settings, e.g. the maintenance banner.
CREATE TABLE site_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
