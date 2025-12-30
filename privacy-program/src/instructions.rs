use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = PrivacyPoolAccount::LEN,
        seeds = [b"privacy_pool", pool_id.to_le_bytes().as_ref()],
        bump
    )]
    pub privacy_pool: Account<'info, PrivacyPoolAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub privacy_pool: Account<'info, PrivacyPoolAccount>,

    #[account(
        init,
        payer = depositor,
        space = CommitmentAccount::LEN
    )]
    pub commitment_account: Account<'info, CommitmentAccount>,

    #[account(mut)]
    pub depositor: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub privacy_pool: Account<'info, PrivacyPoolAccount>,

    #[account(
        init,
        payer = withdrawer,
        space = NullifierAccount::LEN
    )]
    pub nullifier_account: Account<'info, NullifierAccount>,

    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PrivateTransfer<'info> {
    #[account(
        init,
        payer = sender,
        space = 8 + 256 + 256 + 256 + 1 + 32 + 8
    )]
    pub transfer_account: Account<'info, PrivateTransferAccount>,

    #[account(mut)]
    pub sender: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CrossChainTransfer<'info> {
    #[account(
        init,
        payer = sender,
        space = 8 + 2 + 2 + 512 + 32 + 12 + 16 + 32 + 8 + 1
    )]
    pub bridge_account: Account<'info, CrossChainBridgeAccount>,

    #[account(mut)]
    pub sender: Signer<'info>,

    pub system_program: Program<'info, System>,
}
