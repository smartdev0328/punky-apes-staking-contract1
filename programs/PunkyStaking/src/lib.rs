use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_lang::solana_program::{clock};
use std::convert::Into;
use crate::constants::*;
declare_id!("3n7nc4FUKJRxhovqwxV9XfYahRcjbxwnaFp1ZSXWqAZ1");

mod constants {

    pub const DAY_TIME: u32 = 60;
    pub const LIFE_TIME: u32 = 8 * DAY_TIME;
    pub const DECIMAL: u64 = 1000000000;
    pub const DEFAULT_REWARD: u64 = 8 * DECIMAL;
    pub const BEFORE_LIFETIME_REWARD: u64 = 0;
    pub const DEFAULT_TOKEN_NUMBER: u64 = 100 * DECIMAL;
}

#[program]
pub mod punky_staking {
    use super::*;

    pub fn create_vault(_ctx: Context<CreateVaultContext>, bump_vault: u8) -> ProgramResult {
        Ok(())
    }

    pub fn create_data_account(ctx: Context<CreateDataContext>, bump_data: u8) -> ProgramResult {
        let data = &mut ctx.accounts.data;
        data.nft_first = 0;
        Ok(())
    }
    
    pub fn create_pool_signer(_ctx: Context<CreatePoolSignerContext>, bump_signer: u8) -> ProgramResult {
        Ok(())
    }

    pub fn create_pool(ctx: Context<CreatePoolContext>, bump_pool: u8) -> ProgramResult {
        let pool = &mut ctx.accounts.pool;
        pool.user = ctx.accounts.user.key();
        pool.mint = ctx.accounts.mint.key();
        Ok(())
    }

    pub fn stake(ctx: Context<StakeContext>, token_type: u8, attribute: u8) -> ProgramResult {
        let pool = &mut ctx.accounts.pool;

        if pool.user != ctx.accounts.user.key() || pool.mint != ctx.accounts.mint.key() {
            return Err(ErrorCode::AuthorityInvalid.into());
        }

        let data = &mut ctx.accounts.data;
        let clock = clock::Clock::get().unwrap();
        let mut cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
                authority: ctx.accounts.user.to_account_info()
            }
        );
        token::transfer(cpi_ctx, 1)?;
        
        pool.last_time = clock.unix_timestamp as u32;
        pool.reward = DEFAULT_REWARD;
        pool.token_type = token_type;
        
        //reward token moving
        cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from.to_account_info(),
                to: ctx.accounts.token_to.to_account_info(),
                authority: ctx.accounts.user.to_account_info()
            },

        );
        token::transfer(cpi_ctx, DEFAULT_TOKEN_NUMBER)?;

        
        if token_type == 0 {
            data.nft_first += 1;
        }
        Ok(())
    }

    pub fn unstake(ctx: Context<UnstakeContext>, bump_signer: u8, bump_vault: u8) -> ProgramResult {
        let pool =  &mut ctx.accounts.pool;
        if pool.user != ctx.accounts.user.key() || pool.mint != ctx.accounts.mint.key() {
            return Err(ErrorCode::AuthorityInvalid.into());
        }

        let data = &mut ctx.accounts.data;
        let clock = clock::Clock::get().unwrap();

        let pool_signer_seeds = &[
            b"NFT STAKING POOL SIGNER".as_ref(),
            ctx.accounts.user.to_account_info().key.as_ref(),
            &[bump_signer],
        ];

        let pool_signer = &[&pool_signer_seeds[..]];

        let mut cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.nft_from.to_account_info(),
                to: ctx.accounts.nft_to.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info()
            },
            pool_signer
        );

        token::transfer(cpi_ctx, 1)?;

        let vault_seeds = &[
            b"NFT STAKING VAULT".as_ref(),
            &[bump_vault],
        ];

        let vault_signer = &[&vault_seeds[..]];

        cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from.to_account_info(),
                to: ctx.accounts.token_to.to_account_info(),
                authority: ctx.accounts.vault.to_account_info()
            },
            vault_signer
        );
        
        

        let life_time = clock.unix_timestamp as u32 - pool.last_time;
      
        if life_time < LIFE_TIME {
            pool.reward = BEFORE_LIFETIME_REWARD;
        }
        let days: u32 = life_time / DAY_TIME;
        let total_reward =  days as u64 * pool.reward + DEFAULT_TOKEN_NUMBER;
        token::transfer(cpi_ctx, total_reward.into())?;

        if pool.token_type == 0 && data.nft_first > 0 {
            data.nft_first -= 1;
        }
        Ok(())
    }

    pub fn claim(ctx: Context<ClaimContext>, bump_vault: u8) -> ProgramResult {
        let pool =  &mut ctx.accounts.pool;
        if pool.user != ctx.accounts.user.key() || pool.mint != ctx.accounts.mint.key() {
            return Err(ErrorCode::AuthorityInvalid.into());
        }

        let clock = clock::Clock::get().unwrap();
        let vault_seeds = &[
            b"NFT STAKING VAULT".as_ref(),
            &[bump_vault],
        ];

        let vault_signer = &[&vault_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from.to_account_info(),
                to: ctx.accounts.token_to.to_account_info(),
                authority: ctx.accounts.vault.to_account_info()
            },
            vault_signer
        );

        let life_time = clock.unix_timestamp as u32 - pool.last_time ;
      
        if life_time < LIFE_TIME {
            pool.reward = BEFORE_LIFETIME_REWARD;
        }

        let days: u32 = life_time / DAY_TIME;
        let total_reward =  days as u64 * pool.reward;
        token::transfer(cpi_ctx, total_reward.into())?;
        pool.last_time = clock.unix_timestamp as u32;
        pool.reward = DEFAULT_REWARD;
        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateVaultContext<'info> {
    #[account(init, seeds = [b"NFT STAKING VAULT".as_ref()], bump, payer = admin, space = 8 + 1)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateDataContext<'info> {
    #[account(init, seeds = [b"NFT STAKING DATA".as_ref()], bump, payer = admin, space = 8 + 8)]
    pub data: Account<'info, Data>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreatePoolSignerContext<'info> {
    #[account(init, seeds = [b"NFT STAKING POOL SIGNER".as_ref(), user.key.as_ref()], bump, payer = user, space = 8 + 1)]
    pub pool_signer: Account<'info, PoolSigner>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreatePoolContext<'info> {
    #[account(init, seeds = [b"NFT STAKING POOL".as_ref(), user.key.as_ref(), mint.key().as_ref()], bump, payer = user, space = 8 + 32 + 32 + 8 + 4 + 1)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct StakeContext<'info> {
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub data: Account<'info, Data>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_from: Box<Account<'info, TokenAccount>>, // user token account
    #[account(mut)]
    pub token_to: Box<Account<'info, TokenAccount>>, // vault token account
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,//for init
}

#[derive(Accounts)]
pub struct UnstakeContext<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub pool_signer: Account<'info, PoolSigner>,
    pub vault: Account<'info, Vault>, // this vault account
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub data: Account<'info, Data>,
    #[account(mut)]
    pub nft_from: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub nft_to: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub token_from: Box<Account<'info, TokenAccount>>, // vault token account
    #[account(mut)]
    pub token_to: Box<Account<'info, TokenAccount>>, // user token account
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct ClaimContext<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub vault: Account<'info, Vault>, // this vault account
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_from: Box<Account<'info, TokenAccount>>, // vault token account
    #[account(mut)]
    pub token_to: Box<Account<'info, TokenAccount>>, // user token account
    pub token_program: Program<'info, Token>
}

#[account]
pub struct Data {
    pub nft_first: u32,
}

#[account]
pub struct Vault {
    pub bump_vault: u8
}

#[account]
pub struct PoolSigner {
    pub bump_signer: u8
}
#[account]
pub struct Pool {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub reward: u64,
    pub last_time: u32,
    pub token_type: u8
}


#[error]
pub enum ErrorCode {
    #[msg("Authority is invalid")]
    AuthorityInvalid,
    #[msg("Index out of range")]
    OutRange,
    #[msg("Invalid attribute")]
    InvalidAttribute,
    #[msg("Invalid token")]
    InvalidToken,
}