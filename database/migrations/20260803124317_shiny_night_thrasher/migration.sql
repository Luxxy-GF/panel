CREATE TABLE "user_email_verifications" (
	"uuid" uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	"user_uuid" uuid NOT NULL,
	"token" text NOT NULL,
	"created" timestamp DEFAULT now() NOT NULL
);

ALTER TABLE "users" ADD COLUMN "verified" boolean DEFAULT true NOT NULL;
CREATE INDEX "user_email_verifications_user_uuid_idx" ON "user_email_verifications" ("user_uuid");
CREATE UNIQUE INDEX "user_email_verifications_token_idx" ON "user_email_verifications" ("token");
ALTER TABLE "user_email_verifications" ADD CONSTRAINT "user_email_verifications_user_uuid_users_uuid_fkey" FOREIGN KEY ("user_uuid") REFERENCES "users"("uuid") ON DELETE CASCADE;