locals {
  indented_sql = "{{ sql . \"  \" }}"
}

env "local" {
  name = atlas.env
  src  = "file://schema.sql"
  url  = getenv("DATABASE_URL")
  dev  = getenv("DEV_DATABASE_URL")
  migration {
    dir = "file://migrations"
  }
  format {
    migrate {
      diff = local.indented_sql
    }
  }
}
