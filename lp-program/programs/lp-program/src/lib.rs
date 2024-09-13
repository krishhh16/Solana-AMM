use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_lang::system_program;

declare_id!("CFqZq4eVNHpjntzo1vF2fjUF1VwSGn81SoEngv559syq");

#[program]
pub mod lp_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,pool_id:u64, sol_amount: u64, token_amount: u64, lp_incentive: u64 ) -> Result<()> {

        let pool = &mut ctx.accounts.pool;
        
        // Converting the sol transfered to the program into wsol so that we don't have to handle the sol and token seperately

        // sends the sol to wsol_vault
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.creator.to_account_info(),
                    to: ctx.accounts.wsol_vault.to_account_info()
                }
            ),
            sol_amount
        )?;


        // mints the wsol token to the user's associate token account(Which will be created in before calling this function in TS)
        token::mint_to(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), token::MintTo{
                mint: ctx.accounts.wsol_mint.to_account_info(),
                to: ctx.accounts.creator_wsol_account.to_account_info(),
                authority: ctx.accounts.wsol_vault.to_account_info()
            } , &[&[b"wsol_vault", &[ctx.bumps.wsol_vault]]],
        ),
            sol_amount
        )?;

        // Logic to send the respective tokens to the pda now.
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.creator_wsol_account.to_account_info(),
                    to: ctx.accounts.pool_wsol_account.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info()
                }
            )
            , sol_amount)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer{ 
                    from: ctx.accounts.creator_spl_account.to_account_info(),
                    to: ctx.accounts.pool_spl_account.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info()
                }
            ),
            token_amount
        )?;

        // Logic to send the creator the LP tokens to the provider
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.lp_token_mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            6, 
            ctx.accounts.lp_token_pda.key,
            Some(ctx.accounts.lp_token_pda.key)
        )?;

        token::initialize_account(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeAccount {
                    account: ctx.accounts.creator_lp_token_account.to_account_info(),
                    mint: ctx.accounts.lp_token_mint.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info()
                }
            )
        )?;

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo{
                    mint: ctx.accounts.lp_token_mint.to_account_info(),
                    to: ctx.accounts.creator_lp_token_account.to_account_info(),
                    authority: ctx.accounts.lp_token_pda.to_account_info()
                }
                , 
                &[&[b"lp_token_authority", &[ctx.bumps.lp_token_pda]]],
                ),
                lp_incentive
        )?;

        

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
        seeds = [b"wsol_valut",creator.key().as_ref(), &pool_id.to_le_bytes().as_ref() ],
        bump
    )]
    pub wsol_vault: SystemAccount<'info>, // The account that will facilitate the transfer of sol and hold the user funds 
    pub wsol_mint: Account<'info, Mint>, // The mint address of the wsol account
    #[account(mut)]
    pub creator_wsol_account: Account<'info, TokenAccount>, // user's associate token account that hold the user's wsol
    pub token_program: Program<'info, Token>, // Toke program address
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub creator: Signer<'info>, // Signer
    #[account(mut)]
    pub creator_spl_account: Account<'info, TokenAccount>, // this is the associate token account that owns the specific token that the user is trying to create the value of
    #[account(mut)]
    pub pool_spl_account: Account<'info, TokenAccount>, // spl ATA for the pool
    #[account(mut)]
    pub pool_wsol_account: Account<'info, TokenAccount>, // wsol ATA for the pool
    pub rent: Sysvar<'info, Rent>,  
    #[account(
        seeds= [b"lp_token_authority"],
        bump
    )]
    pub lp_token_pda: UncheckedAccount<'info>,
    pub lp_token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = creator,
        token::mint = lp_token_mint,
        token::authority = creator,
    )]
    pub creator_lp_token_account: Account<'info, TokenAccount>,
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
