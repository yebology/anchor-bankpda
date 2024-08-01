use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction::transfer as system_transfer;
use anchor_lang::solana_program::program::invoke as invoke;

declare_id!("2CQ9AnuPiW2oTmctsgEAUREjADdz2ymAKCPPHJQAWUaW");

#[program]
pub mod bank_pda {
    use super::*;

    pub fn create_account(ctx: Context<Create>, name: String) -> Result<()> {
        let bank = &mut ctx.accounts.bank;
        bank.name = name;
        bank.balance = 0;
        bank.owner = *ctx.accounts.user.key;
        Ok(())
    }

    pub fn deposit_token(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let transaction = system_transfer(
            &ctx.accounts.user.key(), 
            &ctx.accounts.bank.key(), 
            amount
        );
        invoke(
            &transaction, 
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.bank.to_account_info()
            ]
        )?;
        (&mut ctx.accounts.bank).balance += amount;
        Ok(())
    }

    pub fn withdraw_balance(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let bank = &mut ctx.accounts.bank;
        let user = &mut ctx.accounts.user;
        let rent = Rent::get()?.minimum_balance(bank.to_account_info().data_len());
        if bank.owner != user.key() {
            return Err(ProgramError::IllegalOwner.into());
        }
        if **bank.to_account_info().lamports.borrow() - rent < amount {
            return Err(ProgramError::InsufficientFunds.into());
        }
        **bank.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(
        init, 
        payer=user, 
        space=8+1+8+32, 
        seeds=[b"bankaccount", user.key().as_ref()], 
        bump
    )]
    pub bank: Account<'info, Bank>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub bank: Account<'info, Bank>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub bank: Account<'info, Bank>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[account]
pub struct Bank {
    pub name: String,
    pub balance: u64,
    pub owner: Pubkey
}
