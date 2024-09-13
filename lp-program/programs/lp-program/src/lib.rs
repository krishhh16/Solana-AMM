use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_lang::system_program;

declare_id!("CFqZq4eVNHpjntzo1vF2fjUF1VwSGn81SoEngv559syq");

#[program]
pub mod lp_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,pool_id:u64, sol_amount: u64) -> Result<()> {

        let pool = &mut ctx.accounts.pool;
        
        // Converting the sol transfered to the program into wsol so that we don't have to handle the sol and token seperately
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.creator.to_account_info(),
                    to: ctx.accounts.wsol_vault.to_account_info()
                }
            ),
            sol_amount
        );

        token::mint_to(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), token::MintTo{
                mint: ctx.accounts.wsol_mint.to_account_info(),
                to: ctx.accounts.user_wsol_account.to_account_info(),
                authority: ctx.accounts.wsol_vault.to_account_info()
            } , &[&[b"wsol_vault", &[ctx.bumps.wsol_vault]]],
        ),
            sol_amount
        );

        

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(pool_id:u64)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = creator, 
        space = 8 + 8 + 32 + 32 + 32, 
        seeds=[b"poolinitiate",creator.key().as_ref(), &pool_id.to_le_bytes().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut, 
        seeds = [b"wsol_valut"],
        bump
    )]
    pub wsol_vault: SystemAccount<'info>,
    pub wsol_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_wsol_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub creator: Signer<'info>
}

#[account]
pub struct Pool {
   pub pool_id: u64,
   pub token_a_reserve: Pubkey,
   pub token_b_reserve: Pubkey,
   pub lp_mint: Pubkey
}

pub struct LiquidityProvider {
    pub address: Pubkey,
    pub pool_id: u64,
    pub lp_token_share: u64,
}
