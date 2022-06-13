-- local: timestamp
-- UTC: timestamptz
create table users (
    id bigserial primary key,
    name varchar(10) UNIQUE not null,
    email varchar(256) UNIQUE not null,
    pass varchar(256) not null, -- 'passwd hash'
    create_dt timestamp not null default current_timestamp, -- 'create datetime'
    update_dt timestamp not null default current_timestamp, -- 'udpate datetime'
    status varchar(10) not null default 'normal'-- comment 'status: normal, blocked, deleted',
);
create table tags (
    id bigserial primary key,
    name varchar(32) UNIQUE not null,
    info varchar(1048576),
    post_count int8 default 0,
    create_dt timestamp not null default current_timestamp,
    update_dt timestamp not null default current_timestamp
);
-- COMMENT ON COLUMN "users"."pass" IS 'passwd hash'

-- docker run -d --restart always --name pg-demo -e POSTGRES_USER=template -e POSTGRES_DB=templatedb  -e POSTGRES_PASSWORD=MTcwNzUyNzIzMDY4Nzk2MzQ3Mjg= -p 5432:5432  postgres:14

-- libpg-dev/postgresql-devel
-- pip3 install pgcli
-- pgcli postgres://template:MTcwNzUyNzIzMDY4Nzk2MzQ3Mjg=@localhost:5432/templatedb
-- create table users
-- \db; \l+; \di; \d users;

-- DATABASE_URL="postgres://template:MTcwNzUyNzIzMDY4Nzk2MzQ3Mjg=@localhost:5432/templatedb"
