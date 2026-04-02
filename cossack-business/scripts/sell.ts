/**
 * Скрипт для продажу предметів на маркетплейсі
 * Гравець продає NFT і отримує MagicToken
 */

import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, setProvider } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

interface SellRequest {
  itemMint: PublicKey; // Мінт NFT предмета
  itemType: number; // 0-3
  itemName: string;
}

async function sellItemOnMarketplace(request: SellRequest) {
  const provider = AnchorProvider.env();
  setProvider(provider);
  const workspace = anchor.workspace as typeof anchor.workspace;

  const marketplaceProgram = workspace.Marketplace;
  const magicTokenProgram = workspace.MagicToken;

  console.log(`💰 Selling ${request.itemName} on marketplace...`);

  try {
    // 1. Отримати конфіг маркетплейсу
    const marketplaceConfigPda = PublicKey.findProgramAddressSync(
      [Buffer.from("marketplace_config")],
      marketplaceProgram.programId
    )[0];

    const marketplaceConfig =
      await marketplaceProgram.account.marketplaceConfig.fetch(
        marketplaceConfigPda
      );

    const itemPrice = marketplaceConfig.itemPrices[request.itemType];
    console.log(`Price: ${itemPrice.toString()} MagicToken`);

    // 2. Продати предмет
    const tx = await marketplaceProgram.methods
      .sellItem(request.itemType)
      .accounts({
        seller: provider.publicKey,
        marketplaceConfig: marketplaceConfigPda,
        itemNftMint: request.itemMint,
        sellerItemAccount: PublicKey.default, // Має бути токен-акаунт з NFT
        sellerMagicTokenAccount: PublicKey.default, // Має бути токен-акаунт для MagicToken
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("✅ Item sold successfully!");
    console.log(`   Item: ${request.itemName}`);
    console.log(`   Received: ${itemPrice.toString()} MagicToken`);
    console.log(`   TX: ${tx}`);

    // 3. Мінтити MagicToken через CPI
    console.log("🪙 Minting MagicToken reward...");

    const tokenAuthorityPda = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_token_authority")],
      magicTokenProgram.programId
    )[0];

    // Примітка: це були б викликано автоматично через CPI з маркетплейсу
    // За тепер це заготовка

    return {
      tx,
      itemPrice: itemPrice.toString(),
    };
  } catch (error) {
    console.error("❌ Error during marketplace sale:", error);
    throw error;
  }
}

export { sellItemOnMarketplace };
