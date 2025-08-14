import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CapstoneChessBetting } from "../target/types/capstone_chess_betting";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";

describe("capstone-chess-betting", () => {
  
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.capstoneChessBetting as Program<CapstoneChessBetting>;

  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let winner: PublicKey | null = null;

  let seed = new anchor.BN(1);
  let bet_amount = new anchor.BN(LAMPORTS_PER_SOL * 0.01); // 0.01 SOL
  let match_duration = new anchor.BN(60);
  let playerA = Keypair.generate();
  let playerB = Keypair.generate(); 

});
