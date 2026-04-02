/**
 * Скрипт ініціалізації гри "Козацький бізнес"
 * 
 * Виконує:
 * 1. Ініціалізацію GameSettings (конфігурація гри)
 * 2. Ініціалізацію MagicToken
 * 3. Ініціалізацію маркетплейсу
 */

import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, setProvider } from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
  createInitializeMintInstruction,
} from "@solana/spl-token";

interface GameConfig {
  resourceMints: PublicKey[];
  magicTokenMint: PublicKey;
  itemPrices: bigint[];
}

async function createSPLToken(
  provider: AnchorProvider,
  decimals: number = 0
): Promise<PublicKey> {
  const mint = Keypair.generate();
  const transaction = new Transaction();

  // Створити мінт токена
  const createMintIx = createInitializeMintInstruction(
    mint.publicKey,
    decimals,
    provider.publicKey,
    provider.publicKey
  );

  transaction.add(
    SystemProgram.createAccount({
      fromPubkey: provider.publicKey,
      newAccountPubkey: mint.publicKey,
      space: MINT_SIZE,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(
        MINT_SIZE
      ),
      programId: TOKEN_PROGRAM_ID,
    }),
    createMintIx
  );

  await provider.sendAndConfirm(transaction, [mint]);
  console.log(`✅ Created SPL token: ${mint.publicKey.toString()}`);

  return mint.publicKey;
}

async function initializeGame() {
  const provider = AnchorProvider.env();
  setProvider(provider);
  const workspace = anchor.workspace as typeof anchor.workspace;

  console.log("🎮 Initializing Kozatskyi Business...");
  console.log(`Connected wallet: ${provider.publicKey.toString()}`);

  try {
    // 1. Створити 6 ресурсів
    console.log("\n📦 Creating 6 resource tokens...");
    const resourceMints: PublicKey[] = [];
    const resourceNames = ["WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND"];

    for (let i = 0; i < 6; i++) {
      const mint = await createSPLToken(provider);
      resourceMints.push(mint);
      console.log(`   ${i + 1}. ${resourceNames[i]}: ${mint.toString()}`);
    }

    // 2. Створити MagicToken
    console.log("\n💎 Creating MagicToken...");
    const magicTokenMint = await createSPLToken(provider);
    console.log(`✅ MagicToken: ${magicTokenMint.toString()}`);

    // 3. Інєціалізувати GameSettings через resource_manager
    console.log("\n⚙️ Initializing GameSettings...");
    const resourceManagerProgram = workspace.ResourceManager;
    const gameSettingsPda = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      resourceManagerProgram.programId
    )[0];

    const itemPrices = [BigInt(100), BigInt(150), BigInt(200), BigInt(250)]; // Ціни в MagicToken

    try {
      await resourceManagerProgram.methods
        .setupGameSystem(resourceNames, itemPrices)
        .accounts({
          payer: provider.publicKey,
          gameSettings: gameSettingsPda,
          magicTokenMint: magicTokenMint,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(
          resourceMints.map((pubkey) => ({
            pubkey,
            isSigner: false,
            isWritable: false,
          }))
        )
        .rpc();

      console.log("✅ GameSettings initialized");
    } catch (err) {
      console.error("Error initializing GameSettings:", err);
    }

    // 4. Ініціалізувати magic_token програму
    console.log("\n🪙 Initializing MagicToken authority...");
    const magicTokenProgram = workspace.MagicToken;
    const tokenAuthorityPda = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_token_authority")],
      magicTokenProgram.programId
    )[0];

    try {
      await magicTokenProgram.methods
        .initializeMintAuthority(PublicKey.default) // Marketplace address буде встановлена пізніше
        .accounts({
          payer: provider.publicKey,
          tokenAuthority: tokenAuthorityPda,
          magicTokenMint: magicTokenMint,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("✅ MagicToken authority initialized");
    } catch (err) {
      console.error("Error initializing MagicToken:", err);
    }

    // 5. Ініціалізувати маркетплейс
    console.log("\n🛒 Initializing Marketplace...");
    const marketplaceProgram = workspace.Marketplace;
    const marketplaceConfigPda = PublicKey.findProgramAddressSync(
      [Buffer.from("marketplace_config")],
      marketplaceProgram.programId
    )[0];

    try {
      await marketplaceProgram.methods
        .initializeMarketplace(magicTokenProgram.programId, itemPrices)
        .accounts({
          payer: provider.publicKey,
          marketplaceConfig: marketplaceConfigPda,
          magicTokenMint: magicTokenMint,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("✅ Marketplace initialized");
    } catch (err) {
      console.error("Error initializing Marketplace:", err);
    }

    // Вивести результат
    console.log("\n═══════════════════════════════════════════");
    console.log("✅ GAME INITIALIZATION COMPLETE!");
    console.log("═══════════════════════════════════════════");
    console.log("\nResources:");
    resourceNames.forEach((name, i) => {
      console.log(`  ${name}: ${resourceMints[i].toString()}`);
    });
    console.log(`\nMagicToken: ${magicTokenMint.toString()}`);
    console.log(`GameSettings: ${gameSettingsPda.toString()}`);
    console.log(`Marketplace: ${marketplaceConfigPda.toString()}`);

    // Зберегти конфіг для подальшого використання
    const gameConfig: GameConfig = {
      resourceMints,
      magicTokenMint,
      itemPrices,
    };

    return gameConfig;
  } catch (error) {
    console.error("❌ Error during initialization:", error);
    throw error;
  }
}

// Запустити скрипт
initializeGame().catch(console.error);

export { initializeGame };
