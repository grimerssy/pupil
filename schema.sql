create table users (
    id bigserial primary key,
    email text not null unique,
    name text not null,
    password_hash text not null,
    role text not null check (role in ('teacher', 'student'))
);
