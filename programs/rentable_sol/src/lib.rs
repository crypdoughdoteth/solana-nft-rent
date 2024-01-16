use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

declare_id!("Hus6vJsPgoTE86HUVzaJfJKZM8kfrk6y5LMwbGKhtr8H");

#[program]
pub mod rentable_sol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct RentableToken {
   token_owner: Pubkey, 

   wrapper_outstanding: bool, 
   locked: bool, 
   price: u64,
   expiration: i64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub rentable_token_pda: Account<'info, RentableToken>,
    #[account(
        has_one = owner,
        
    )]
    pub token: Account<'info, TokenAccount>, 
    pub owner: Signer<'info>,
}

// why have two flags -- wrapper_outstanding and locked? 
// the token can be locked without having the wrapper outstanding
// this is not an illegal state since the Owner has control but nobody
// yet came along to borrow the asset.
