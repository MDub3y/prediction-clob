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

    const SENTINEL = 4294967295;
    const marketId = 1;

    let marketPda: PublicKey;
    let collateralMint: PublicKey;
    let outcomeAMint: PublicKey;
    let outcomeBMint: PublicKey;
    let userAccountPda: PublicKey;
    let marketVault: PublicKey;
    let userCollateralAta: PublicKey;
    let userOutcomeAAta: PublicKey;
    let userOutcomeBAta: PublicKey;

    let obAKeypair = Keypair.generate();
    let obBKeypair = Keypair.generate();

    before(async () => {
        const marketIdBuffer = Buffer.alloc(4);
        marketIdBuffer.writeUInt32LE(marketId);
        [marketPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketIdBuffer],
            program.programId
        );

        collateralMint = await createMint(provider.connection, payer, payer.publicKey, null, 6);
        outcomeAMint = await createMint(provider.connection, payer, marketPda, null, 0);
        outcomeBMint = await createMint(provider.connection, payer, marketPda, null, 0);

        [userAccountPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("user"), marketPda.toBuffer(), payer.publicKey.toBuffer()],
            program.programId
        );

        userCollateralAta = (await getOrCreateAssociatedTokenAccount(
            provider.connection, payer, collateralMint, payer.publicKey
        )).address;

        userOutcomeAAta = (await getOrCreateAssociatedTokenAccount(provider.connection, payer, outcomeAMint, payer.publicKey)).address;
        userOutcomeBAta = (await getOrCreateAssociatedTokenAccount(provider.connection, payer, outcomeBMint, payer.publicKey)).address;

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
                outcomeAMint: outcomeAMint,
                outcomeBMint: outcomeBMint,
                userOutcomeAAta: userOutcomeAAta,
                userOutcomeBAta: userOutcomeBAta,
                user: payer.publicKey,
                userCollateralAta: userCollateralAta,
                marketVault: marketVault,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: SYSVAR_CLOCK_PUBKEY,
                systemProgram: SystemProgram.programId,
            } as any)
            .rpc();

        const userState = await program.account.userAccount.fetch(userAccountPda);
        assert.ok(userState.owner.equals(payer.publicKey));

        const obState = await program.account.orderbook.fetch(obAKeypair.publicKey);
        await program.methods.cancelOrder(obState.bidHead, 0).accounts({
            orderbook: obAKeypair.publicKey,
            market: marketPda,
            userAccount: userAccountPda,
            user: payer.publicKey,
        } as any).rpc();

        await program.methods.claimCollateral().accounts({
            userAccount: userAccountPda,
            market: marketPda,
            collateralVault: marketVault,
            userCollateralAta: userCollateralAta,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: payer.publicKey,
        } as any).rpc();

        console.log("✅ Placement verified and book cleared.");
    });

    it("Splits 10 units of collateral into Outcome A and B tokens", async () => {
        const amount = new anchor.BN(10);
        await program.methods
            .split(amount)
            .accounts({
                market: marketPda,
                outcomeAMint: outcomeAMint,
                outcomeBMint: outcomeBMint,
                userOutcomeAAta: userOutcomeAAta,
                userOutcomeBAta: userOutcomeBAta,
                userCollateralAta: userCollateralAta,
                marketVault: marketVault,
                user: payer.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
            } as any)
            .rpc();

        const balA = await provider.connection.getTokenAccountBalance(userOutcomeAAta);
        assert.strictEqual(balA.value.amount, "10", "Should have 10 A tokens");
        console.log("✅ Split successful.");
    });

    it("Cancels an open order and refunds collateral to ledger", async () => {
        const price = new anchor.BN(10);
        const quantity = new anchor.BN(5);

        await program.methods
            .placeOrder(true, quantity, price)
            .accounts({
                orderbookA: obAKeypair.publicKey,
                orderbookB: obBKeypair.publicKey,
                market: marketPda,
                userAccount: userAccountPda,
                outcomeAMint: outcomeAMint,
                outcomeBMint: outcomeBMint,
                userOutcomeAAta: userOutcomeAAta,
                userOutcomeBAta: userOutcomeBAta,
                user: payer.publicKey,
                userCollateralAta: userCollateralAta,
                marketVault: marketVault,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: SYSVAR_CLOCK_PUBKEY,
                systemProgram: SystemProgram.programId,
            } as any)
            .rpc();

        const obState = await program.account.orderbook.fetch(obAKeypair.publicKey);
        const orderIdx = obState.bidHead;

        await program.methods
            .cancelOrder(orderIdx, 0)
            .accounts({
                orderbook: obAKeypair.publicKey,
                market: marketPda,
                userAccount: userAccountPda,
                user: payer.publicKey,
            } as any)
            .rpc();

        const userState = await program.account.userAccount.fetch(userAccountPda);
        assert.strictEqual(userState.collateralBalance.toNumber(), 50, "Refund should be exactly 50");

        await program.methods.claimCollateral().accounts({
            userAccount: userAccountPda,
            market: marketPda,
            collateralVault: marketVault,
            userCollateralAta: userCollateralAta,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: payer.publicKey,
        } as any).rpc();

        console.log("✅ Cancellation verified.");
    });

    it("Claims collateral back to user ATA", async () => {
        const price = new anchor.BN(20);
        const quantity = new anchor.BN(5);

        await program.methods
            .placeOrder(true, quantity, price)
            .accounts({
                orderbookA: obAKeypair.publicKey,
                orderbookB: obBKeypair.publicKey,
                market: marketPda,
                userAccount: userAccountPda,
                outcomeAMint: outcomeAMint,
                outcomeBMint: outcomeBMint,
                userOutcomeAAta: userOutcomeAAta,
                userOutcomeBAta: userOutcomeBAta,
                user: payer.publicKey,
                userCollateralAta: userCollateralAta,
                marketVault: marketVault,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: SYSVAR_CLOCK_PUBKEY,
                systemProgram: SystemProgram.programId,
            } as any)
            .rpc();

        const obState = await program.account.orderbook.fetch(obAKeypair.publicKey);
        await program.methods
            .cancelOrder(obState.bidHead, 0)
            .accounts({
                orderbook: obAKeypair.publicKey,
                market: marketPda,
                userAccount: userAccountPda,
                user: payer.publicKey,
            } as any)
            .rpc();

        const userStateMid = await program.account.userAccount.fetch(userAccountPda);
        const amountToClaim = userStateMid.collateralBalance; // Should be 100

        const ataBefore = (await provider.connection.getTokenAccountBalance(userCollateralAta)).value.amount;

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

        const ataAfter = (await provider.connection.getTokenAccountBalance(userCollateralAta)).value.amount;
        const userStateAfter = await program.account.userAccount.fetch(userAccountPda);

        assert.strictEqual(
            Number(ataAfter) - Number(ataBefore),
            amountToClaim.toNumber(),
            "User ATA should increase by the ledger amount"
        );
        assert.strictEqual(userStateAfter.collateralBalance.toNumber(), 0, "Ledger should be empty after claim");

        console.log(`✅ Successfully claimed ${amountToClaim.toNumber()} units from ledger to wallet.`);
    });
});