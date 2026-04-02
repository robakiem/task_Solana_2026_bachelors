/**
 * Скрипт для пошуку ресурсів
 * Гравець отримує 3 випадкових ресурси раз на 60 секунд
 */

import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, setProvider } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";

async function executeResourceSearch(
  gameConfigMints: PublicKey[],
  playerTokenAccounts: PublicKey[]
) {
  const provider = AnchorProvider.env();
  setProvider(provider);
  const workspace = anchor.workspace as typeof anchor.workspace;

  const searchProgram = workspace.Search;
  const player = provider.publicKey;

  console.log(`🔍 Executing resource search for player: ${player.toString()}`);

  const playerPda = PublicKey.findProgramAddressSync(
    [Buffer.from("player"), player.toBuffer()],
    searchProgram.programId
  )[0];

  // Генерувати 3 випадкові індекси ресурсів
  const resourceIndices: number[] = [];
  for (let i = 0; i < 3; i++) {
    resourceIndices.push(Math.floor(Math.random() * 6));
  }

  console.log(`Resources to search: ${resourceIndices.join(", ")}`);

  try {
    // Перевірити час до наступого пошуку
    const playerState = await searchProgram.account.playerSearchState.fetch(
      playerPda
    );
    const currentTime = Date.now() / 1000;
    const nextSearchTime =
      playerState.lastSearchTime.toNumber() +
      60;

    if (currentTime < nextSearchTime) {
      const secondsWait = Math.ceil(nextSearchTime - currentTime);
      console.log(
        `⏳ Search is on cooldown. Wait ${secondsWait} more seconds.`
      );
      return;
    }

    // Виконати пошук
    const tx = await searchProgram.methods
      .executeSearch(resourceIndices)
      .accounts({
        player,
        playerState: playerPda,
        searchAuthority: provider.publicKey, // Мав бути PDA з правом на мінтинг
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .remainingAccounts(
        [
          ...gameConfigMints.map((pubkey) => ({
            pubkey,
            isSigner: false,
            isWritable: false,
          })),
          ...playerTokenAccounts.slice(0, 3).map((pubkey) => ({
            pubkey,
            isSigner: false,
            isWritable: true,
          })),
        ]
      )
      .rpc();

    console.log("✅ Search completed!");
    console.log(`TX: ${tx}`);
  } catch (error) {
    console.error("❌ Error during search:", error);
    throw error;
  }
}

export { executeResourceSearch };
