CREATE TABLE public.email_tokens (
    email VARCHAR(256) NOT NULL,
    user_id INT8 NOT NULL,
    token VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    confirmed BOOL NULL DEFAULT false,
    expired BOOL NULL DEFAULT false,
    rowid INT8 NOT VISIBLE NOT NULL DEFAULT unique_rowid(),
    CONSTRAINT "primary" PRIMARY KEY (rowid ASC),
    UNIQUE INDEX email_tokens_token_key (token ASC),
    FAMILY "primary" (
        email,
        user_id,
        token,
        created_at,
        confirmed,
        expired,
        rowid
    )
);

CREATE SEQUENCE public.tags_id_seq MINVALUE 1 MAXVALUE 9223372036854775807 INCREMENT 1 START 1;

CREATE TABLE public.tags (
    id INT8 NOT NULL DEFAULT nextval('public.tags_id_seq'),
    name VARCHAR(64) NOT NULL,
    post_count INT8 NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    info VARCHAR NULL DEFAULT '',
    rowid INT8 NOT VISIBLE NOT NULL DEFAULT unique_rowid(),
    CONSTRAINT tags_pkey PRIMARY KEY (id ASC),
    FAMILY "primary" (
        id,
        name,
        post_count,
        created_at,
        updated_at,
        info,
        rowid
    )
);

CREATE SEQUENCE public.post_tags_id_seq MINVALUE 1 MAXVALUE 9223372036854775807 INCREMENT 1 START 1;

CREATE TABLE public.post_tags (
    id INT8 NOT NULL DEFAULT nextval(
        'public.post_tags_id_seq'
    ),
    post_id INT8 NOT NULL DEFAULT 0,
    tag_id INT8 NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    rowid INT8 NOT VISIBLE NOT NULL DEFAULT unique_rowid(),
    CONSTRAINT post_tags_pkey PRIMARY KEY (id ASC),
    FAMILY "primary" (
        id,
        post_id,
        tag_id,
        created_at,
        updated_at,
        rowid
    )
);

CREATE SEQUENCE public.posts_id_seq MINVALUE 1 MAXVALUE 9223372036854775807 INCREMENT 1 START 1;

CREATE TABLE public.posts (
    id INT8 NOT NULL DEFAULT nextval('public.posts_id_seq'),
    title VARCHAR(512) NULL,
    description VARCHAR(1000000) NOT NULL,
    tag_ids INT8 [] NULL,
    posted_by INT8 NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    upadted_by INT8 NOT NULL DEFAULT 0,
    visible1 BOOL NULL DEFAULT true,
    op_id INT8 NULL DEFAULT 0,
    updated_by INT8 NULL,
    votes INT8 NULL DEFAULT 0,
    reply_to INT8 NULL,
    slug VARCHAR(256) NULL DEFAULT NULL,
    views INT8 NULL DEFAULT 0,
    rowid INT8 NOT VISIBLE NOT NULL DEFAULT unique_rowid(),
    answer_accepted BOOL NULL DEFAULT false,
    answer_count INT8 NULL DEFAULT 0,
    CONSTRAINT posts_pkey PRIMARY KEY (id ASC),
    INDEX uatime_idx (updated_at ASC),
    INDEX posts_answer_count_idx (answer_count ASC),
    FAMILY "primary" (
        id,
        title,
        description,
        tag_ids,
        posted_by,
        created_at,
        updated_at,
        upadted_by,
        visible1,
        op_id,
        updated_by,
        votes,
        reply_to,
        slug,
        views,
        rowid,
        answer_accepted,
        answer_count
    )
);

CREATE SEQUENCE public.users_id_seq MINVALUE 1 MAXVALUE 9223372036854775807 INCREMENT 1 START 1;

CREATE TABLE public.users (
    id INT8 NOT NULL DEFAULT nextval('public.users_id_seq'),
    username VARCHAR(60) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    name VARCHAR NULL,
    seen_notification_id INT8 NOT NULL DEFAULT 0,
    last_posted_at TIMESTAMPTZ NULL,
    password_hash VARCHAR(256) NULL,
    salt VARCHAR(64) NULL,
    active BOOL NOT NULL DEFAULT false,
    username_lower VARCHAR(60) NOT NULL,
    last_seen_at TIMESTAMPTZ NULL,
    admin BOOL NOT NULL DEFAULT false,
    last_emailed_at TIMESTAMPTZ NULL,
    approved BOOL NOT NULL DEFAULT false,
    approved_at TIMESTAMPTZ NULL,
    previous_visit_at TIMESTAMPTZ NULL,
    suspended_at TIMESTAMPTZ NULL,
    suspended_till TIMESTAMPTZ NULL,
    date_of_birth DATE NULL,
    views INT8 NOT NULL DEFAULT 0,
    flag_level INT8 NOT NULL DEFAULT 0,
    ip_address INET NULL,
    title VARCHAR NULL,
    locale VARCHAR(10) NULL,
    registration_ip_address INET NULL,
    first_seen_at TIMESTAMPTZ NULL,
    silenced_till TIMESTAMPTZ NULL,
    email VARCHAR(255) NOT NULL,
    email_verified BOOL NULL DEFAULT false,
    email_verification_code VARCHAR(64) NULL DEFAULT '',
    designation VARCHAR NULL,
    location VARCHAR NULL,
    image_url STRING NULL DEFAULT '',
    git VARCHAR(128) NULL DEFAULT '',
    website VARCHAR(256) NULL DEFAULT '',
    twitter VARCHAR(128) NULL DEFAULT '',
    about_me VARCHAR NULL DEFAULT '',
    rowid INT8 NOT VISIBLE NOT NULL DEFAULT unique_rowid(),
    reputation INT8 NULL DEFAULT 1,
    CONSTRAINT users_pkey PRIMARY KEY (id ASC),
    FAMILY "primary" (
        id,
        username,
        created_at,
        updated_at,
        name,
        seen_notification_id,
        last_posted_at,
        password_hash,
        salt,
        active,
        username_lower,
        last_seen_at,
        admin,
        last_emailed_at,
        approved,
        approved_at,
        previous_visit_at,
        suspended_at,
        suspended_till,
        date_of_birth,
        views,
        flag_level,
        ip_address,
        title,
        locale,
        registration_ip_address,
        first_seen_at,
        silenced_till,
        email,
        email_verified,
        email_verification_code,
        designation,
        location,
        image_url,
        git,
        website,
        twitter,
        about_me,
        rowid,
        reputation
    )
);

CREATE TABLE public.votes (
    post_id INT8 NOT NULL,
    user_id INT8 NOT NULL,
    rowid INT8 NOT VISIBLE NOT NULL DEFAULT unique_rowid(),
    vote INT8 NULL DEFAULT 0,
    ts TIMESTAMPTZ NULL DEFAULT now(),
    CONSTRAINT votes_pkey PRIMARY KEY (post_id ASC, user_id ASC),
    UNIQUE INDEX votes_post_id_user_id_key (post_id ASC, user_id ASC),
    FAMILY "primary" (post_id, user_id, rowid, vote, ts)
);

ALTER TABLE
    public.votes
ADD
    CONSTRAINT votes_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id);

ALTER TABLE
    public.votes
ADD
    CONSTRAINT votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);