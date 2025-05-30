-- Modify "users" table
ALTER TABLE "public"."users" ADD CONSTRAINT "users_role_check" CHECK (role = ANY (ARRAY['teacher'::text, 'student'::text])), ADD COLUMN "role" text NULL;

UPDATE "public"."users" SET "role" = 'student';


ALTER TABLE "public"."users" ALTER COLUMN "role" SET NOT NULL;
