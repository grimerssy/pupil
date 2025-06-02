-- Create "subjects" table
CREATE TABLE "public"."subjects" (
  "id" text NOT NULL,
  "title" text NOT NULL,
  PRIMARY KEY ("id")
);
-- Create "grades" table
CREATE TABLE "public"."grades" (
  "user_id" bigint NOT NULL,
  "subject_id" text NOT NULL,
  "value" numeric(5,2) NULL,
  CONSTRAINT "grades_subject_id_fkey" FOREIGN KEY ("subject_id") REFERENCES "public"."subjects" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT "grades_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "public"."users" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
