/**
 * Скрипт для ініціалізації гравця
 */

import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, setProvider } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Signer } from "@solana/web3.js";

async function initializePlayerForSearch(playerKeypair?: Signer) {
  const provider = AnchorProvider.env();
  setProvider(provider);
  const workspace = anchor.workspace as typeof anchor.workspace;

  const player = playerKeypair || provider.wallet;
  const searchProgram = workspace.Search;

  console.log(`🎮 Initializing player: ${player.publicKey.toString()}`);

  const playerPda = PublicKey.findProgramAddressSync(
    [Buffer.from("player"), player.publicKey.toBuffer()],
    searchProgram.programId
  )[0];

  try {
    const tx = await searchProgram.methods
      .initializePlayer()
      .accounts({
        payer: player.publicKey,
        playerState: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player as Signer])
      .rpc();

    console.log("✅ Player initialized for search");
    console.log(`   PDA: ${playerPda.toString()}`);
    console.log(`   TX: ${tx}`);
  } catch (error) {
    console.error("❌ Error initializing player:", error);
    throw error;
  }
}

// Запустити скрипт
initializePlayerForSearch().catch(console.error);

export { initializePlayerForSearch };
