/// Модуль пошуку ресурсів
/// Механіка:
/// - Гравець створює Player PDA (один для кожного гравця)
/// - Пошук дозволений раз на 60 секунд
/// - За один пошук гравець отримує 3 випадкових ресурси
/// - Час перевіряється через PDA з timestamp

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, MintTo};

declare_id!("97CsecsErAgRJsbLVaRitisJCYbenpUGAkaJTdDeGdn8");

pub const SEARCH_COOLDOWN: i64 = 60; // 60 секунд
pub const RESOURCES_PER_SEARCH: u64 = 3;

/// Дані гравця з врахуванням часу останнього пошуку
#[account]
pub struct PlayerSearchState {
    /// Власник цього акаунту
    pub owner: Pubkey,
    
    /// Час успішного пошуку (Unix timestamp)
    pub last_search_time: i64,
    
    /// Кількість успішних пошуків
    pub search_count: u64,
    
    /// Bump seed
    pub bump: u8,
}

#[program]
pub mod search {
    use super::*;

    /// Ініціалізація стану гравця для пошуку
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        let player_state = &mut ctx.accounts.player_state;
        player_state.owner = ctx.accounts.payer.key();
        player_state.last_search_time = 0;
        player_state.search_count = 0;
        player_state.bump = ctx.bumps.player_state;

        msg!("Player initialized for resource search");
        msg!("Player: {}", ctx.accounts.payer.key());

        Ok(())
    }

    /// Виконати пошук ресурсів
    /// Виконується через resource_manager для мінтингу
    pub fn execute_search<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ExecuteSearch<'info>>,
        resource_indices: [u8; 3],
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let player_state = &mut ctx.accounts.player_state;

        // Перевірка cooldown-у
        require!(
            current_time >= player_state.last_search_time + SEARCH_COOLDOWN,
            SearchError::SearchOnCooldown
        );

        // Перевірка індексів (мають бути від 0 до 5)
        for idx in resource_indices.iter() {
            require!(*idx < 6, SearchError::InvalidResourceIndex);
        }

        // Очікуємо 9 remaining accounts: 6 мінтів + 3 токен-акаунти
        require!(
            ctx.remaining_accounts.len() >= 9,
            SearchError::InvalidRemainingAccounts
        );

        // Оновити час останнього пошуку
        player_state.last_search_time = current_time;
        player_state.search_count += 1;

        // Витягти всі дані з ctx ПЕРЕД loop щоб уникнути borrow checker issues
        let authority_key = ctx.accounts.search_authority.key();
        let token_program_key = ctx.accounts.token_program.to_account_info().key();

        // Витягти всі remaining accounts перед loop
        let mut mint_accounts: Vec<AccountInfo> = Vec::new();
        let mut token_accounts: Vec<AccountInfo> = Vec::new();
        for i in 0..6 {
            mint_accounts.push(ctx.remaining_accounts[i].clone());
        }
        for i in 0..3 {
            token_accounts.push(ctx.remaining_accounts[6 + i].clone());
        }

        // Тепер можемо безпечно獨립ити loop без ctx
        // Шукаємо авторитет та програму у remaining accounts
        let authority_account = ctx.remaining_accounts.iter()
            .find(|acc| acc.key() == authority_key)
            .ok_or(error!(SearchError::InvalidRemainingAccounts))?
            .clone();

        let token_program = ctx.remaining_accounts.iter()
            .find(|acc| acc.key() == token_program_key)
            .unwrap_or(&ctx.accounts.token_program.to_account_info())
            .clone();

        // Мінтити 3 ресурси на токен-акаунти гравця
        for (i, resource_idx) in resource_indices.iter().enumerate() {
            let mint_account = mint_accounts[*resource_idx as usize].clone();
            let token_account = token_accounts[i].clone();

            let cpi_accounts = MintTo {
                mint: mint_account,
                to: token_account,
                authority: authority_account.clone(),
            };

            let cpi_ctx = CpiContext::new(token_program.clone(), cpi_accounts);

            anchor_spl::token_interface::mint_to(cpi_ctx, RESOURCES_PER_SEARCH)?;

            msg!("Minted resource {} to player", resource_idx);
        }

        msg!("Search completed! Total searches: {}", player_state.search_count);
        msg!("Time until next search: {} seconds", SEARCH_COOLDOWN);

        Ok(())
    }

    /// Отримати інформацію про стан гравця
    pub fn get_player_search_info(ctx: Context<GetPlayerInfo>) -> Result<PlayerSearchState> {
        Ok((*ctx.accounts.player_state).clone())
    }

    /// Обчислити час до наступного пошуку
    pub fn time_until_next_search(ctx: Context<GetPlayerInfo>) -> Result<i64> {
        let current_time = Clock::get()?.unix_timestamp;
        let player_state = &ctx.accounts.player_state;
        
        let next_search_time = player_state.last_search_time + SEARCH_COOLDOWN;
        let time_remaining = if current_time < next_search_time {
            next_search_time - current_time
        } else {
            0
        };

        Ok(time_remaining)
    }
}

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = std::mem::size_of::<PlayerSearchState>() + 8,
        seeds = [b"player", payer.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerSearchState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteSearch<'info> {
    pub player: Signer<'info>,

    /// PDA з правом на мінтинг (створюється resource_manager)
    pub search_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", player.key().as_ref()],
        bump = player_state.bump
    )]
    pub player_state: Account<'info, PlayerSearchState>,


    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct GetPlayerInfo<'info> {
    pub player: Signer<'info>,

    pub player_state: Account<'info, PlayerSearchState>,
}

#[error_code]
pub enum SearchError {
    #[msg("Search is on cooldown. Wait 60 seconds between searches")]
    SearchOnCooldown,
    #[msg("Invalid resource index (must be 0-5)")]
    InvalidResourceIndex,
    #[msg("Expected 9 remaining accounts (6 resource mints + 3 destination token accounts)")]
    InvalidRemainingAccounts,
}
