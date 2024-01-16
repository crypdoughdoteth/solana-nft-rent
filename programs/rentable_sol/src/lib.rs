use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Hus6vJsPgoTE86HUVzaJfJKZM8kfrk6y5LMwbGKhtr8H");

#[program]
pub mod rentable_sol {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        owner: Pubkey,
        price: u64,
        expiration_date: i64,
    ) -> Result<()> {
        // create pda
        // set state with lending data
        // tranfer token to TokenAccount of this program
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // set state of pda
        let rent_data = &mut ctx.accounts.rentable_token_pda;
        rent_data.token_owner = owner;
        rent_data.price = price;
        rent_data.expiration = expiration_date;
        rent_data.bump = ctx.bumps.rentable_token_pda;
        // execute transfer of NFT to Program's ATA
        token::transfer(
            // Create new Cross Program Invocation Context
            CpiContext::new(
                // Token program
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    // ATA for mint_key + ctx.program_id
                    to: ctx.accounts.to_token_account.to_account_info(),
                    authority: ctx.accounts.from.to_account_info(),
                },
            ),
            1,
        )?;
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct RentableToken {
    token_owner: Pubkey,
    renter: Option<Pubkey>,
    associated_token_acc: Pubkey,
    locked: bool,
    price: u64,
    expiration: i64,
    bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub from: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = owner,
        space = 1 + 32 + 32 + 32 + 8 + 8, seeds = [b"rentable-tokens", owner.key().as_ref()], bump
    )]
    pub rentable_token_pda: Account<'info, RentableToken>,
    pub from_token_account: Account<'info, TokenAccount>,
    // ATA for mint_key + ctx.program_id
    pub to_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
