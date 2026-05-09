/* import { LiteSVM } from "litesvm";
import * as anchor from "@coral-xyz/anchor";
import {
    PublicKey,
    Keypair,
    SystemProgram,
    Transaction,
    SYSVAR_CLOCK_PUBKEY,
    LAMPORTS_PER_SOL
} from "@solana/web3.js";
import { expect } from "chai";
import * as fs from "fs";
import BN from "bn.js";
import { address } from "@solana/addresses";
import { lamports } from "@solana/rpc-types";
import type {
    TransactionMessageBytes,
    SignaturesMap
} from "@solana/transactions";

const IDL = JSON.parse(fs.readFileSync("./target/idl/prediction_clob.json", "utf-8"));
const PROGRAM_ID = new PublicKey("2cffJrXyZjoN1jT2BB671BjeqfxaJzHGgfjAd8QZQ8qh");

const getSvmSignatures = (tx: Transaction): SignaturesMap => {
    const signatures: SignaturesMap = {};
    tx.signatures.forEach(sigpair => {
        if (sigpair.signature) {
            signatures[address(sigpair.publicKey.toBase58())] = new Uint8Array(sigpair.signature) as any;
        }
    });
    return signatures;
};

describe("Prediction Market CLOB", () => {
    let svm: LiteSVM;
    let payer: Keypair;
    let marketPda: PublicKey;
    let obA: PublicKey;
    let obB: PublicKey;
    let usdcMint: PublicKey;
    let outcomeAMint: PublicKey;
    let outcomeBMint: PublicKey;
    let marketVault: PublicKey;

    const marketId = 1;
    const coder = new anchor.BorshCoder(IDL);

    before(async () => {
        svm = new LiteSVM();
        payer = Keypair.generate();

        const programBinary = fs.readFileSync("./target/deploy/prediction_clob.so");
        await svm.addProgram(address(PROGRAM_ID.toBase58()), programBinary);
        await svm.airdrop(address(payer.publicKey.toBase58()), lamports(BigInt(10_000_000_000)));

        const marketIdBuffer = Buffer.alloc(4);
        marketIdBuffer.writeUInt32LE(marketId);

        [marketPda] = PublicKey.findProgramAddressSync([Buffer.from("market"), marketIdBuffer], PROGRAM_ID);
        [marketVault] = PublicKey.findProgramAddressSync([Buffer.from("vault"), marketIdBuffer], PROGRAM_ID);
        [outcomeAMint] = PublicKey.findProgramAddressSync([Buffer.from("outcome_a"), marketIdBuffer], PROGRAM_ID);
        [outcomeBMint] = PublicKey.findProgramAddressSync([Buffer.from("outcome_b"), marketIdBuffer], PROGRAM_ID);

        [obA] = PublicKey.findProgramAddressSync([Buffer.from("orderbook"), marketPda.toBuffer(), outcomeAMint.toBuffer()], PROGRAM_ID);
        [obB] = PublicKey.findProgramAddressSync([Buffer.from("orderbook"), marketPda.toBuffer(), outcomeBMint.toBuffer()], PROGRAM_ID);

        usdcMint = Keypair.generate().publicKey;

        const mintData = Buffer.alloc(82);
        mintData.writeUInt8(1, 45);

        await svm.setAccount({
            address: address(usdcMint.toBase58()),
            lamports: lamports(BigInt(1_000_000_000)),
            data: mintData,
            executable: false,
            programAddress: address(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            ),
            space: BigInt(mintData.length),
        });

        const marketData = await coder.accounts.encode("Market", {
            authority: payer.publicKey,
            market_id: marketId,
            settlement_deadline: new BN(1000),
            collateral_vault: marketVault,
            outcome_a_mint: outcomeAMint,
            outcome_b_mint: outcomeBMint,
            collateral_mint: usdcMint,
            is_settled: false,
            winning_outcome: null,
            reported_outcome: null,
            report_timestamp: new BN(0),
            challenge_end_timestamp: new BN(0),
            total_collateral_locked: new BN(0),
            bump: 255,
        });

        await svm.setAccount({
            address: address(marketPda.toBase58()),
            lamports: lamports(BigInt(1_000_000_000)),
            data: new Uint8Array(marketData),
            executable: false,
            programAddress: address(PROGRAM_ID.toBase58()),
            space: BigInt(marketData.length)
        });
    });

    const setupUser = async (user: Keypair, usdcAccount: bigint) => {
        const ata = Keypair.generate().publicKey;
        const tokenAccData = Buffer.alloc(165);

        tokenAccData.set(usdcMint.toBuffer(), 0);
        tokenAccData.set(user.publicKey.toBuffer(), 32);
        tokenAccData.writeBigUInt64LE(usdcAccount, 64);
        tokenAccData.writeUInt8(1, 108);

        await svm.airdrop(address(user.publicKey.toBase58()), lamports(BigInt(LAMPORTS_PER_SOL)));
        await svm.setAccount({
            address: address(ata.toBase58()),
            lamports: lamports(BigInt(1_000_000_000)),
            data: tokenAccData,
            executable: false,
            programAddress: address(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            ),
            space: BigInt(165)
        });

        return ata;
    };

    it("Completes market lifecycle", async () => {
        for (const ob of [obA, obB]) {
            const ixData = coder.instruction.encode("initialize_orderbook", {});
            const tx = new Transaction().add({
                keys: [
                    { pubkey: ob, isSigner: false, isWritable: true },
                    { pubkey: marketPda, isSigner: false, isWritable: true },
                    { pubkey: ob === obA ? outcomeAMint : outcomeBMint, isSigner: false, isWritable: false },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data: ixData
            });
            tx.recentBlockhash = await svm.latestBlockhash();
            tx.sign(payer);

            await svm.sendTransaction({
                messageBytes: tx.serializeMessage() as unknown as TransactionMessageBytes,
                signatures: getSvmSignatures(tx)
            });
        }

        const alice = Keypair.generate();
        const bob = Keypair.generate();
        const aliceUsdc = await setupUser(alice, BigInt(1000));
        const bobUsdc = await setupUser(bob, BigInt(1000));

        const [aliceLedger] = PublicKey.findProgramAddressSync([Buffer.from("user_account"), marketPda.toBuffer(), alice.publicKey.toBuffer()], PROGRAM_ID);
        const [bobLedger] = PublicKey.findProgramAddressSync([Buffer.from("user_account"), marketPda.toBuffer(), bob.publicKey.toBuffer()], PROGRAM_ID);

        const aliceIx = coder.instruction.encode("place_order", { is_buying_a: true, quantity: new BN(100), price: new BN(60) });
        const aliceTx = new Transaction().add({
            keys: [
                { pubkey: obA, isSigner: false, isWritable: true },
                { pubkey: obB, isSigner: false, isWritable: true },
                { pubkey: marketPda, isSigner: false, isWritable: true },
                { pubkey: aliceLedger, isSigner: false, isWritable: true },
                { pubkey: alice.publicKey, isSigner: true, isWritable: true },
                { pubkey: aliceUsdc, isSigner: false, isWritable: true },
                { pubkey: marketVault, isSigner: false, isWritable: true },
                { pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner: false, isWritable: false },
                { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
            ],
            programId: PROGRAM_ID,
            data: aliceIx,
        });
        aliceTx.recentBlockhash = await svm.latestBlockhash();
        aliceTx.sign(alice);

        await svm.sendTransaction({
            messageBytes: aliceTx.serializeMessage() as unknown as TransactionMessageBytes,
            signatures: getSvmSignatures(aliceTx)
        });

        const bobIx = coder.instruction.encode("place_order", { is_buying_a: false, quantity: new BN(100), price: new BN(40) });
        const bobTx = new Transaction().add({
            keys: [
                { pubkey: obA, isSigner: false, isWritable: true },
                { pubkey: obB, isSigner: false, isWritable: true },
                { pubkey: marketPda, isSigner: false, isWritable: true },
                { pubkey: bobLedger, isSigner: false, isWritable: true },
                { pubkey: bob.publicKey, isSigner: true, isWritable: true },
                { pubkey: bobUsdc, isSigner: false, isWritable: true },
                { pubkey: marketVault, isSigner: false, isWritable: true },
                { pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner: false, isWritable: false },
                { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
            ],
            programId: PROGRAM_ID,
            data: bobIx,
        });
        bobTx.recentBlockhash = await svm.latestBlockhash();
        bobTx.sign(bob);

        await svm.sendTransaction({
            messageBytes: bobTx.serializeMessage() as unknown as TransactionMessageBytes,
            signatures: getSvmSignatures(bobTx)
        });

        // verify balances
        const aliceAcc = await svm.getAccount(address(aliceLedger.toBase58()));
        if (!aliceAcc.exists) {
            throw new Error("Alice ledger account not found");
        }
        const aliceData =
            coder.accounts.decode(
                "UserAccount",
                Buffer.from(aliceAcc.data)
            );
        expect(aliceData.outcome_a_balance.toNumber()).to.equal(100);
        console.log("✅ Virtual Match successful: Alice received 100 Outcome A tokens.");

        // market resolution
        const currentClock = svm.getClock();
        await svm.setClock({
            slot: currentClock.slot,
            epoch: currentClock.epoch,
            epochStartTimestamp: currentClock.epochStartTimestamp,
            leaderScheduleEpoch: currentClock.leaderScheduleEpoch,
            unixTimestamp: BigInt(2000),
        });

        const proposeIx = coder.instruction.encode("propose_outcome", { reported_outcome: 0 });
        const proposeTx = new Transaction().add({
            keys: [
                { pubkey: marketPda, isSigner: false, isWritable: true },
                { pubkey: payer.publicKey, isSigner: true, isWritable: false },
            ],
            programId: PROGRAM_ID,
            data: proposeIx,
        });
        proposeTx.recentBlockhash = await svm.latestBlockhash();
        proposeTx.sign(payer);

        await svm.sendTransaction({
            messageBytes: proposeTx.serializeMessage() as any,
            signatures: getSvmSignatures(proposeTx),
        });
        console.log("✅ Outcome Proposed.");

        await svm.setClock({
            slot: currentClock.slot,
            epoch: currentClock.epoch,
            epochStartTimestamp: currentClock.epochStartTimestamp,
            leaderScheduleEpoch: currentClock.leaderScheduleEpoch,
            unixTimestamp: BigInt(2000 + 86401),
        });
        const finalizeIx = coder.instruction.encode("finalize_market", {});
        const finalizeTx = new Transaction().add({
            keys: [{ pubkey: marketPda, isSigner: false, isWritable: true }],
            programId: PROGRAM_ID,
            data: finalizeIx,
        });
        finalizeTx.recentBlockhash = await svm.latestBlockhash();

        await svm.sendTransaction({
            messageBytes: finalizeTx.serializeMessage() as any,
            signatures: getSvmSignatures(finalizeTx),
        });

        console.log("✅ Market Finalized.");
    });
}); */