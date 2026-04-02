/// Модуль управління предметами як NFT (Non-Fungible Tokens)
/// Функціонал:
/// - Створення унікальних NFT предметів з типами
/// - Інтеграція з Metaplex для метаданих
/// - Отримання інформації про предмет та його власника

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, MintTo};

declare_id!("7bgkpNNCBumC1GFLDLbpnwGTkfzFoBv1dia9a7h4noVS");

/// Типи предметів в грі
pub const SWORD: u8 = 0;              // Шабля козака
pub const STAFF: u8 = 1;              // Посох старійшини
pub const ARMOR: u8 = 2;              // Броня характерника
pub const BRACELET: u8 = 3;           // Бойовий браслет

/// Метаінформація про NFT предмет, прив'язана до його мінту
#[account]
pub struct ItemData {
    /// Тип предмета (0-3)
    pub item_type: u8,
    
    /// Власник предмета
    pub owner: Pubkey,
    
    /// Адреса мінту цього NFT
    pub mint_address: Pubkey,
    
    /// Час створення (для можливого лвелування)
    pub created_at: i64,
    
    /// Bump seed для безпеки
    pub bump: u8,
}

#[program]
pub mod item_nft {
    use super::*;

    /// Створення нового NFT предмета
    /// Викликається з програми Crafting після перевірки рецепту
    pub fn mint_item_nft(
        ctx: Context<MintItemNFT>,
        item_type: u8,
        _metadata_uri: String,
    ) -> Result<()> {
        require!(item_type < 4, ItemError::InvalidItemType);

        // Мінтинг NFT (1 копія)
        let cpi_accounts = MintTo {
            mint: ctx.accounts.item_mint.to_account_info(),
            to: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.crafting_program.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        anchor_spl::token_interface::mint_to(cpi_ctx, 1)?;

        // Записання метаданих про предмет
        let item_data = &mut ctx.accounts.item_data;
        item_data.item_type = item_type;
        item_data.owner = ctx.accounts.player.key();
        item_data.mint_address = ctx.accounts.item_mint.key();
        item_data.created_at = Clock::get()?.unix_timestamp;
        item_data.bump = ctx.bumps.item_data;

        let item_type_name = match item_type {
            SWORD => "Kozak Sword",
            STAFF => "Elder Staff",
            ARMOR => "Character Armor",
            BRACELET => "Battle Bracelet",
            _ => "Unknown Item",
        };

        msg!("NFT {} created successfully", item_type_name);
        msg!("Mint: {}", ctx.accounts.item_mint.key());
        msg!("Owner: {}", ctx.accounts.player.key());

        Ok(())
    }

    /// Отримати інформацію про предмет по його мінту
    pub fn get_item_details(ctx: Context<GetItemDetails>) -> Result<ItemData> {
        Ok((*ctx.accounts.item_data).clone())
    }

    /// Передача прав власності (для майбутніх торгів між гравцями)
    pub fn transfer_item_ownership(
        ctx: Context<TransferOwnership>,
        new_owner: Pubkey,
    ) -> Result<()> {
        let item_data = &mut ctx.accounts.item_data;
        require_keys_eq!(item_data.owner, ctx.accounts.current_owner.key(), ItemError::NotItemOwner);
        
        item_data.owner = new_owner;
        
        msg!("Item ownership transferred to {}", new_owner);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintItemNFT<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// Програма Crafting, яка має дозвіл мінтити
    pub crafting_program: Signer<'info>,

    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    #[account(mut)]
    pub item_mint: UncheckedAccount<'info>,

    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    #[account(
        init,
        payer = player,
        space = std::mem::size_of::<ItemData>() + 8,
        seeds = [b"item", item_mint.key().as_ref()],
        bump
    )]
    pub item_data: Account<'info, ItemData>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetItemDetails<'info> {
    pub item_data: Account<'info, ItemData>,
}

#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    pub current_owner: Signer<'info>,

    #[account(
        mut,
        constraint = item_data.owner == current_owner.key()
    )]
    pub item_data: Account<'info, ItemData>,
}

#[error_code]
pub enum ItemError {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    #[msg("Only item owner can perform this action")]
    NotItemOwner,
}
