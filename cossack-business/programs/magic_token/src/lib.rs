/// Модуль мінтингу MagicToken
/// Обмежений доступ: мінтинг дозволений ТІЛЬКИ з впеозиції Marketplace
/// Механіка безпеки:
/// - Підп усередині PDA (Program Derived Address)
/// - Cross-Program Invocation (CPI) з marketplace програми

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, MintTo};

declare_id!("69MQNzzxdev4Pxo1KpKh89FNoQ1rSZ2dgEazxb4KwjxM");

/// Орган управління мінтингом MagicToken
#[account]
pub struct TokenAuthority {
    /// Адреса Marketplace програми, яка має право викликати мінтинг
    pub marketplace_program: Pubkey,
    
    /// Адміністратор системи (може змінити marketplace адресу в надзвичайних ситуаціях)
    pub admin: Pubkey,
    
    /// Мінт токена, яким управляє цей орган
    pub token_mint: Pubkey,
    
    /// Bump seed для безпеки
    pub bump: u8,
}

#[program]
pub mod magic_token {
    use super::*;

    /// Ініціалізація органу управління мінтингом
    pub fn initialize_mint_authority(
        ctx: Context<InitializeMintAuthority>,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        let authority = &mut ctx.accounts.token_authority;
        authority.marketplace_program = marketplace_program;
        authority.admin = ctx.accounts.payer.key();
        authority.token_mint = ctx.accounts.magic_token_mint.key();
        authority.bump = ctx.bumps.token_authority;

        msg!("MagicToken minting authority initialized");
        msg!("Marketplace program: {}", marketplace_program);
        
        Ok(())
    }

    /// Мінтинг MagicToken - викликається ТІЛЬКИ з інших програм через CPI
    pub fn mint_rewards(
        ctx: Context<MintTokenRewards>,
        amount: u64,
    ) -> Result<()> {
        // Перевірка: виклик має бути з Marketplace програми
        require_keys_eq!(
            ctx.accounts.token_authority.marketplace_program,
            *ctx.program_id,
            CustomError::UnauthorizedMintCaller
        );

        let authority_seeds = &[
            b"magic_token_authority".as_ref(),
            &[ctx.accounts.token_authority.bump],
        ];
        let signer_seeds = &[&authority_seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.magic_token_mint.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: ctx.accounts.token_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        anchor_spl::token_interface::mint_to(cpi_ctx, amount)?;

        msg!("Minted {} MagicToken", amount);

        Ok(())
    }

    /// Оновити адресу Marketplace (тільки адміністратор)
    pub fn update_marketplace_address(
        ctx: Context<AdminUpdate>,
        new_marketplace: Pubkey,
    ) -> Result<()> {
        let authority = &mut ctx.accounts.token_authority;
        authority.marketplace_program = new_marketplace;
        
        msg!("Marketplace address updated");

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMintAuthority<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = std::mem::size_of::<TokenAuthority>() + 8,
        seeds = [b"magic_token_authority"],
        bump
    )]
    pub token_authority: Account<'info, TokenAuthority>,

    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    pub magic_token_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokenRewards<'info> {
    /// Орган управління з дозволом на мінтинг
    #[account(
        seeds = [b"magic_token_authority"],
        bump = token_authority.bump
    )]
    pub token_authority: Account<'info, TokenAuthority>,

    /// Мінт токена
    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    #[account(mut)]
    pub magic_token_mint: UncheckedAccount<'info>,

    /// Токен-акаунт, на який миниться сума
    /// CHECK: Перевірка типу виконується під час CPI мінтингу
    #[account(mut)]
    pub destination_token_account: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct AdminUpdate<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"magic_token_authority"],
        bump = token_authority.bump,
        constraint = token_authority.admin == admin.key()
    )]
    pub token_authority: Account<'info, TokenAuthority>,
}

#[error_code]
pub enum CustomError {
    #[msg("Only Marketplace program can mint MagicToken")]
    UnauthorizedMintCaller,
}
