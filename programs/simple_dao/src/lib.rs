use anchor_lang::prelude::*;
use anchor_lang::system_program;
use solana_program::system_instruction;

declare_id!("89G9TNmmZNSri7z6apU6vQNj4RJhdqKEpTMppaUz13Nr");

const MAX_DATA_LEN: usize = 100;
const MAX_APPROVERS: usize = 50;

#[program]
pub mod dao_contract {
    use super::*;

    pub fn create_dao(ctx: Context<CreateDao>, members: Vec<Pubkey>) -> Result<()> {
        require!(members.len() >= 2 as usize, ErrorCode::NotEnoughMembers);

        let dao = &mut ctx.accounts.daoinfo;
        dao.members = members;

        Ok(())
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        daoPubkey: Pubkey,
        title: String,
        description: String,
        program_id: Pubkey,
        data: Vec<u8>,
        options: Vec<ProposalOption>,
        proposer: Pubkey,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let daoinfo = &ctx.accounts.daoinfo;

        require!(
            daoinfo.members.contains(ctx.accounts.proposer.key),
            ErrorCode::Unauthorized
        );

        let timeStamp = Clock::get()?.unix_timestamp;

        proposal.dao = daoPubkey;
        proposal.title = title;
        proposal.description = description;
        proposal.program_id = program_id;
        proposal.data = data;

        proposal.start_time = timeStamp;
        proposal.end_time = timeStamp + 100;
        proposal.executed = false;
        proposal.options = options;
        proposal.proposer = proposer;

        Ok(())
    }

    pub fn vote(ctx: Context<Vote>, option_index: u8) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let daoinfo = &ctx.accounts.daoinfo;
        let voter = ctx.accounts.voter.key();

        // Make sure the person is a member of the DAO
        require!(
            daoinfo.members.contains(&voter),
            ErrorCode::Unauthorized
        );

        // Make sure that a member can only vote once
        require!(
            !proposal.voters.contains(&voter),
            ErrorCode::AlreadyVoted
        );

        // bounds check for option_index
        let idx = option_index as usize;
        require!(idx < proposal.options.len(), ErrorCode::InvalidOption);

        proposal.voters.push(voter);
        proposal.options[idx].vote_count = proposal.options[idx].vote_count.saturating_add(1);

        Ok(())
    }

    pub fn vote_count(ctx: Context<FinalizeProposal>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;

        let now = Clock::get()?.unix_timestamp;
        require!(now > proposal.end_time, ErrorCode::VotingStillActive);

        require!(!proposal.executed, ErrorCode::AlreadyFinalized);

        // Find the option with the highest votes
        let mut max_votes: u64 = 0;
        let mut winner_idx: u8 = 0;

        let names = vec!["Alice", "Bob", "Charlie", "David"];

        for (index, name) in names.iter().enumerate() {
            println!("Index: {}, Name: {}", index, name);
        }

        for (i, option) in proposal.options.iter().enumerate() {
            if option.vote_count > max_votes {
                max_votes = option.vote_count;
                winner_idx = i as u8;
            }
        }

        proposal.winner_index = Some(winner_idx);

        Ok(())
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;

        require!(proposal.winner_index.is_some(), ErrorCode::ProposalNotFinalized);
        require!(!proposal.executed, ErrorCode::AlreadyExecuted);

        // If the winner option is no, then do nothing
        if proposal.winner_index.unwrap() == 1 {
            proposal.executed = true;
            return Ok(());
        }

        // if the winner option is yes, then mint tokens
        // Construct the CPI Instruction
        let ix = solana_program::instruction::Instruction {
            program_id: proposal.program_id,
            accounts: vec![
                AccountMeta::new(ctx.remaining_accounts[0].key(), false), // mint
                AccountMeta::new(ctx.remaining_accounts[1].key(), false), // authority
                AccountMeta::new(ctx.remaining_accounts[2].key(), false), // destination
                AccountMeta::new_readonly(ctx.remaining_accounts[3].key(), false), // destinationOwner
                AccountMeta::new(ctx.remaining_accounts[4].key(), true), // payer
                AccountMeta::new_readonly(ctx.remaining_accounts[5].key(), false), // rent
                AccountMeta::new_readonly(ctx.remaining_accounts[6].key(), false), // systemProgram
                AccountMeta::new_readonly(ctx.remaining_accounts[7].key(), false), // tokenProgram
                AccountMeta::new_readonly(ctx.remaining_accounts[8].key(), false), // associatedTokenProgram
            ],
            data: proposal.data.clone(),
        };

        // Perform the CPI (no signer seeds used for simplicity)
        let account_infos = ctx.remaining_accounts;
        solana_program::program::invoke(&ix, account_infos)?;

        // Mark transaction as executed
        proposal.executed = true;

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
    pub proposer: Signer<'info>,

    #[account()]
    pub daoinfo: Account<'info, DaoInfo>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account()]
    pub daoinfo: Account<'info, DaoInfo>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account()]
    pub daoinfo: Account<'info, DaoInfo>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    pub daoinfo: Account<'info, DaoInfo>,
}

#[account]
pub struct Proposal {
    pub dao: Pubkey,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub program_id: Pubkey,
    pub data: Vec<u8>,
    pub options: Vec<ProposalOption>,
    pub voters: Vec<Pubkey>,
    pub start_time: i64,
    pub end_time: i64,
    pub executed: bool,
    pub winner_index: Option<u8>
}

impl Proposal {
    pub const LEN: usize = 32   // dao: Pubkey
        + 32                   // proposer: Pubkey
        + 4 + MAX_DATA_LEN     // title: String (4 bytes prefix + bytes)
        + 4 + MAX_DATA_LEN     // description: String
        + 4                    // Program Id
        + 100                  // Data Length of instructions
        + 4 + (MAX_APPROVERS * (4 + MAX_DATA_LEN)) // options Vec (approx)
        + 4 + (32 * MAX_APPROVERS) // voters Vec
        + 8   // start_time
        + 8   // end_time
        + 1 // executed: bool
        + 1; // winner_index Option<u8>
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

    #[msg("Member has Already Voted")]
    AlreadyVoted,

    #[msg("Invalid option index")]
    InvalidOption,

    #[msg("Voting period still active")]
    VotingStillActive,

    #[msg("Proposal is not finalized")]
    ProposalNotFinalized,

    #[msg("Proposal already finalized")]
    AlreadyFinalized,

    #[msg("Voting has already executed")]
    AlreadyExecuted
}