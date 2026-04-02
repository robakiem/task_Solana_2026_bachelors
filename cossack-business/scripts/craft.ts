/**
 * Скрипт для крафту предметів
 * Перевіряє рецепт, спалює ресурси, мінтить NFT
 */

import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, setProvider } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

interface CraftRequest {
  itemType: number; // 0-3
  itemName: string;
  resourceMints: PublicKey[];
  playerResourceAccounts: PublicKey[];
}

async function craftItem(request: CraftRequest, itemMintAddress: PublicKey) {
  const provider = AnchorProvider.env();
  setProvider(provider);
  const workspace = anchor.workspace as typeof anchor.workspace;

  const craftingProgram = workspace.Crafting;
  const itemNftProgram = workspace.ItemNft;

  console.log(
    `🔨 Crafting ${request.itemName}... (Type: ${request.itemType})`
  );

  try {
    // 1. Отримати рецепт
    const recipe = await craftingProgram.methods
      .getRecipe(request.itemType)
      .view();

    console.log("Recipe requirements:", recipe);

    // 2. Виконати крафт (спалити ресурси)
    const remainingAccounts = [
      ...request.resourceMints.map((pubkey) => ({
        pubkey,
        isWritable: false,
        isSigner: false,
      })),
      ...request.playerResourceAccounts.map((pubkey) => ({
        pubkey,
        isWritable: true,
        isSigner: false,
      })),
    ];

    const craftTx = await craftingProgram.methods
      .craftItem(request.itemType, "ipfs://placeholder-metadata")
      .accounts({
        player: provider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts(remainingAccounts)
      .rpc();

    console.log(`✅ Craft successful!`);
    console.log(`   TX: ${craftTx}`);

    // 3. Мінтити NFT (це повинно бути частині крафту через CPI)
    console.log(`📦 Minting NFT...`);

    const craftTx2 = await itemNftProgram.methods
      .mintItemNft(request.itemType, "ipfs://placeholder-metadata-uri")
      .accounts({
        player: provider.publicKey,
        craftingProgram: provider.publicKey,
        itemMint: itemMintAddress,
        playerTokenAccount: PublicKey.default, // Має бути токен-акаунт гравця
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log(
      `✅ NFT minted! Item: ${request.itemName} (Mint: ${itemMintAddress.toString()})`
    );
    console.log(`   TX: ${craftTx2}`);
  } catch (error) {
    console.error("❌ Error during crafting:", error);
    throw error;
  }
}

// Типи предметів та їх рецепти
const RECIPES = {
  0: {
    name: "Козацька шабля",
    requirements: [1, 3, 0, 1, 0, 0], // [WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND]
  },
  1: {
    name: "Посох старійшини",
    requirements: [2, 0, 1, 0, 0, 1],
  },
  2: {
    name: "Броня характерника",
    requirements: [0, 2, 1, 4, 0, 0],
  },
  3: {
    name: "Бойовий браслет",
    requirements: [0, 4, 2, 0, 0, 2],
  },
};

export { craftItem, RECIPES };
