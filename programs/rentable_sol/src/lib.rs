use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{token::{self, Token, TokenAccount, Transfer, Mint}, associated_token::AssociatedToken};

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
                    to: ctx.accounts.to_token_account.to_account_info(),
                    authority: ctx.accounts.from.to_account_info(),
                },
            ),
            1,
        )?;
        Ok(())
    }

    pub fn borrow(ctx: Context<Borrow>) -> Result<()> {
        // pay in lamports for the rental
        // require locked == true
        // set renter to Some(caller)

        let state = &mut ctx.accounts.rentable_token_pda;
        require!(
            ctx.accounts.system_program.signer_key() != None,
            Errors::NoSigner
        );
        state.renter = Some(*ctx.accounts.system_program.signer_key().unwrap());
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, ctx.accounts.rentable_token_pda.price)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let state = &mut ctx.accounts.rentable_token_pda;
        let clock = Clock::get()?;
        if let Some(_) = state.renter {
            require!( clock.unix_timestamp >= state.expiration, Errors::NotExpired );
        }
        require!( ctx.program_id == &ctx.accounts.lamports_from.key(), Errors::WrongAddress ); 
        token::transfer(
            // Create new Cross Program Invocation Context
            CpiContext::new(
                // Token program
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from_token_account.to_account_info(),
                    to: ctx.accounts.to_token_account.to_account_info(),
                    authority: ctx.accounts.from_token_account.to_account_info(),
                },
            ),
            1,
        )?;

        // dump the lamports to the original payer and mark account for garbage collection
        let lamports_from_account = &ctx.accounts.lamports_from;
        let lamports_to_account = &ctx.accounts.lamports_from;
        let balance = lamports_from_account.to_account_info().lamports(); 
        if **lamports_from_account.try_borrow_lamports()? < balance {
           return err!(Errors::InsufficientBalance); 
        }
        **lamports_from_account.try_borrow_mut_lamports()? -= balance;
        **lamports_to_account.try_borrow_mut_lamports()? += balance;    
        Ok(()) 
    }

}

pub fn active_rental(ctx: Context<ActiveRental>) -> Result<()> {
    let state = &ctx.accounts.rentable_token_pda;
    let clock = Clock::get()?;
    require!((clock.unix_timestamp < state.expiration) && state.renter.is_some(), Errors::NotExpired);
    Ok(())
}


type Timestamp = i64;

#[account]
#[derive(Default)]
pub struct RentableToken {
    pub token_owner: Pubkey,
    pub renter: Option<Pubkey>,
    pub associated_token_acc: Pubkey,
    pub price: u64,
    pub expiration: Timestamp,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub from: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = owner,
        space = 1 + 32 + 32 + 32 + 8 + 8, seeds = [b"rentable-tokens", owner.key().as_ref()], 
        bump
    )]
    pub rentable_token_pda: Account<'info, RentableToken>,
    pub from_token_account: Account<'info, TokenAccount>,
    // ATA for mint_key + ctx.program_id
    #[account(
        init, 
        payer = owner,
        associated_token::mint = mint, 
        associated_token::authority = owner,    
    )]
    pub to_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub rentable_token_pda: Account<'info, RentableToken>,
    pub system_program: Program<'info, System>,
    pub signer: Signer<'info>,
    /// CHECK: NO R/W
    pub from: AccountInfo<'info>,
    /// CHECK: NO R/W
    pub to: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account( mut )]
    pub rentable_token_pda: Account<'info, RentableToken>,
    #[account(address = rentable_token_pda.token_owner)]
    pub signer: Signer<'info>,
    #[account(address = rentable_token_pda.associated_token_acc)]
    pub from_token_account: Account<'info, TokenAccount>,
    #[account(address = rentable_token_pda.token_owner)]
    pub to_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: NO R/W
    pub lamports_from: AccountInfo<'info>,
    /// CHECK: NO R/W
    #[account(address = rentable_token_pda.token_owner)]
    pub lamports_to: AccountInfo<'info>
}


#[derive(Accounts)]
pub struct ActiveRental<'info> {
    pub rentable_token_pda: Account<'info, RentableToken>,
}

#[error_code]
pub enum Errors {
    #[msg("No signer was found for the transaction")]
    NoSigner,
    #[msg("This key does not correspond to the owner's key")]
    NotOwner,
    #[msg("The lending period is still active")]
    NotExpired,
    #[msg("Account does not have sufficient lamports to transfer")]
    InsufficientBalance,
    #[msg("The incorrect address for this PDA was provided")]
    WrongAddress
}
