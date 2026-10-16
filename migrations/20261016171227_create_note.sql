-- example — remove when building your service
--
-- Demo table mirroring the Fastify template's `note` model (Prisma migration
-- 20260315150045_init). Divergence, kept deliberately: Prisma declared a
-- `template` schema, but the committed TS migration was unqualified, so this
-- table lives in the default `public` schema exactly like the TS template's.
CREATE TABLE "note" (
    "id" SERIAL PRIMARY KEY,
    "owner" TEXT NOT NULL,
    "note" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL
);
