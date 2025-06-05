-- Create "keys" table
CREATE TABLE "public"."keys" (
  "value" uuid NOT NULL,
  "user_id" bigint NOT NULL,
  PRIMARY KEY ("value"),
  CONSTRAINT "keys_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
