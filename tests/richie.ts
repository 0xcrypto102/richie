import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Richie } from "../target/types/richie";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Keypair, SYSVAR_RENT_PUBKEY, Transaction, sendAndConfirmTransaction, ComputeBudgetProgram } from "@solana/web3.js";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { createSyncNativeInstruction, getAccount, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, initializeTransferHookInstructionData, NATIVE_MINT, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Buffer } from "buffer";

describe("richie", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Richie as Program<Richie>;
  const owner = anchor.Wallet.local().payer;
  console.log(bs58.encode(owner.secretKey))

  let user1 = Keypair.fromSecretKey(bs58.decode("3TqtgMohnJo9tqa5y534jftCRiZZ6XTQxAAYt5oMyWCy7gqLc2o1YabegDKnnEU8eoFy6CsTVMe4BrY2y6ksbFq3"));
  let user2 = Keypair.fromSecretKey(bs58.decode("3DKVvTnRE5xegB8zg16woZDvv8zRJYgkLu1kVnv5Em5ev9LrA96tKZv5v6JXk1ET98KRJTy5FFApiE7RH54Zpjj4"));

  const [config] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );
  const [stakeVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    program.programId
  );
  const [rewardVault] = PublicKey.findProgramAddressSync(
    [Buffer.from("reward")],
    program.programId
  );
  const [stakes] = PublicKey.findProgramAddressSync(
    [Buffer.from("stake")],
    program.programId
  );
  const stakeTokenMint = new PublicKey("37TEpUD1tDgnA5o7iNT66doHoeS7sX4doCW9zahBXxqH");
  const rewardTokenMint = new PublicKey("4T28UVwGqgVwvZDgtwLqJ5HbdgzUWcFzAAgrxW9pPsXZ");
  it("Is initialized stake vault!", async () => {
    try {
      const aprBps = 10;
      const epochDuration = 14 * 3600 * 6; // 6 hours
      const initStakeVaultTx = await program.rpc.initializeStakeVault(
        new anchor.BN(aprBps),
        new anchor.BN(epochDuration), {
          accounts: {
            config,
            admin: owner.publicKey,
            stakeTokenMint,
            stakeVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [owner]
        }
      );
      console.log("initStakeVaultTx->", initStakeVaultTx);
    } catch (error) {
      console.log("error", error);
    }
  });
  it("Is initialized reward vault!", async () => {
    try {
      const initRewardVault = await program.rpc.initializeRewardVault(
        {
          accounts: {
            config,
            admin: owner.publicKey,
            rewardMint: rewardTokenMint,
            rewardVault,
            stakes,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [owner]
        }
      );
      console.log("initRewardVault->", initRewardVault);
    } catch (error) {
      console.log("error", error);
    }
  });
  it("Create the pre staking", async() => {
    try {
      const index = new anchor.BN(0);
      const rewardAmount = 0; // 100 tokens as reward

      const [epoch] = PublicKey.findProgramAddressSync(
        [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const rewardMintTokenAccount = getAssociatedTokenAddressSync(
        rewardTokenMint,
        owner.publicKey
      );

      const tx = await program.rpc.toggle(
        index,
        new anchor.BN(rewardAmount), {
          accounts: {
            owner: owner.publicKey,
            config,
            epoch,
            rewardMint: rewardTokenMint,
            rewardMintTokenAccount,
            rewardVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [owner]
        }
      );
      console.log("tx->", tx)
    } catch (error) {
      console.log("error->", error);
    }
  });
  it("stake in pre-staking", async() => {
    try {
      const index = new anchor.BN(0);
      const amount = 20 * 10 ** 9;
      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user1.publicKey.toBuffer()],
        program.programId
      );
      const fromTokenAccount = getAssociatedTokenAddressSync(
        stakeTokenMint,
        user1.publicKey
      );
      const [epoch] = PublicKey.findProgramAddressSync(
        [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const lockPeriod = 1;
      console.log(await program.account.config.fetch(config));
      const tx = await program.rpc.stake(
        index,
        new anchor.BN(amount), 
        lockPeriod, {
          accounts: {
            user: user1.publicKey,
            config,
            stakeTokenMint,
            userStake,
            fromTokenAccount,
            stakeVault,
            epoch,
            stakes,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [user1]
        }
      );
      console.log("tx->", tx);
      const userStakeInfo = await program.account.userStake.fetch(userStake);
      console.log("userStakeInfo->", userStakeInfo);
    } catch (error) {
      console.log("error:", error);
    }
  });

  it("Call Manage rewards before start first epoch", async() => {
    try {
      const users = await program.account.userStake.all();
      for(let i = 0; i<users.length; i++) {
        const user = users[i];
        const index = new anchor.BN(0);
        const userStake = user.publicKey
     
        const [epoch] = PublicKey.findProgramAddressSync(
          [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
          program.programId
        );

        const tx = await program.rpc.manageStakerReward(
          index,
          {
            accounts: {
              admin: owner.publicKey,
              config,
              epoch,
              user: user.account.owner,
              userStake,
              stakes,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId
            },
            signers: [owner]
          }
        );
        console.log("tx->", tx);
      }
    } catch (error) {
      console.log("error:", error);
    }
  });
  it("Create the first epoch", async() => {
    try {
      const index = new anchor.BN(2);
      const rewardAmount = 100 * 10 ** 9; // 100 tokens as reward

      const [epoch] = PublicKey.findProgramAddressSync(
        [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const rewardMintTokenAccount = getAssociatedTokenAddressSync(
        rewardTokenMint,
        owner.publicKey
      );

      const tx = await program.rpc.toggle(
        index,
        new anchor.BN(rewardAmount), {
          accounts: {
            owner: owner.publicKey,
            config,
            epoch,
            rewardMint: rewardTokenMint,
            rewardMintTokenAccount,
            rewardVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [owner]
        }
      );
      console.log("tx->", tx)
    } catch (error) {
      console.log("error->", error);
    }
  });
  /*
  it("stake in epoch1", async() => {
    try {
      const index = new anchor.BN(1);
      const amount = 20 * 10 ** 9;
      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user1.publicKey.toBuffer()],
        program.programId
      );
      const fromTokenAccount = getAssociatedTokenAddressSync(
        stakeTokenMint,
        user1.publicKey
      );
      const [epoch] = PublicKey.findProgramAddressSync(
        [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const lockPeriod = 4;
      const tx = await program.rpc.stake(
        index,
        new anchor.BN(amount), 
        lockPeriod, {
          accounts: {
            user: user1.publicKey,
            config,
            stakeTokenMint,
            userStake,
            fromTokenAccount,
            stakeVault,
            epoch,
            stakes,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [user1]
        }
      );
      console.log("tx->", tx);
      const userStakeInfo = await program.account.userStake.fetch(userStake);
      console.log("userStakeInfo->", userStakeInfo);
    } catch (error) {
      console.log("error:", error);
    }
  });

  it("stake in epoch1", async() => {
    try {
      const index = new anchor.BN(1);
      const amount = 10 * 10 ** 9;
      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user2.publicKey.toBuffer()],
        program.programId
      );
      const fromTokenAccount = getAssociatedTokenAddressSync(
        stakeTokenMint,
        user2.publicKey
      );
      const [epoch] = PublicKey.findProgramAddressSync(
        [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const lockPeriod = 1;

      const tx = await program.rpc.stake(
        index,
        new anchor.BN(amount), 
        lockPeriod, {
          accounts: {
            user: user2.publicKey,
            config,
            stakeTokenMint,
            userStake,
            fromTokenAccount,
            stakeVault,
            epoch,
            stakes,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
          },
          signers: [user2]
        }
      );
      console.log("tx->", tx);
      const userStakeInfo = await program.account.userStake.fetch(userStake);
      console.log("userStakeInfo->", userStakeInfo);

      const epochInfo = await program.account.epoch.fetch(epoch);
      console.log("epochInfo->", epochInfo);
    } catch (error) {
      console.log("error:", error);
    }
  });
  */
  /*
  it("user 1 withdraw token", async() => {
    try {
      const index = new anchor.BN(1);
      
      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user1.publicKey.toBuffer()],
        program.programId
      );
      const [epoch] = PublicKey.findProgramAddressSync(
        [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const userStakeData = await program.account.userStake.fetch(userStake);
      console.log("userStakeData->", userStakeData);
      const toTokenAccount = getAssociatedTokenAddressSync(
        stakeTokenMint,
        user1.publicKey
      );

      const tx = await program.rpc.withdraw(
        new anchor.BN(1), {
          accounts: {
            user: user1.publicKey,
            config,
            epoch,
            userStake,
            stakeTokenMint,
            stakeVault,
            toTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID
          },
          signers: [user1]
        }
      );
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  */
  /*
  it("Manage rewards", async() => {
    try {
      const users = await program.account.userStake.all();
      for(let i = 0; i<users.length; i++) {
        const user = users[i];
        const index = new anchor.BN(1);
        const userStake = user.publicKey
     
        const [epoch] = PublicKey.findProgramAddressSync(
          [Buffer.from("epoch"), index.toArrayLike(Buffer, "le", 8)],
          program.programId
        );

        const tx = await program.rpc.manageStakerReward(
          index,
          {
            accounts: {
              admin: owner.publicKey,
              config,
              epoch,
              user: user.account.owner,
              userStake,
              stakes,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId
            },
            signers: [owner]
          }
        );
        console.log("tx->", tx);
      }
    } catch (error) {
      console.log("error:", error);
    }
  });
  */
  /*
  it("User 1 Cliam the reward", async() => {
    try {
      const index = new anchor.BN(1);

      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user1.publicKey.toBuffer()],
        program.programId
      );
      const userRewardAccount = getAssociatedTokenAddressSync(
        rewardTokenMint,
        user1.publicKey
      );

      const tx = await program.rpc.claim({
        accounts: {
          user: user1.publicKey,
          config,
          userStake,
          rewardVault,
          rewardMint: rewardTokenMint,
          userRewardAccount,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId
        },
        signers: [user1]
      });
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  it("User 2 Cliam the reward", async() => {
    try {
      const index = new anchor.BN(1);

      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user2.publicKey.toBuffer()],
        program.programId
      );
      const userRewardAccount = getAssociatedTokenAddressSync(
        rewardTokenMint,
        user2.publicKey
      );

      const tx = await program.rpc.claim({
        accounts: {
          user: user2.publicKey,
          config,
          userStake,
          rewardVault,
          rewardMint: rewardTokenMint,
          userRewardAccount,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId
        },
        signers: [user2]
      });
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
 
  it("Update duration", async() => {
    try {
      const duration = 14 * 24 * 60 * 60; // 14 days
      const tx = await program.rpc.updateEpochDuration(new anchor.BN(duration), {
        accounts: {
          config,
          admin: owner.publicKey
        },
        signers: [owner]
      });
      console.log("tx->", tx);
    } catch (error) {
      console.log("error:", error);
    }
  });
  */
});
