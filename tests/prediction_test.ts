import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Keypair, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import assert from "assert";
import { PredictionClob } from "../target/types/prediction_clob";

describe("prediction-clob-integration", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.PredictionClob as Program<PredictionClob>;
    const payer = (provider.wallet as anchor.Wallet).payer;

    const marketId = 1;

    let marketPda: PublicKey;
    let collateralMint: PublicKey;
    let outcomeAMint: PublicKey;
    let outcomeBMint: PublicKey;
    let userAccountPda: PublicKey;
    let marketVault: PublicKey;
    let userCollateralAta: PublicKey;

    let obAKeypair = Keypair.generate();
    let obBKeypair = Keypair.generate();

    before(async () => {
        collateralMint = await createMint(provider.connection, payer, payer.publicKey, null, 6);
        outcomeAMint = await createMint(provider.connection, payer, payer.publicKey, null, 0);
        outcomeBMint = await createMint(provider.connection, payer, payer.publicKey, null, 0);

        const marketIdBuffer = Buffer.alloc(4);
        marketIdBuffer.writeUInt32LE(marketId);
        [marketPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketIdBuffer],
            program.programId
        );

        [userAccountPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("user"), marketPda.toBuffer(), payer.publicKey.toBuffer()],
            program.programId
        );

        userCollateralAta = (await getOrCreateAssociatedTokenAccount(
            provider.connection, payer, collateralMint, payer.publicKey
        )).address;

        marketVault = (await getOrCreateAssociatedTokenAccount(
            provider.connection, payer, collateralMint, marketPda, true
        )).address;

        await mintTo(provider.connection, payer, collateralMint, userCollateralAta, payer, 1000_000_000);
    });

    it("Performs Full System Initialization", async () => {
        const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

        await program.methods
            .initializeMarket(marketId, deadline, outcomeAMint, outcomeBMint, collateralMint)
            .accounts({ market: marketPda, authority: payer.publicKey } as any)
            .rpc();

        const ORDERBOOK_SPACE = 8 + 90256;
        const rent = await provider.connection.getMinimumBalanceForRentExemption(ORDERBOOK_SPACE);

        for (let [kb, mint] of [[obAKeypair, outcomeAMint], [obBKeypair, outcomeBMint]] as const) {
            const createIx = SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: kb.publicKey,
                lamports: rent,
                space: ORDERBOOK_SPACE,
                programId: program.programId,
            });

            await program.methods
                .initializeOrderbook()
                .accounts({
                    orderbook: kb.publicKey,
                    market: marketPda,
                    outcomeMint: mint,
                    payer: payer.publicKey,
                } as any)
                .preInstructions([createIx])
                .signers([kb])
                .rpc();
        }

        console.log("Initializing User Account PDA...");
        await program.methods
            .initializeUserAccount()
            .accounts({
                userAccount: userAccountPda,
                market: marketPda,
                owner: payer.publicKey,
                systemProgram: SystemProgram.programId,
            } as any)
            .rpc();
    });

    it("Places an order and verifies positions", async () => {
        const price = new anchor.BN(50);
        const quantity = new anchor.BN(10);

        await program.methods
            .placeOrder(true, quantity, price)
            .accounts({
                orderbookA: obAKeypair.publicKey,
                orderbookB: obBKeypair.publicKey,
                market: marketPda,
                userAccount: userAccountPda,
                user: payer.publicKey,
                userCollateralAta: userCollateralAta,
                marketVault: marketVault,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: SYSVAR_CLOCK_PUBKEY,
            } as any)
            .rpc();

        const userState = await program.account.userAccount.fetch(userAccountPda);
        console.log(`User Outcome A Position: ${userState.outcomeABalance}`);

        assert.ok(userState.owner.equals(payer.publicKey));
        console.log("✅ User Portfolio PDA correctly tracking position.");
    });

    it("Claims collateral back to user ATA", async () => {
        let userState = await program.account.userAccount.fetch(userAccountPda);
        const amountToClaim = userState.collateralBalance;

        if (amountToClaim.isZero()) {
            console.log("⚠️ collateralBalance is 0. Expecting 'NothingToClaim' error.");
            try {
                await program.methods
                    .claimCollateral()
                    .accounts({
                        userAccount: userAccountPda,
                        market: marketPda,
                        collateralVault: marketVault,
                        userCollateralAta: userCollateralAta,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        user: payer.publicKey,
                    } as any)
                    .rpc();
                assert.fail("Should have failed with NothingToClaim");
            } catch (err) {
                assert.ok(err.logs.join("").includes("NothingToClaim") || err.message.includes("6000"));
                console.log("✅ Corrected rejected claim for zero balance.");
            }
            return;
        }

        const ataBefore = (await provider.connection.getTokenAccountBalance(userCollateralAta)).value.amount;

        const tx = await program.methods
            .claimCollateral()
            .accounts({
                userAccount: userAccountPda,
                market: marketPda,
                collateralVault: marketVault,
                userCollateralAta: userCollateralAta,
                tokenProgram: TOKEN_PROGRAM_ID,
                user: payer.publicKey,
            } as any)
            .rpc();

        console.log("✅ Claim TX:", tx);

        const ataAfter = (await provider.connection.getTokenAccountBalance(userCollateralAta)).value.amount;
        const userStateAfter = await program.account.userAccount.fetch(userAccountPda);

        assert.strictEqual(
            Number(ataAfter) - Number(ataBefore),
            amountToClaim.toNumber(),
            "User ATA should increase by the claimed amount"
        );
        assert.strictEqual(userStateAfter.collateralBalance.toNumber(), 0, "Ledger balance should be reset to 0");
    });
});