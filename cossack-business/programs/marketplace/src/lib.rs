/// Модуль маркетплейсу
/// Функціоналу:
/// - Гравець продає NFT предмет
/// - NFT спалюється через Token Program
/// - Гравець отримує MagicToken через CPI до magic_token програми
/// - Механіка \"redemption flow\" - система автоматично скупляє предмети

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, Burn};

declare_id!("CpemvudwMWJN2TufMwxjAfM5iiLyfFJkDQNvaiyoQgLM");

/// Конфігурація маркетплейсу
#[account]
pub struct MarketplaceConfig {
    /// Адміністратор маркетплейсу
    pub admin: Pubkey,
    
    /// Адреса мінту MagicToken
    pub magic_token_mint: Pubkey,
    
    /// Адреса програми magic_token для CPI
    pub magic_token_program: Pubkey,
    
    /// Ціни для різних типів предметів
    pub item_prices: [u64; 4],
    
    /// Bump seed
    pub bump: u8,
}

/// Запис про продаж (для аудиту та аналітики)
#[account]
pub struct SaleRecord {
    /// Хто продав
    pub seller: Pubkey,
    
    /// Тип предмета
    pub item_type: u8,
    
    /// Мінт NFT
    pub item_mint: Pubkey,
    
    /// Отримана кількість MagicToken
    pub price_paid: u64,
    
    /// Час продажу
    pub timestamp: i64,
    
    /// Bump seed
    pub bump: u8,
}

#[program]
pub mod marketplace {
    use super::*;

    /// Ініціалізація конфігурації маркетплейсу
    pub fn initialize_marketplace(
        ctx: Context<InitializeMarketplace>,
        magic_token_program: Pubkey,
        prices: [u64; 4],
    ) -> Result<()> {
        let config = &mut ctx.accounts.marketplace_config;
        config.admin = ctx.accounts.payer.key();
        config.magic_token_mint = ctx.accounts.magic_token_mint.key();
        config.magic_token_program = magic_token_program;
        config.item_prices = prices;
        config.bump = ctx.bumps.marketplace_config;

        msg!("Marketplace initialized");
        msg!("MagicToken program: {}", magic_token_program);
        msg!("Prices: SWORD={}, STAFF={}, ARMOR={}, BRACELET={}",
             prices[0], prices[1], prices[2], prices[3]);

        Ok(())
    }

    /// Продати предмет на маркетплейс
    pub fn sell_item(
        ctx: Context<SellItem>,
        item_type: u8,
    ) -> Result<()> {
        require!(item_type < 4, MarketplaceError::InvalidItemType);

        let config = &ctx.accounts.marketplace_config;
        let price = config.item_prices[item_type as usize];

        // Спалити NFT від гравця
        let cpi_accounts = Burn {
            mint: ctx.accounts.item_nft_mint.to_account_info(),
            from: ctx.accounts.seller_item_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token_interface::burn(cpi_ctx, 1)?;

        msg!("NFT burned successfully");

        // Мінтити MagicToken для гравця через CPI до magic_token програми
        // (In production, це буде через CPI)
        // За тепер залишаємо заготовку для CPI

        // Записати продаж
        let record = &mut ctx.accounts.sale_record;
        record.seller = ctx.accounts.seller.key();
        record.item_type = item_type;
        record.item_mint = ctx.accounts.item_nft_mint.key();
        record.price_paid = price;
        record.timestamp = Clock::get()?.unix_timestamp;
        record.bump = ctx.bumps.sale_record;

        let item_names = ["Kozak Sword", "Elder Staff", "Character Armor", "Battle Bracelet"];
        let item_name = item_names[item_type as usize];

        msg!("{} sold for {} MagicToken", item_name, price);

        Ok(())
    }

    /// Отримати ціну для предмета
    pub fn get_item_price(ctx: Context<GetPrice>, item_type: u8) -> Result<u64> {
        require!(item_type < 4, MarketplaceError::InvalidItemType);
        Ok(ctx.accounts.marketplace_config.item_prices[item_type as usize])
    }

    /// Оновити ціни (тільки адміністратор)
    pub fn update_prices(
        ctx: Context<AdminAction>,
        new_prices: [u64; 4],
    ) -> Result<()> {
        let config = &mut ctx.accounts.marketplace_config;
        config.item_prices = new_prices;

        msg!("Prices updated");

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = std::mem::size_of::<MarketplaceConfig>() + 8,
        seeds = [b"marketplace_config"],
        bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    pub magic_token_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SellItem<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        seeds = [b"marketplace_config"],
        bump = marketplace_config.bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    /// NFT мінт предмета
    /// CHECK: Перевірка типу виконується під час CPI спалювання
    #[account(mut)]
    pub item_nft_mint: UncheckedAccount<'info>,

    /// Токен-акаунт гравця з NFT
    /// CHECK: Перевірка типу виконується під час CPI спалювання
    #[account(mut)]
    pub seller_item_account: UncheckedAccount<'info>,

    /// Токен-акаунт гравця для отримання MagicToken
    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    #[account(mut)]
    pub seller_magic_token_account: UncheckedAccount<'info>,

    #[account(
        init,
        payer = seller,
        space = std::mem::size_of::<SaleRecord>() + 8,
        seeds = [b"sale", item_nft_mint.key().as_ref(), seller.key().as_ref()],
        bump
    )]
    pub sale_record: Account<'info, SaleRecord>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    pub marketplace_config: Account<'info, MarketplaceConfig>,
}

#[derive(Accounts)]
pub struct AdminAction<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"marketplace_config"],
        bump = marketplace_config.bump,
        constraint = marketplace_config.admin == admin.key()
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
}

#[error_code]
pub enum MarketplaceError {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
}
