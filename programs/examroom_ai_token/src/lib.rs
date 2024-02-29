use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::ops::DerefMut;

use {
    anchor_lang::{
        solana_program::system_program, AnchorDeserialize, AnchorSerialize,
        Key,
    },
};
#[constant]
const PREFIX: String = "examroom_ai";
#[constant]
const MAX_LEN: usize = 500;



declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


#[program]
pub mod examroom_ai_token {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>,
        _mint_token_vault_bump: u8,                           // for whatever reason the bump needed to be first, otherwise it complains about seed
        num_tokens: u64)->Result<()>{
    let airdrop= &mut ctx.accounts.airdrop;
    airdrop.mint_token_vault = &ctx.accounts.token_program;
    airdrop.authority = &ctx.accounts.from;
    airdrop.whitelist = &ctx.accounts.to_ata;
    airdrop.mint = &ctx.accounts.from_ata;
    airdrop.mint_token_vault_bump = _mint_token_vault_bump;
    airdrop.counter = 0;
    airdrop.whitelist_on = true;

    //Now Initialising whitelist data account
    let mut whitelist = ctx.accounts.whitelist.load_init()?;
    let data = whitelist.deref_mut();
    data.addresses = [Pubkey::default(); MAX_LEN];

    // msg!("token account owner: {}", ctx.accounts.mint_token_vault.owner);

    // setting PDA
    let (mint_token_vault_authority, _mint_token_vault_authority_bump) =
        Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);

    let cpi_accounts = SplTransfer {

            from: source.to_account_info().clone(),

            to: destination.to_account_info().clone(),

            authority: authority.to_account_info().clone(),

        };
    let cpi_program = token_program.to_account_info();


    token::transfer(

        CpiContext::new(cpi_program, cpi_accounts),

        amount)?;

    Ok(())


    }

    // pub fn add_mint_tokens(ctx: Context<AddMintTokens>, num_tokens: u64) -> Result<()> {

    //     // Transfer mint token from user to vault
    //     token::transfer(
    //         ctx.accounts.into_transfer_to_pda_context(),
    //         num_tokens
    //     )?;

    //     Ok(())
    }

    pub fn add_whitelist_addresses(
        ctx: Context<AddWhitelistAddresses>,
        addresses: Vec<Pubkey>,
    ) -> Result<()> {
        let airdrop = &mut ctx.accounts.airdrop;
        let mut whitelist = ctx.accounts.whitelist.load_mut()?;

        if !airdrop.whitelist.eq(&ctx.accounts.whitelist.key()) {
            msg!("wrong whitelist: {}", &ctx.accounts.whitelist.key());
            return Err(ErrorCode::WrongWhitelist.into());
        }

        let length = addresses.len();
        let counter = airdrop.counter as usize;

        
        if length + counter > MAX_LEN {
            return Err(ErrorCode::NotEnoughSpace.into());
        }

        
        for i in 0..length {
            whitelist.addresses[counter + i] = addresses[i];
        }
        airdrop.counter = counter as u16 + addresses.len() as u16;
        

        Ok(())
    }

    pub fn reset_whitelist_counter(ctx: Context<ResetWhitelistCounter>) -> Result<()> {
        let airdrop = &mut ctx.accounts.airdrop;
        airdrop.counter = 0;
        Ok(())
    }

    pub fn update_airdrop(ctx: Context<UpdateAirdrop>,
        on: Option<bool>) -> Result<()>{
        let airdrop= &mut ctx.accounts.airdrop;

        if let Some(whitelist_on) = on {
        
        airdrop.whitelist_on = whitelist_on;
        }
        Ok(())
    }


    pub fn send_mint_token(ctx: Context<SendMintToken>, whitelist_address_index: u16) -> Result<()> {

        let airdrop = &mut ctx.accounts.airdrop;

        // check we've got enough mint tokens
        if ctx.accounts.mint_token_vault.amount == 0 {
            return Err(ErrorCode::NotEnoughMintTokens.into());
        }

        // check if we need to check the whitelist
        if airdrop.whitelist_on {

            // make sure proper whitelist was passed in
            if !airdrop.whitelist.eq(&ctx.accounts.whitelist.key()) {
                // msg!("wrong whitelist: {}", &ctx.accounts.whitelist.key());
                return Err(ErrorCode::WrongWhitelist.into());
            }

            let i = whitelist_address_index as usize;

            // make sure the index is in range
            if i >= MAX_LEN - 1 || i > airdrop.counter as usize {
                return Err(ErrorCode::WhitelistAddressIndexOutOfRange.into());
            }

            // check if the key at the given index matches
            let payer_key = ctx.accounts.payer.key;
            let mut whitelist = ctx.accounts.whitelist.load_mut()?;

            // if this address is found on the whitelist at the given index, remove it
            if payer_key.eq(&whitelist.addresses[i].key()) {
                msg!("whitelist address key matches!");
                whitelist.addresses[i] = Pubkey::default();
            } else {
                return Err(ErrorCode::WhitelistAddressNotFound.into());
            }

        }

        // now on to the actual purchase
        if *ctx.accounts.mint_token_vault.to_account_info().key != airdrop.mint_token_vault  {
            return Err(ErrorCode::WrongTokenVault.into());
        }

        // transfer a mint token from the vault to the payer
        let (_mint_token_vault_authority, _mint_token_vault_authority_bump) =
            Pubkey::find_program_address(&[PREFIX.as_bytes()], ctx.program_id);
        let authority_seeds = &[PREFIX.as_bytes(), &[_mint_token_vault_authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_payer_context()
                .with_signer(&[&authority_seeds[..]]),
            1,
        )?;

        Ok(())
    }











#[derive(Accounts)]
#[instruction(mint_token_vault_bump: u8)]
pub struct Initialize<'info> {
    pub from: Signer<'info>,
    #[account(mut)]
    pub from_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(init,payer = authority,space=10240)]
    airdrop: Account<'info, Airdrop>,
    #[account(mut, signer)]
    authority: AccountInfo<'info>,
    #[account(zero)]
    whitelist: Account<'info, Whitelist>,
    mint: Account<'info, Mint>,                                      // mint for the token used to hit the candy machine
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    #[account(executable, "token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    #[account(mut, "authority_mint_account.owner == *authority.key")]
    authority_mint_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [PREFIX.as_bytes(), mint.key().as_ref()],
        bump = mint_token_vault_bump,
        init,
        payer = authority,
        token::mint = mint,
        token::authority = authority
        )]
    mint_token_vault: Account<'info, TokenAccount>,
}



#[derive(Accounts)]
#[instruction(mint_token_vault_bump: u8)]
pub struct AddMintTokens<'info> {
    #[account(mut, signer)]
    authority: AccountInfo<'info>,
    mint: Account<'info, Mint>,                                      // mint for the token used to hit the candy machine
    #[account(executable, "token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    #[account(mut, "authority_mint_account.owner == *authority.key")]
    authority_mint_account: Account<'info, TokenAccount>,
    #[account(mut)]
    mint_token_vault: Account<'info, TokenAccount>,
}

impl<'info> AddMintTokens<'info> {

    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.authority_mint_account.to_account_info().clone(),
            to: self.mint_token_vault.to_account_info().clone(),
            authority: self.authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct AddWhitelistAddresses<'info> {
    #[account(mut, has_one = authority)]
    airdrop: Account<'info, Airdrop>,
    #[account(mut)]
    whitelist: AccountLoader<'info, Whitelist>,
    authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResetWhitelistCounter<'info> {
    #[account(mut, has_one = authority)]
    airdrop: Account<'info, Airdrop>,
    authority: Signer<'info>,
}
#[account(zero_copy)]
pub struct Whitelist {
    addresses: [Pubkey; 500],        // note: this has to be set to a literal like this. can't be set to MAX_LEN constant
}

#[derive(Accounts)]
pub struct UpdateAirdrop<'info> {
    #[account(mut, has_one = authority)]
    airdrop: Account<'info, Airdrop>,
    authority: Signer<'info>,
}


#[derive(Accounts)]
pub struct SendMintToken<'info> {

    #[account(mut)]
    airdrop: Account<'info, Airdrop>,
    #[account(mut, signer)]
    payer: AccountInfo<'info>,
    #[account(mut)]
    whitelist: AccountLoader<'info, Whitelist>,
    #[account(address = system_program::ID)]
    system_program: AccountInfo<'info>,
    #[account(mut)]
    mint_token_vault: Account<'info, TokenAccount>,
    mint_token_vault_authority: AccountInfo<'info>,

    #[account(mut, "payer_mint_account.owner == *payer.key")]
    payer_mint_account: Account<'info, TokenAccount>,

    #[account(executable, "token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
}

impl<'info> PurchaseMintToken<'info> {

    fn into_transfer_to_payer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.mint_token_vault.to_account_info().clone(),
            to: self.payer_mint_account.to_account_info().clone(),
            authority: self.mint_token_vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

}

#[account]
#[derive(Default)]
pub struct Airdrop {
    whitelist_on: bool,
    authority: Pubkey,
    whitelist: Pubkey,                 
    mint: Pubkey,
    mint_token_vault: Pubkey,
    mint_token_vault_bump: u8,
    counter: u16,                       
}


#[error_code]
pub enum ErrorCode {
    #[msg("Wrong token vault")]
    WrongTokenVault,
    #[msg("No mint tokens left")]
    NotEnoughMintTokens,
    #[msg("Not enough space left in whitelist!")]
    NotEnoughSpace,
    #[msg("Wrong whitelist")]
    WrongWhitelist,
    #[msg("Whitelist address index out of range")]
    WhitelistAddressIndexOutOfRange,
    #[msg("Whitelisted address not found at given index")]
    WhitelistAddressNotFound
}