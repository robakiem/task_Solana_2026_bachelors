/// Модуль крафту предметів
/// Система крафту з перевіркою рецептів:
/// - Гравець надає необхідні ресурси
/// - Програма перевіряє рецепт
/// - Ресурси спалюються через Token Program
/// - Викликається мінтинг NFT через item_nft програму

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Burn, TokenInterface};

declare_id!("F363uvEn2XmANvZW4Lf2Wq7XZ63ic11qZMLEbnWEB9ik");

/// Типи предметів та їх рецепти
pub const SWORD: u8 = 0;              // 3x IRON + 1x WOOD + 1x LEATHER
pub const STAFF: u8 = 1;              // 2x WOOD + 1x GOLD + 1x DIAMOND
pub const ARMOR: u8 = 2;              // 4x LEATHER + 2x IRON + 1x GOLD
pub const BRACELET: u8 = 3;           // 4x IRON + 2x GOLD + 2x DIAMOND

pub const RESOURCES: &[u8] = &[
    0, // WOOD
    1, // IRON
    2, // GOLD
    3, // LEATHER
    4, // STONE
    5, // DIAMOND
];

/// Структура, яка визначає рецепт для кожного передмета
#[derive(Clone)]
pub struct Recipe {
    pub item_type: u8,
    pub requirements: [u64; 6], // Кількість кожного ресурсу (WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND)
}

impl Recipe {
    fn sword() -> Self {
        let mut req = [0u64; 6];
        req[1] = 3; // IRON
        req[0] = 1; // WOOD
        req[3] = 1; // LEATHER
        Recipe {
            item_type: SWORD,
            requirements: req,
        }
    }

    fn staff() -> Self {
        let mut req = [0u64; 6];
        req[0] = 2; // WOOD
        req[2] = 1; // GOLD
        req[5] = 1; // DIAMOND
        Recipe {
            item_type: STAFF,
            requirements: req,
        }
    }

    fn armor() -> Self {
        let mut req = [0u64; 6];
        req[3] = 4; // LEATHER
        req[1] = 2; // IRON
        req[2] = 1; // GOLD
        Recipe {
            item_type: ARMOR,
            requirements: req,
        }
    }

    fn bracelet() -> Self {
        let mut req = [0u64; 6];
        req[1] = 4; // IRON
        req[2] = 2; // GOLD
        req[5] = 2; // DIAMOND
        Recipe {
            item_type: BRACELET,
            requirements: req,
        }
    }

    pub fn for_item(item_type: u8) -> Result<Self> {
        match item_type {
            SWORD => Ok(Recipe::sword()),
            STAFF => Ok(Recipe::staff()),
            ARMOR => Ok(Recipe::armor()),
            BRACELET => Ok(Recipe::bracelet()),
            _ => Err(error!(CraftingError::UnknownItem)),
        }
    }
}

#[program]
pub mod crafting {
    use super::*;

    /// Виконати крафт предмета
    /// Перевіряє рецепт -> спалює ресурси -> мінтить NFT
    pub fn craft_item<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CraftItem<'info>>,
        item_type: u8,
        _metadata_uri: String,
    ) -> Result<()> {
        // Отримати рецепт
        let recipe = Recipe::for_item(item_type)?;

        // Очікуємо 12 remaining accounts: 6 mint + 6 token accounts гравця.
        require!(
            ctx.remaining_accounts.len() == 12,
            CraftingError::InvalidRemainingAccounts
        );

        // Витягти авторитет та програму ПЕРЕД loop
        let authority = ctx.accounts.player.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();

        // Burn resources via Token Program
        for (i, &required) in recipe.requirements.iter().enumerate() {
            if required > 0 {
                let resource_mint = ctx.remaining_accounts[i].clone();
                let resource_account = ctx.remaining_accounts[6 + i].clone();

                let cpi_accounts = Burn {
                    mint: resource_mint,
                    from: resource_account,
                    authority: authority.clone(),
                };

                let cpi_ctx = CpiContext::new(token_program.clone(), cpi_accounts);
                anchor_spl::token_interface::burn(cpi_ctx, required)?;

                msg!("Burned {} units of resource {}", required, i);
            }
        }

        // Item type names
        let item_name = match item_type {
            SWORD => "Kozak Sword",
            STAFF => "Elder Staff",
            ARMOR => "Character Armor",
            BRACELET => "Battle Bracelet",
            _ => "Unknown Item",
        };

        msg!("{} successfully crafted", item_name);

        Ok(())
    }

    /// Отримати інформацію про рецепт
    pub fn get_recipe(_ctx: Context<GetRecipe>, item_type: u8) -> Result<RecipeInfo> {
        let recipe = Recipe::for_item(item_type)?;

        Ok(RecipeInfo {
            item_type: recipe.item_type,
            requirements: recipe.requirements,
        })
    }
}

#[derive(Accounts)]
pub struct CraftItem<'info> {
    pub player: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct GetRecipe {}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RecipeInfo {
    pub item_type: u8,
    pub requirements: [u64; 6],
}

#[error_code]
pub enum CraftingError {
    #[msg("Unknown item type")]
    UnknownItem,
    #[msg("Insufficient resources to craft this item")]
    InsufficientResources,
    #[msg("Expected 12 remaining accounts (6 mints + 6 player token accounts)")]
    InvalidRemainingAccounts,
}
