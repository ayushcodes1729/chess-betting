import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CapstoneChessBetting } from "../target/types/capstone_chess_betting";

describe("capstone-chess-betting", () => {
  
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.capstoneChessBetting as Program<CapstoneChessBetting>;

  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  

});
