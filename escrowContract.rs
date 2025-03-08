use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_instruction;

declare_id!("Enfhk7m22FBtcR1HANuq3rxPD3qbeZRgASX4cxr7GvKZ");

#[program]
pub mod escrow_transfer {
    use super::*;

    /// **Deposit funds into the escrow PDA**
    pub fn deposit(ctx: Context<Deposit>, pda_receiver: Pubkey, amount: u64) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow_account;
        let current_time = Clock::get()?.unix_timestamp;

        if escrow_account.amount == 0 {
            escrow_account.deposit = ctx.accounts.depositor.key();
            escrow_account.pda_receiver = pda_receiver;
            escrow_account.timestamp = current_time + 60;
            escrow_account.is_completed = false;
        }

        escrow_account.amount += amount;

        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.depositor.key(),
            &ctx.accounts.escrow_account.key(),
            amount,
        );

        invoke(
            &transfer_instruction,
            &[
                ctx.accounts.depositor.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        let deposit_pubkey = ctx.accounts.escrow_account.deposit;
        msg!("deded {}", deposit_pubkey);

        msg!(
            "Deposit successful! PDA: {}",
            ctx.accounts.escrow_account.key()
        );

        Ok(())
    }

    /// **Withdraw funds from the escrow PDA**
    // pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    //     let amount = ctx.accounts.escrow_account.amount;
    //     require!(amount > 0, ErrorCode::NoFundsAvailable);

    //     // Ensure only the depositor can withdraw
    //     require_keys_eq!(
    //         ctx.accounts.deposit.key(),
    //         ctx.accounts.escrow_account.deposit,
    //         ErrorCode::UnauthorizedReceiver
    //     );

    //     // Transfer funds using the depositor as the authorized signer
    //     invoke(
    //         &system_instruction::transfer(
    //             &ctx.accounts.escrow_account.to_account_info().key, // Escrow PDA
    //             &ctx.accounts.deposit.key(), // Depositor (who originally deposited)
    //             amount,
    //         ),
    //         &[
    //             ctx.accounts.escrow_account.to_account_info(),
    //             ctx.accounts.deposit.to_account_info(),
    //             ctx.accounts.system_program.to_account_info(),
    //         ],
    //     )?;

    //     // Update escrow state
    //     let escrow_account = &mut ctx.accounts.escrow_account;
    //     escrow_account.is_completed = true;
    //     escrow_account.amount = 0;

    //     msg!(
    //         "Withdraw successful! {} withdrawn back to {}",
    //         amount,
    //         ctx.accounts.depositor.key()
    //     );
    //     Ok(())
    // }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::NoFundsAvailable);

        // Transfer SOL from the signer to the receiver
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.signer.key(),
                &ctx.accounts.receiver.key(),
                amount,
            ),
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.receiver.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        let escrow_account = &mut ctx.accounts.escrow_account;
        escrow_account.deposit = ctx.accounts.signer.key();
        escrow_account.pda_receiver = ctx.accounts.receiver.key();
        escrow_account.is_completed = true;
        escrow_account.amount = 0;

        msg!(
            "Withdraw successful! {} lamports sent to {}",
            amount,
            ctx.accounts.receiver.key()
        );
        Ok(())
    }

}

/// **Deposit Struct - Initializes Escrow PDA**
#[derive(Accounts)]
// #[instruction(escrow_id : u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        init_if_needed,
        payer = depositor,
        space = 8 + EscrowAccount::INIT_SPACE,
        seeds = [b"escro" , depositor.key().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

/// **Withdraw Struct - Withdraw from Escrow PDA**
// #[derive(Accounts)]
// #[instruction(escrow_id: u64)]
// pub struct Withdraw<'info> {
//     #[account(mut)]
//     pub receiver: SystemAccount<'info>, // ✅ Ensure it's a valid account

//     #[account(
//         mut,
//         seeds = [b"escro", escrow_account.deposit.as_ref()],  // ✅ Must match deposit
//         bump,
//     )]
//     pub escrow_account: Account<'info, EscrowAccount>,

//     pub system_program: Program<'info, System>,
// }
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // The sender must sign to withdraw

    #[account(mut)]
    pub receiver: SystemAccount<'info>, // Receiver gets the SOL
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + EscrowAccount::INIT_SPACE,
        seeds = [b"nn"],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub system_program: Program<'info, System>,
}

/// **Escrow PDA Struct**
#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub deposit: Pubkey,      // The original depositor
    pub pda_receiver: Pubkey, // Receiver of the funds
    pub amount: u64,          // Amount stored in escrow
    pub timestamp: i64,       // Time lock
    pub is_completed: bool,   // Track completion status
}

/// **Custom Errors**
#[error_code]
pub enum ErrorCode {
    #[msg("No funds available.")]
    NoFundsAvailable,

    #[msg("Unauthorized receiver.")]
    UnauthorizedReceiver,
}
