import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import * as splToken from "@solana/spl-token";
import { expect } from "chai";

describe("Cossack Business - Game Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const resourceManagerProgram = anchor.workspace.ResourceManager;
  const searchProgram = anchor.workspace.Search;
  const craftingProgram = anchor.workspace.Crafting;
  const itemNftProgram = anchor.workspace.ItemNft;
  const magicTokenProgram = anchor.workspace.MagicToken;
  const marketplaceProgram = anchor.workspace.Marketplace;

  let resourceMints: PublicKey[] = [];
  let magicTokenMint: PublicKey;
  let gameSettingsPda: PublicKey;
  let playerPda: PublicKey;
  let playerTokenAccounts: PublicKey[] = [];
  let tokenAuthorityPda: PublicKey;
  let marketplaceConfigPda: PublicKey;
  let itemNftMint: PublicKey;
  let itemDataPda: PublicKey;

  it("Initialize game and create resource mints", async () => {
    gameSettingsPda = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      resourceManagerProgram.programId
    )[0];

    // Create 6 resource mints
    for (let i = 0; i < 6; i++) {
      const mint = anchor.web3.Keypair.generate();
      const transaction = new anchor.web3.Transaction();

      const createMintIx = await splToken.createInitializeMintInstruction(
        mint.publicKey,
        0,
        provider.publicKey,
        provider.publicKey
      );

      transaction.add(
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: mint.publicKey,
          space: splToken.MINT_SIZE,
          lamports: await provider.connection.getMinimumBalanceForRentExemption(
            splToken.MINT_SIZE
          ),
          programId: splToken.TOKEN_PROGRAM_ID,
        }),
        createMintIx
      );

      await provider.sendAndConfirm(transaction, [mint]);
      resourceMints.push(mint.publicKey);
    }

    expect(resourceMints.length).to.equal(6);
  });

  it("Create MagicToken mint", async () => {
    const mint = anchor.web3.Keypair.generate();
    const transaction = new anchor.web3.Transaction();

    const createMintIx = await splToken.createInitializeMintInstruction(
      mint.publicKey,
      0,
      provider.publicKey,
      provider.publicKey
    );

    transaction.add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mint.publicKey,
        space: splToken.MINT_SIZE,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          splToken.MINT_SIZE
        ),
        programId: splToken.TOKEN_PROGRAM_ID,
      }),
      createMintIx
    );

    await provider.sendAndConfirm(transaction, [mint]);
    magicTokenMint = mint.publicKey;

    expect(magicTokenMint).to.not.be.null;
  });

  it("Setup game system with resource_manager", async () => {
    const itemPrices = [BigInt(100), BigInt(150), BigInt(200), BigInt(250)];
    const resourceNames = [
      "WOOD",
      "IRON",
      "GOLD",
      "LEATHER",
      "STONE",
      "DIAMOND",
    ];

    const tx = await resourceManagerProgram.methods
      .setupGameSystem(resourceNames, itemPrices)
      .accounts({
        payer: provider.publicKey,
        gameSettings: gameSettingsPda,
        magicTokenMint: magicTokenMint,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .remainingAccounts(
        resourceMints.map((pubkey) => ({
          pubkey,
          isSigner: false,
          isWritable: false,
        }))
      )
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Initialize player for search", async () => {
    playerPda = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), provider.publicKey.toBuffer()],
      searchProgram.programId
    )[0];

    const tx = await searchProgram.methods
      .initializePlayer()
      .accounts({
        payer: provider.publicKey,
        playerState: playerPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Initialize MagicToken authority", async () => {
    tokenAuthorityPda = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_token_authority")],
      magicTokenProgram.programId
    )[0];

    const tx = await magicTokenProgram.methods
      .initializeMintAuthority(PublicKey.default)
      .accounts({
        payer: provider.publicKey,
        tokenAuthority: tokenAuthorityPda,
        magicTokenMint: magicTokenMint,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Initialize marketplace", async () => {
    marketplaceConfigPda = PublicKey.findProgramAddressSync(
      [Buffer.from("marketplace_config")],
      marketplaceProgram.programId
    )[0];

    const itemPrices = [BigInt(100), BigInt(150), BigInt(200), BigInt(250)];

    const tx = await marketplaceProgram.methods
      .initializeMarketplace(magicTokenProgram.programId, itemPrices)
      .accounts({
        payer: provider.publicKey,
        marketplaceConfig: marketplaceConfigPda,
        magicTokenMint: magicTokenMint,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Get recipe from crafting program", async () => {
    const recipe = await craftingProgram.methods.getRecipe(0).view();

    expect(recipe).to.have.property("itemType");
    expect(recipe).to.have.property("requirements");
    expect(recipe.requirements).to.be.an("array").with.length(6);
  });

  it("Get item price from marketplace", async () => {
    const price = await marketplaceProgram.methods
      .getItemPrice(0)
      .accounts({
        marketplaceConfig: marketplaceConfigPda,
      })
      .view();

    expect(price).to.equal(100n);
  });

  // ===== EXPANDED TEST SUITE =====

  it("Create player token accounts for resources", async () => {
    for (let i = 0; i < 6; i++) {
      const tokenAccount = await splToken.getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet as any,
        resourceMints[i],
        provider.publicKey
      );
      playerTokenAccounts.push(tokenAccount.address);
    }

    expect(playerTokenAccounts.length).to.equal(6);
  });

  it("Mint resources to player accounts", async () => {
    for (let i = 0; i < 6; i++) {
      const mintIx = await splToken.createMintToInstruction(
        resourceMints[i],
        playerTokenAccounts[i],
        provider.publicKey,
        BigInt(100)
      );

      const transaction = new anchor.web3.Transaction().add(mintIx);
      await provider.sendAndConfirm(transaction);
    }

    for (let i = 0; i < 6; i++) {
      const account = await splToken.getAccount(
        provider.connection,
        playerTokenAccounts[i]
      );
      expect(account.amount).to.equal(BigInt(100));
    }
  });

  it("Execute resource search", async () => {
    const resourceIndices = [0, 1, 2];

    // Create destination token accounts for search results
    const destinationAccounts = [];
    for (let i = 0; i < 3; i++) {
      const account = await splToken.getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet as any,
        resourceMints[i],
        provider.publicKey
      );
      destinationAccounts.push(account.address);
    }

    // Build remaining accounts: 6 mints + 3 destination accounts
    const remainingAccounts = [
      ...resourceMints.map((pubkey) => ({
        pubkey,
        isSigner: false,
        isWritable: false,
      })),
      ...destinationAccounts.map((pubkey) => ({
        pubkey,
        isSigner: false,
        isWritable: true,
      })),
    ];

    const tx = await searchProgram.methods
      .executeSearch(resourceIndices)
      .accounts({
        player: provider.publicKey,
        playerState: playerPda,
        searchAuthority: provider.publicKey,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
      })
      .remainingAccounts(remainingAccounts)
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Check search cooldown timer", async () => {
    const timeRemaining = await searchProgram.methods
      .timeUntilNextSearch()
      .accounts({
        player: provider.publicKey,
        playerState: playerPda,
      })
      .view();

    expect(timeRemaining).to.be.a("bigint");
    expect(timeRemaining > BigInt(0)).to.be.true;
  });

  it("Create NFT mint for crafting", async () => {
    const mint = anchor.web3.Keypair.generate();
    const transaction = new anchor.web3.Transaction();

    const createMintIx = await splToken.createInitializeMintInstruction(
      mint.publicKey,
      0,
      provider.publicKey,
      provider.publicKey
    );

    transaction.add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mint.publicKey,
        space: splToken.MINT_SIZE,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          splToken.MINT_SIZE
        ),
        programId: splToken.TOKEN_PROGRAM_ID,
      }),
      createMintIx
    );

    await provider.sendAndConfirm(transaction, [mint]);
    itemNftMint = mint.publicKey;

    itemDataPda = PublicKey.findProgramAddressSync(
      [Buffer.from("item"), itemNftMint.toBuffer()],
      itemNftProgram.programId
    )[0];

    expect(itemNftMint).to.not.be.null;
  });

  it("Mint NFT item", async () => {
    const playerItemAccount = await splToken.getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet as any,
      itemNftMint,
      provider.publicKey
    );

    const tx = await itemNftProgram.methods
      .mintItemNft(0, "ipfs://placeholder-uri")
      .accounts({
        player: provider.publicKey,
        craftingProgram: provider.publicKey,
        itemMint: itemNftMint,
        playerTokenAccount: playerItemAccount.address,
        itemData: itemDataPda,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Get item details from NFT program", async () => {
    const itemData = await itemNftProgram.methods
      .getItemDetails()
      .accounts({
        itemData: itemDataPda,
      })
      .view();

    expect(itemData).to.have.property("itemType");
    expect(itemData.itemType).to.equal(0);
    expect(itemData).to.have.property("owner");
    expect(itemData.owner).to.deep.equal(provider.publicKey);
  });

  it("Update marketplace prices (admin only)", async () => {
    const newPrices = [BigInt(200), BigInt(300), BigInt(400), BigInt(500)];

    const tx = await marketplaceProgram.methods
      .updatePrices(newPrices)
      .accounts({
        admin: provider.publicKey,
        marketplaceConfig: marketplaceConfigPda,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Verify updated prices", async () => {
    const price = await marketplaceProgram.methods
      .getItemPrice(0)
      .accounts({
        marketplaceConfig: marketplaceConfigPda,
      })
      .view();

    expect(price).to.equal(200n);
  });

  it("Transfer NFT ownership", async () => {
    const newOwner = anchor.web3.Keypair.generate().publicKey;

    const tx = await itemNftProgram.methods
      .transferItemOwnership(newOwner)
      .accounts({
        currentOwner: provider.publicKey,
        itemData: itemDataPda,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Verify ownership transfer", async () => {
    const itemData = await itemNftProgram.methods
      .getItemDetails()
      .accounts({
        itemData: itemDataPda,
      })
      .view();

    expect(itemData.owner).to.not.deep.equal(provider.publicKey);
  });

  it("Get player search info", async () => {
    const playerInfo = await searchProgram.methods
      .getPlayerSearchInfo()
      .accounts({
        player: provider.publicKey,
        playerState: playerPda,
      })
      .view();

    expect(playerInfo).to.have.property("owner");
    expect(playerInfo).to.have.property("searchCount");
    expect(playerInfo.searchCount > BigInt(0)).to.be.true;
  });

  it("Verify resource manager game config", async () => {
    const gameConfig = await resourceManagerProgram.account.gameSettings.fetch(
      gameSettingsPda
    );

    expect(gameConfig.resourceMints).to.have.lengthOf(6);
    expect(gameConfig.magicTokenMint).to.deep.equal(magicTokenMint);
    expect(gameConfig.itemPrices).to.have.lengthOf(4);
  });

  it("Verify marketplace config", async () => {
    const marketplaceConfig = await marketplaceProgram.account.marketplaceConfig.fetch(
      marketplaceConfigPda
    );

    expect(marketplaceConfig.magicTokenMint).to.deep.equal(magicTokenMint);
    expect(marketplaceConfig.magicTokenProgram).to.not.be.null;
    expect(marketplaceConfig.itemPrices).to.have.lengthOf(4);
  });

  it("Verify MagicToken authority", async () => {
    const tokenAuthority = await magicTokenProgram.account.tokenAuthority.fetch(
      tokenAuthorityPda
    );

    expect(tokenAuthority.tokenMint).to.deep.equal(magicTokenMint);
    expect(tokenAuthority).to.have.property("bump");
  });
});
