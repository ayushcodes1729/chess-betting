import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CapstoneChessBetting } from "../target/types/capstone_chess_betting";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import authWallet from '../auth-wallet.json'; // Import the auth wallet
import { assert, expect } from "chai";



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

  let seed: anchor.BN;
  let betAmount: anchor.BN;
  let playerA: anchor.web3.Keypair;
  let playerB: anchor.web3.Keypair;



  describe("A complete match", () => {
    before(async () => {
      seed = new anchor.BN(1);
      betAmount = new anchor.BN(LAMPORTS_PER_SOL * 1);
      playerA = Keypair.generate();
      playerB = Keypair.generate();
      // Airdrop SOL to players

      let airdrop1 = await connection.requestAirdrop(playerA.publicKey, LAMPORTS_PER_SOL * 5);
      let airdrop2 = await connection.requestAirdrop(playerB.publicKey, LAMPORTS_PER_SOL * 5);
      await connection.confirmTransaction(airdrop1);
      await connection.confirmTransaction(airdrop2);
      code = "ayush123";

      // Create match account
      [matchAccount, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from("match"), seed.toArrayLike(Buffer, "le", 8), Buffer.from(code), playerA.publicKey.toBuffer()],
        program.programId
      );

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

    it("Accept Match", async () => {
      const tx = await program.methods
        .acceptMatch(code)
        .accountsPartial({
          playerB: playerB.publicKey,
          playerA: playerA.publicKey,
          matchAccount: matchAccount,
          vault: vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerB])
        .rpc();
      const matchData = await program.account.matchState.fetch(matchAccount);
      const vaultAccount = await connection.getAccountInfo(vault);
      matchStatus = matchData.status;
      assert.ok(matchData.playerB.equals(playerB.publicKey));
      assert.ok("inProgress" in matchStatus);
      assert.equal(vaultAccount.lamports, 2 * betAmount.toNumber(), "Vault should contain the total bet amount of both the players here");
      assert.ok(matchData.playerA.equals(playerA.publicKey));
    });

    it("Final Payouts", async () => {
      winner = playerA.publicKey; // Simulating player A as the winner
      const initialBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const initialTreasuryBalance = await connection.getBalance(treasuryPda);
      const vaultAccount = await connection.getAccountInfo(vault);
      const vaultRent = await connection.getMinimumBalanceForRentExemption(vaultAccount.data.length);
      const tx = await program.methods
        .finalPayouts(code, winner)
        .accountsPartial({
          authority: authority.publicKey,
          playerA: playerA.publicKey,
          playerB: playerB.publicKey,
          matchAccount: matchAccount,
          vault: vault,
          systemProgram: SystemProgram.programId,
          treasuryPda: treasuryPda,
          config: config,
        })
        .signers([authority])
        .rpc();
      try {
        await program.account.matchState.fetch(matchAccount);
      } catch (error) {
        expect(error.message).to.include("Account does not exist");

      }

      try {
        await connection.getAccountInfo(vault);
      } catch (error) {
        expect(error.message).to.include("Account does not exist");

      }

      const totalBetAmount = betAmount.mul(new anchor.BN(2));
      const winningAmount = totalBetAmount.sub(totalBetAmount.div(new anchor.BN(200))); // 0.5% fee deducted
      const finalBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const feeAmount = totalBetAmount.div(new anchor.BN(200));

      const treasuryAccount = await connection.getAccountInfo(treasuryPda);
      const playerAInfo = await connection.getAccountInfo(playerA.publicKey);
      assert.ok(treasuryAccount, "Treasury account should exist");
      assert.ok(playerAInfo, "Player A account should exist");
      assert.equal(finalBalancePlayerA, initialBalancePlayerA + winningAmount.toNumber(), "Player A should receive the winning amount");
      assert.ok(treasuryAccount.lamports > initialTreasuryBalance + feeAmount.toNumber(), "Treasury should have at least receive the fee amount");
    })

    it("Withdraw from Treasury", async () => {
      const initialTreasuryBalance = await connection.getBalance(treasuryPda);
      const treasuryAccount = await connection.getAccountInfo(treasuryPda);
      const authorityInfo = await connection.getAccountInfo(authority.publicKey);
      const authorityInitialBalance = authorityInfo ? authorityInfo.lamports : 0;
      const treasuryRent = await connection.getMinimumBalanceForRentExemption(treasuryAccount.data.length);
      const tx = await program.methods
        .withdrawFromTreasury()
        .accountsPartial({
          authority: authority.publicKey,
          treasuryPda: treasuryPda,
          config: config,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();
      const finalTreasuryBalance = await connection.getBalance(treasuryPda);
      const transferAmount = initialTreasuryBalance - finalTreasuryBalance;
      const authorityFinalBalance = await connection.getBalance(authority.publicKey);
      assert.ok(
        Math.abs(authorityFinalBalance - (authorityInitialBalance + transferAmount)) < 10_000, // 10k lamports tolerance (~0.00001 SOL)
        `Authority did not receive expected funds. Expected ≈${transferAmount}, got ${authorityFinalBalance}`
      );
      assert.ok(finalTreasuryBalance >= treasuryRent, "Treasury balance should decrease");
    })
  })

  describe("A chess match with Draw Result", () => {
    before(async () => {
      seed = new anchor.BN(2);
      betAmount = new anchor.BN(LAMPORTS_PER_SOL * 1);
      playerA = Keypair.generate();
      playerB = Keypair.generate();
      // Airdrop SOL to players

      let airdrop1 = await connection.requestAirdrop(playerA.publicKey, LAMPORTS_PER_SOL * 5);
      let airdrop2 = await connection.requestAirdrop(playerB.publicKey, LAMPORTS_PER_SOL * 5);
      await connection.confirmTransaction(airdrop1);
      await connection.confirmTransaction(airdrop2);
      code = "draw123";

      // Create match account
      [matchAccount, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from("match"), seed.toArrayLike(Buffer, "le", 8), Buffer.from(code), playerA.publicKey.toBuffer()],
        program.programId
      );

      // Create vault account
      [vault, vaultBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), matchAccount.toBuffer()],
        program.programId
      );

    });

    it("Initialize Match for Draw", async () => {
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

    it("Accept Match for Draw", async () => {
      const tx = await program.methods
        .acceptMatch(code)
        .accountsPartial({
          playerB: playerB.publicKey,
          playerA: playerA.publicKey,
          matchAccount: matchAccount,
          vault: vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerB])
        .rpc();
      const matchData = await program.account.matchState.fetch(matchAccount);
      const vaultAccount = await connection.getAccountInfo(vault);
      matchStatus = matchData.status;
      assert.ok(matchData.playerB.equals(playerB.publicKey));
      assert.ok("inProgress" in matchStatus);
      assert.equal(vaultAccount.lamports, 2 * betAmount.toNumber(), "Vault should contain the total bet amount of both the players here");
      assert.ok(matchData.playerA.equals(playerA.publicKey));
    });
    it("Final Payouts for Draw", async () => {
      winner = null; // Simulating a draw
      const initialBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const initialBalancePlayerB = await connection.getBalance(playerB.publicKey);
      const initialTreasuryBalance = await connection.getBalance(treasuryPda);
      const vaultAccount = await connection.getAccountInfo(vault);
      const vaultRent = await connection.getMinimumBalanceForRentExemption(vaultAccount.data.length);

      const tx = await program.methods
        .finalPayouts(code, winner)
        .accountsPartial({
          authority: authority.publicKey,
          playerA: playerA.publicKey,
          playerB: playerB.publicKey,
          matchAccount: matchAccount,
          vault: vault,
          systemProgram: SystemProgram.programId,
          treasuryPda: treasuryPda,
          config: config,
        })
        .signers([authority])
        .rpc();

      try {
        await program.account.matchState.fetch(matchAccount);
      } catch (error) {
        expect(error.message).to.include("Account does not exist");
      }

      try {
        await connection.getAccountInfo(vault);
      } catch (error) {
        expect(error.message).to.include("Account does not exist");
      }

      // Each player’s refund = betAmount - 1% of betAmount
      const refundAmount = betAmount.sub(betAmount.div(new anchor.BN(100))); // 99% of bet
      const feeAmount = betAmount.div(new anchor.BN(100)); // 1% per player
      const totalFee = feeAmount.mul(new anchor.BN(2));   // Treasury gets 2% total

      const finalBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const finalBalancePlayerB = await connection.getBalance(playerB.publicKey);

      const treasuryAccount = await connection.getAccountInfo(treasuryPda);
      const playerAInfo = await connection.getAccountInfo(playerA.publicKey);
      const playerBInfo = await connection.getAccountInfo(playerB.publicKey);

      assert.ok(treasuryAccount, "Treasury account should exist");
      assert.ok(playerAInfo, "Player A account should exist");
      assert.ok(playerBInfo, "Player B account should exist");

      // Fix asserts to match program logic
      assert.equal(
        finalBalancePlayerA,
        initialBalancePlayerA + refundAmount.toNumber(),
        "Player A should receive their bet minus 1%"
      );
      assert.equal(
        finalBalancePlayerB,
        initialBalancePlayerB + refundAmount.toNumber(),
        "Player B should receive their bet minus 1%"
      );
      assert.ok(
        treasuryAccount.lamports >= initialTreasuryBalance + totalFee.toNumber(),
        "Treasury should have received both players' 1% fees"
      );
    });
  });

  describe("A cancel Match in waiting state", () => {
    before(async () => {
      seed = new anchor.BN(3);
      betAmount = new anchor.BN(LAMPORTS_PER_SOL * 1);
      playerA = Keypair.generate();
      playerB = Keypair.generate();
      // Airdrop SOL to players

      let airdrop1 = await connection.requestAirdrop(playerA.publicKey, LAMPORTS_PER_SOL * 5);
      let airdrop2 = await connection.requestAirdrop(playerB.publicKey, LAMPORTS_PER_SOL * 5);
      await connection.confirmTransaction(airdrop1);
      await connection.confirmTransaction(airdrop2);
      code = "cancel123";

      // Create match account
      [matchAccount, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from("match"), seed.toArrayLike(Buffer, "le", 8), Buffer.from(code), playerA.publicKey.toBuffer()],
        program.programId
      );

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

    });

    it("Initialize Match for Cancel", async () => {
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

    it("Cancel Match", async () => {
      const initialBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const initialBalancePlayerB = await connection.getBalance(playerB.publicKey);

      const tx = await program.methods
        .cancelMatch(code)
        .accountsPartial({
          playerA: playerA.publicKey,
          playerB: playerB.publicKey,
          config: config,
          player: playerA.publicKey,
          treasuryPda: treasuryPda,
          matchAccount: matchAccount,
          vault: vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerA])
        .rpc();

      const finalBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const finalBalancePlayerB = await connection.getBalance(playerB.publicKey);

      assert.equal(finalBalancePlayerA, initialBalancePlayerA + betAmount.toNumber(), "Player A should receive their bet back");
      assert.equal(finalBalancePlayerB, initialBalancePlayerB, "Player B should not receive any funds");
    });


  });

  describe("A cancel Match in inProgress state", () => {
    before(async () => {
      seed = new anchor.BN(4);
      betAmount = new anchor.BN(LAMPORTS_PER_SOL * 1);
      playerA = Keypair.generate();
      playerB = Keypair.generate();
      // Airdrop SOL to players

      let airdrop1 = await connection.requestAirdrop(playerA.publicKey, LAMPORTS_PER_SOL * 5);
      let airdrop2 = await connection.requestAirdrop(playerB.publicKey, LAMPORTS_PER_SOL * 5);
      await connection.confirmTransaction(airdrop1);
      await connection.confirmTransaction(airdrop2);
      code = "cancelInProgress123";

      // Create match account
      [matchAccount, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from("match"), seed.toArrayLike(Buffer, "le", 8), Buffer.from(code), playerA.publicKey.toBuffer()],
        program.programId
      );

      // Create vault account
      [vault, vaultBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), matchAccount.toBuffer()],
        program.programId
      );

    });

    it("Initialize Match for Cancel in Progress", async () => {
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

    it("Accept Match for Cancel in Progress", async () => {
      const tx = await program.methods
        .acceptMatch(code)
        .accountsPartial({
          playerB: playerB.publicKey,
          playerA: playerA.publicKey,
          matchAccount: matchAccount,
          vault: vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerB])
        .rpc();
      const matchData = await program.account.matchState.fetch(matchAccount);
      const vaultAccount = await connection.getAccountInfo(vault);
      matchStatus = matchData.status;
      assert.ok(matchData.playerB.equals(playerB.publicKey));
      assert.ok("inProgress" in matchStatus);
      assert.equal(vaultAccount.lamports, 2 * betAmount.toNumber(), "Vault should contain the total bet amount of both the players here");
      assert.ok(matchData.playerA.equals(playerA.publicKey));
    });

    it("Cancel Match in Progress", async () => {
      const initialBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const initialBalancePlayerB = await connection.getBalance(playerB.publicKey);

      const tx = await program.methods
        .cancelMatch(code)
        .accountsPartial({
          playerA: playerA.publicKey,
          playerB: playerB.publicKey,
          config: config,
          player: playerA.publicKey,
          treasuryPda: treasuryPda,
          matchAccount: matchAccount,
          vault: vault,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerA])
        .rpc();

      const finalBalancePlayerA = await connection.getBalance(playerA.publicKey);
      const finalBalancePlayerB = await connection.getBalance(playerB.publicKey);

      assert.ok(finalBalancePlayerA < initialBalancePlayerA + betAmount.toNumber(), "Player A should receive their bet back - some fee deducted");
      assert.ok(finalBalancePlayerB > initialBalancePlayerB, "Player B should receive their bet back");
    });
  });

});
