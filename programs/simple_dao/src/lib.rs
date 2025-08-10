use anchor_lang::prelude::*;
use anchor_lang::system_program;
use solana_program::system_instruction;

declare_id!("GeRLoMHnP9YxYG7iSYTp8C55paYcYxE9ioUJSe887XcZ");

const MAX_DATA_LEN: usize = 100;
const MAX_APPROVERS: usize = 50;

impl Proposal {
    pub const LEN: usize = 32 + 4 + MAX_DATA_LEN + 4 + MAX_APPROVERS + 32;
}

#[program]
pub mod dao_contract {
    use super::*;

    pub fn create_dao(
        ctx: Context<CreateDao>, 
        members: Vec<Pubkey>
    ) -> Result<()> {
        require!(
            members.len() >= 2 as usize,
            ErrorCode::NotEnoughMembers
        );

        let dao = &mut ctx.accounts.daoinfo;
        dao.members = members;

        Ok(())
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        daoPubkey: Pubkey,
        title: String,
        description: String,
        options: Vec<ProposalOption>,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let daoinfo = &ctx.accounts.daoinfo;

        require!(daoinfo.members.contains(ctx.accounts.proposer.key), ErrorCode::Unauthorized);

        let timeStamp = Clock::get()?.unix_timestamp;

        proposal.dao = daoPubkey;
        proposal.title = title;
        proposal.description = description;

        proposal.start_time = timeStamp;
        proposal.end_time = timeStamp + 864000;
        proposal.executed = false;
        proposal.options = options;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateDao<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32 * 10 + 2 + 1
    )]
    pub daoinfo: Account<'info, DaoInfo>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::LEN
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
    pub proposer: Signer<'info>

    #[account]
    pub daoinfo: Account<'info, DaoInfo>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Proposal {
    pub dao: Pubkey,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub options: Vec<ProposalOption>,
    pub start_time: i64,
    pub end_time: i64,
    pub executed: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProposalOption {
    pub text: String,
    pub vote_count: u64,
}

#[account]
pub struct DaoInfo {
    pub members: Vec<Pubkey>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Not Enough DAO Members")]
    NotEnoughMembers,

    #[msg("Unauthorized")]
    Unauthorized,
}