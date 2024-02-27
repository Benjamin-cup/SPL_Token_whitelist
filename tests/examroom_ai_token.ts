import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ExamroomAiToken } from "../target/types/examroom_ai_token";

describe("examroom_ai_token", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ExamroomAiToken as Program<ExamroomAiToken>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
