import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CapstoneChessBetting } from "../target/types/capstone_chess_betting";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import authWallet from '../auth-wallet.json'; // Import the auth wallet
import { assert } from "chai";



describe("capstone-chess-betting", () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.capstoneChessBetting as Program<CapstoneChessBetting>;

  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let winner: PublicKey | null = null;
  let bump: number;
  let vaultBump: number;
  let matchStatus;
  let matchAccount: PublicKey;
  let vault: PublicKey;
  let matchDuration: number;
  let code: string;

  const authority = Keypair.fromSecretKey(Uint8Array.from(authWallet));
  let treasuryPda: PublicKey;
  let config: PublicKey;
  let configBump: number;
  let treasuryBump: number;

  let seed = new anchor.BN(1);
  let betAmount = new anchor.BN(LAMPORTS_PER_SOL * 1); // 1 SOL bet amount
  let playerA = Keypair.generate();
  let playerB = Keypair.generate();

  before(async () => {
    // Airdrop SOL to players

    let airdrop1 = await connection.requestAirdrop(playerA.publicKey, LAMPORTS_PER_SOL * 5);
    let airdrop2 = await connection.requestAirdrop(playerB.publicKey, LAMPORTS_PER_SOL * 5);
    await connection.confirmTransaction(airdrop1);
    await connection.confirmTransaction(airdrop2);
    console.log("Airdrop 1:", airdrop1);
    console.log("Airdrop 2:", airdrop2);
    code = "ayush123"; 

    // Create match account
    [matchAccount, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("match"), seed.toArrayLike(Buffer, "le", 8), Buffer.from(code), playerA.publicKey.toBuffer()],
      program.programId
    );

    console.log("Match Account:", matchAccount.toBase58());

    // Create vault account
    [vault, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), matchAccount.toBuffer()],
      program.programId
    );

    // Create treasury PDA
    [treasuryPda, treasuryBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury")],
      program.programId
    );

    // Create config PDA
    [config, configBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
    // console.log("Vault:", vault.toBase58());

    // Initialize the config
    await program.methods
      .initConfig()
      .accountsPartial({
        authority: authority.publicKey,
        config: config,
        treasuryPda: treasuryPda,
      })
      .signers([authority])
      .rpc();

    // console.log("Config initialized with authority:", authority.publicKey.toBase58());
  });

  it("Initialize Match", async () => {
    matchDuration = 60;
    const tx = await program.methods
      .initializeMatch(seed, code, matchDuration, betAmount, null, null)
      .accountsPartial({
        playerA: playerA.publicKey,
        matchAccount: matchAccount,
        vault: vault,
        systemProgram: SystemProgram.programId,
      })
      .signers([playerA])
      .rpc();
    console.log("Initialize Match Transaction Signature:", tx);
    const matchData = await program.account.matchState.fetch(matchAccount);
    const vaultAccount = await connection.getAccountInfo(vault);
    matchStatus = matchData.status;
    assert.ok(matchData.matchDuration === matchDuration);
    assert.ok(matchData.betAmount.eq(betAmount));
    assert.ok(matchData.playerA.equals(playerA.publicKey));
    assert.ok(matchData.playerB === null);
    assert.ok("waiting" in matchStatus);
    assert.equal(vaultAccount.lamports, betAmount.toNumber(), "Vault should contain the total bet amount here");
  });
});
