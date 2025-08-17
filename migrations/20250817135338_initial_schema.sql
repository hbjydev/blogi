create table actors (
  did text not null primary key,
  handle text not null unique,
  display_name text not null,
  description text,
  created_at timestamptz not null,
  indexed_at timestamptz not null default now()
);
