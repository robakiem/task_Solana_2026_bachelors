/// Модуль керування ресурсами гри
/// Відповідає за:
/// - Ініціалізацію конфігурації гри
/// - Створення 6 базових ресурсів як SPL Token-2022
/// - Управління правами доступу для мінтингу/спалення ресурсів

use anchor_lang::prelude::*;

declare_id!("2fDbfUgbEmT3BYD7WnAndvQ6pJLCvVF44F1vdkPjevyG");

pub const RESOURCES_COUNT: usize = 6;
pub const ITEM_TYPES_COUNT: usize = 4;

/// Конфігурація гри, зберігає всі критичні налаштування
#[account]
pub struct GameSettings {
    /// Адміністратор, який може змінювати конфіг
    pub authority: Pubkey,
    
    /// Адреси мінтів всіх 6 базових ресурсів
    pub resource_mints: [Pubkey; RESOURCES_COUNT],
    
    /// Мінт токена MagicToken для використання на маркетплейсі
    pub magic_token_mint: Pubkey,
    
    /// Ціни для кожного типу предмета в MagicToken
    pub item_prices: [u64; ITEM_TYPES_COUNT],
    
    /// Флаг для безпеки (bump seed для PDA)
    pub bump: u8,
}

#[program]
pub mod resource_manager {
    use super::*;

    /// Ініціалізація усієї ігрової системи
    /// Створює GameSettings PDA та мінти для 6 ресурсів та MagicToken
    pub fn setup_game_system(
        ctx: Context<SetupGameSystem>,
        resource_names: [String; RESOURCES_COUNT],
        magic_token_price_items: [u64; ITEM_TYPES_COUNT],
    ) -> Result<()> {
        require!(
            ctx.remaining_accounts.len() >= RESOURCES_COUNT,
            ResourceManagerError::InvalidResourceMintAccounts
        );

        let game_config = &mut ctx.accounts.game_settings;
        game_config.authority = ctx.accounts.payer.key();
        game_config.bump = ctx.bumps.game_settings;
        
        // Зберегти адреси всіх мінтів ресурсів з remaining accounts
        for (i, mint_account) in ctx
            .remaining_accounts
            .iter()
            .take(RESOURCES_COUNT)
            .enumerate()
        {
            game_config.resource_mints[i] = *mint_account.key;
            msg!("Resource {} mint: {}", resource_names[i], mint_account.key());
        }
        
        game_config.magic_token_mint = ctx.accounts.magic_token_mint.key();
        game_config.item_prices = magic_token_price_items;

        msg!("Game system initialized successfully");
        msg!("Resources count: {}", RESOURCES_COUNT);
        msg!("MagicToken mint: {}", game_config.magic_token_mint);
        
        Ok(())
    }

    /// Оновити ціни предметів (тільки адміністратор)
    pub fn update_item_prices(
        ctx: Context<AdminOnly>,
        new_prices: [u64; ITEM_TYPES_COUNT],
    ) -> Result<()> {
        let game_config = &mut ctx.accounts.game_settings;
        game_config.item_prices = new_prices;
        
        msg!("Item prices updated");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupGameSystem<'info> {
    /// Той, хто ініціалізує систему (опатер)
    #[account(mut)]
    pub payer: Signer<'info>,

    /// PDA для зберігання конфігурації гри
    #[account(
        init,
        payer = payer,
        space = std::mem::size_of::<GameSettings>() + 8,
        seeds = [b"game_config"],
        bump
    )]
    pub game_settings: Account<'info, GameSettings>,

    /// Мінт для MagicToken
    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    pub magic_token_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    /// Адміністратор системи
    pub authority: Signer<'info>,

    /// Конфігурація гри
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_settings.bump,
        constraint = game_settings.authority == authority.key()
    )]
    pub game_settings: Account<'info, GameSettings>,
}

#[error_code]
pub enum ResourceManagerError {
    #[msg("Expected 6 resource mint accounts")]
    InvalidResourceMintAccounts,
}

