import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
    PublicKey,
    SystemProgram,
    Keypair,
    Transaction
} from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
import assert from "assert";
import { PredictionClob } from "../target/types/prediction_clob";

describe("prediction-clob-integration", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.PredictionClob as Program<PredictionClob>;

    const SENTINEL = 4294967295;
    const marketId = 1;

    let marketPda: PublicKey;
    let collateralMint: PublicKey;
    let outcomeAMint: PublicKey;
    let outcomeBMint: PublicKey;

    // 🏗️ CHANGE: orderbook is now a Keypair, not just a PublicKey
    let orderbookKeypair = Keypair.generate();

    it("Initializes Market and a Large 90KB Orderbook", async () => {
        const payer = (provider.wallet as anchor.Wallet).payer;

        console.log("Creating SPL Mints...");
        collateralMint = await createMint(provider.connection, payer, payer.publicKey, null, 6);
        outcomeAMint = await createMint(provider.connection, payer, payer.publicKey, null, 0);
        outcomeBMint = await createMint(provider.connection, payer, payer.publicKey, null, 0);

        const marketIdBuffer = Buffer.alloc(4);
        marketIdBuffer.writeUInt32LE(marketId);
        [marketPda] = PublicKey.findProgramAddressSync(
            [Buffer.from("market"), marketIdBuffer],
            program.programId
        );

        console.log("Initializing Market...");
        const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);
        await program.methods
            .initializeMarket(marketId, deadline, outcomeAMint, outcomeBMint, collateralMint)
            .accounts({
                market: marketPda,
                authority: payer.publicKey,
                systemProgram: SystemProgram.programId,
            } as any)
            .rpc();

        const ORDERBOOK_SPACE = 8 + 90256;
        const lamports = await provider.connection.getMinimumBalanceForRentExemption(ORDERBOOK_SPACE);

        console.log(`Pre-allocating ${ORDERBOOK_SPACE} bytes for Orderbook...`);
        const createOrderbookIx = SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: orderbookKeypair.publicKey,
            lamports,
            space: ORDERBOOK_SPACE,
            programId: program.programId,
        });

        console.log("Initializing Orderbook (Instruction)...");
        await program.methods
            .initializeOrderbook()
            .accounts({
                orderbook: orderbookKeypair.publicKey,
                market: marketPda,
                outcomeMint: outcomeAMint,
                payer: payer.publicKey,
                systemProgram: SystemProgram.programId,
            } as any)
            .preInstructions([createOrderbookIx])
            .signers([orderbookKeypair])
            .rpc();

        const obState = await program.account.orderbook.fetch(orderbookKeypair.publicKey);
        assert.strictEqual(obState.freeHead, 0);
        console.log("✅ 90KB Orderbook successfully initialized via pre-allocation!");
    });
});