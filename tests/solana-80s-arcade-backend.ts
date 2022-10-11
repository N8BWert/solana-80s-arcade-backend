import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Solana80sArcadeBackend } from "../target/types/solana_80s_arcade_backend";

describe("solana-80s-arcade-backend", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Solana80sArcadeBackend as Program<Solana80sArcadeBackend>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
