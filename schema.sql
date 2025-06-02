create table users (
    id bigserial primary key,
    email text not null unique,
    name text not null,
    password_hash text not null,
    role text not null check (role in ('teacher', 'student'))
);

create table subjects (
    id text primary key,
    title text not null
);

create table grades (
    user_id bigint not null references users (id) on delete cascade,
    subject_id text not null references subjects (id) on delete cascade,
    value numeric(5, 2),
    primary key (user_id, subject_id)
);
