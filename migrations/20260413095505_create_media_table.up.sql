create type media_type as enum ('sticker', 'gif');


create table if not exists "media" (
    "id" uuid not null primary key,
    "name" text not null unique,
    "file_id" text not null,
    "media_type" media_type not null,
    "added_by" bigint,
    "created_at" timestamp with time zone not null default current_timestamp,
    "updated_at" timestamp with time zone not null default current_timestamp
);

create table if not exists "media_user_usage" (
    "media_id" uuid not null references "media" (id) on delete cascade,
    "user_id" bigint not null,
    "usage_count" integer not null default 0,
    "created_at" timestamp with time zone not null default current_timestamp,
    "updated_at" timestamp with time zone not null default current_timestamp,
    primary key (media_id, user_id)
);
