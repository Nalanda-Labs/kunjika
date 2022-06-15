-- local: timestamp
-- UTC: timestamptz
create table users (
    id bigserial primary key,
    username varchar(10) UNIQUE not null,
    email varchar(256) UNIQUE not null,
    pass varchar(256) not null, -- 'passwd hash'
    image_url varchar(256) not null,
    email_verified boolean default false,
    create_dt timestamp with time zone not null default current_timestamp, -- 'create datetime'
    update_dt timestamp with time zone not null default current_timestamp, -- 'udpate datetime'
    status varchar(10) not null default 'normal'-- comment 'status: normal, blocked, deleted',
);
create table tags (
    id bigserial primary key,
    name varchar(32) UNIQUE not null,
    info varchar(1048576),
    post_count int8 default 0,
    create_dt timestamp with time zone not null default current_timestamp,
    update_dt timestamp with time zone not null default current_timestamp
);

CREATE TABLE posts (
    id bigserial primary key,
    title character varying(256),
    description character varying(1000000) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    visible boolean DEFAULT true NOT NULL,
    op_id bigint default 0,
    votes bigint DEFAULT 0 NOT NULL,
    slug character varying(256),
    views bigint DEFAULT 0 NOT NULL,
    answer_accepted boolean DEFAULT false NOT NULL,
    answer_count bigint DEFAULT 0 NOT NULL,
    posted_by_id bigint NOT NULL,
    reply_to_id bigint,
    updated_by_id bigint NOT NULL
);

CREATE TABLE public.post_tags (
    id bigserial NOT NULL,
    post_id bigint NOT NULL,
    tag_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_posted_by_id_fkey FOREIGN KEY (posted_by_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_reply_to_id_fkey FOREIGN KEY (reply_to_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_updated_by_id_fkey FOREIGN KEY (updated_by_id) REFERENCES public.users(id) ON DELETE CASCADE;
