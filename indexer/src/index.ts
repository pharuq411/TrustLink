import { PrismaClient } from "@prisma/client";
import Fastify from "fastify";
import { startIndexer } from "./indexer";

const db = new PrismaClient();
const app = Fastify({ logger: true });

// GET /attestations/:subject
app.get<{ Params: { subject: string } }>(
  "/attestations/:subject",
  async (req) => {
    return db.attestation.findMany({
      where: { subject: req.params.subject },
      orderBy: { timestamp: "desc" },
    });
  }
);

// GET /attestations/issuer/:issuer
app.get<{ Params: { issuer: string } }>(
  "/attestations/issuer/:issuer",
  async (req) => {
    return db.attestation.findMany({
      where: { issuer: req.params.issuer },
      orderBy: { timestamp: "desc" },
    });
  }
);

async function main() {
  await db.$connect();
  startIndexer(db).catch((err) => {
    console.error("Indexer error:", err);
    process.exit(1);
  });
  await app.listen({ port: Number(process.env.PORT ?? 3000), host: "0.0.0.0" });
}

main();
