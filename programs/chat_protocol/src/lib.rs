use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use core::mem::size_of;
use solana_security_txt::security_txt;

declare_id!("TfoKjohya9Ak5N1WrCoKCqaEbCq2EhucHWprbxqrYmV");

#[cfg(not(feature = "no-entrypoint"))] // Ensure it's not included when compiled as a library
security_txt! {
    name: "M4A Protocol",
    project_url: "https://m4a.io",
    contacts: "email fdr3@m4a.io",
    preferred_languages: "en",
    source_code: "https://github.com/FDR-3?tab=repositories",
    policy: "If you find a bug, email me and say something please D:"
}

const INITIAL_CEO_ADDRESS: Pubkey = pubkey!("Fdqu1muWocA5ms8VmTrUxRxxmSattrmpNraQ7RpPvzZg");

// Define the constant public key for the USDC fee recipient
pub const INITIAL_TREASURER_ADDRESS: Pubkey = pubkey!("9BRgCdmwyP5wGVTvKAUDjSwucpqGncurVa35DjaWqSsC"); 

const FEE_1CENT: f64 = 0.01;
const FEE_3CENTS: f64 = 0.03;
const FEE_4CENTS: f64 = 0.04;
const FEE_DOLLAR_TREE: f64 = 1.03;

//Chat Accounts need atleast 119 extra bytes of space to pass with full load
const CHAT_ACCOUNT_EXTRA_SIZE: usize = 119;

//Poll and poll options need atleast 118 extra bytes of space to pass with full load
const POLL_AND_POLL_OPTION_EXTRA_SIZE: usize = 144;

//Comment Sections need atleast 9 extra bytes of space to pass with full load with 1 string in seeds
//Comment Sections need atleast 17 extra bytes of space to pass with full load with 2 string in seeds (now using 2 string seed)
const COMMENT_SECTION_EXTRA_SIZE: usize = 24;

//Comments and replies need atleast 428 extra bytes of space to pass with full load
const COMMENT_REPLY_OR_IDEA_EXTRA_SIZE: usize = 470;

//Idea and federal agents need atleast 12 extra bytes of space to pass with full load
const IDEA_EXTRA_SIZE: usize = 22;
const FEDERAL_AGENT_EXTRA_SIZE: usize = 22;

const MAX_COMMENT_SECTION_PREFIX_OR_NAME_LENGTH: usize = 32;
const MAX_POLL_AND_POLL_OPTION_NAME_LENGTH: usize = 144;
const MAX_CUSTOM_USER_NAME_LENGTH: usize = 144;
const MAX_POST_LENGTH: usize = 444;

enum PostType
{
    Comment = 0,
    Reply = 1,
    Lv3Reply = 2,
    Lv4Reply = 3
}

//Error Codes
#[error_code]
pub enum AuthorizationError 
{
    #[msg("Only the CEO can call this function")]
    NotCEO,
    #[msg("Only the Treasurer can call this function")]
    NotTreasurer,
    #[msg("This comment isn't yours to change")]
    NotCommentOwner,
    #[msg("This reply isn't yours to change")]
    NotReplyOwner
}  

#[error_code]
pub enum InvalidOperationError 
{
    #[msg("This post was deleted")]
    Deleted,
    #[msg("You must vote for the person who wrote the post")]
    WrongDude,
    #[msg("You must vote a non 0 amount")]
    CantVoteZeroAmount,
    #[msg("Can't set flag to the same state")]
    FlagSameState,
    #[msg("Can't delete poll that still has options, please delete remaining options first")]
    PollStillHasOptions
}

#[error_code]
pub enum InvalidLengthError 
{
    #[msg("User Name can't be longer than 144 characters")]
    UserNameTooLong,
    #[msg("Poll or poll option name can't be longer than 144 characters")]
    PollOrPollOptionNameTooLong,
    #[msg("Comment section name prefix can't be longer than 32 characters")]
    CommentSectionNamePrefixTooLong,
    #[msg("Comment section name can't be longer than 32 characters")]
    CommentSectionNameTooLong,
    #[msg("Message can't be longer than 444 characters")]
    MSGTooLong,
} 

// Helper function to handle the USDC fee transfer
fn apply_fee<'info>(
    from_account: AccountInfo<'info>,
    to_account: AccountInfo<'info>,
    signer: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    treasurer: Account<ChatProtocolTreasurer>,
    amount: f64,
    decimal_amount: u8
) -> Result<()> {
    let cpi_accounts = token::Transfer {
        from: from_account,
        to: to_account,
        authority: signer,
    };
    let cpi_program = token_program;
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    let base_int :u64 = 10;
    let conversion_number = base_int.pow(decimal_amount as u32) as f64;
    let fixed_pointed_notation_amount = (amount * conversion_number) as u64;

    //Transfer fee to Treasurer Wallet
    token::transfer(cpi_ctx, fixed_pointed_notation_amount)?;

    msg!("Successfully transferred ${:.2} as fee to: {}", amount,  treasurer.address);

    Ok(())
}

fn send_turd_of_tree<'info>(
    from_account: AccountInfo<'info>,
    to_account: AccountInfo<'info>,
    signer: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    post_owner_address: Pubkey,
    amount: f64,
    decimal_amount: u8
) -> Result<()> {
    let cpi_accounts = token::Transfer {
        from: from_account,
        to: to_account,
        authority: signer,
    };
    let cpi_program = token_program;
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    let base_int :u64 = 10;
    let conversion_number = base_int.pow(decimal_amount as u32) as f64;
    let fixed_pointed_notation_amount = (amount * conversion_number) as u64;

    //Transfer fee to Post Owner Wallet
    token::transfer(cpi_ctx, fixed_pointed_notation_amount)?;

    msg!("Successfully transferred ${:.2} as fee to: {}", amount, post_owner_address.key());

    Ok(())
}

//Functions
#[program]
pub mod chat
{
    use super::*;

    pub fn initialize_chat_protocol_admin_accounts(ctx: Context<InitializeAdminAccounts>) -> Result<()> 
    {
        //Only the initial CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), INITIAL_CEO_ADDRESS, AuthorizationError::NotCEO);

        let ceo = &mut ctx.accounts.ceo;
        ceo.address = INITIAL_CEO_ADDRESS;

        let treasurer = &mut ctx.accounts.treasurer;
        treasurer.address = INITIAL_TREASURER_ADDRESS;

        msg!("Chat Protocol Admin Accounts Initialized");
        msg!("New CEO Address: {}", ceo.address.key());
        msg!("New Treasurer Address: {}", treasurer.address.key());

        Ok(())
    }

    pub fn pass_on_chat_protocol_ceo(ctx: Context<PassOnChatProtocolCEO>, new_ceo_address: Pubkey) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        msg!("The Chat Protocol CEO has passed on the title to a new CEO");
        msg!("New CEO: {}", new_ceo_address.key());

        ceo.address = new_ceo_address.key();

        Ok(())
    }

    pub fn pass_on_chat_protocol_treasurer(ctx: Context<PassOnChatProtocolTreasurer>, new_treasurer_address: Pubkey) -> Result<()> 
    {
        let treasurer = &mut ctx.accounts.treasurer;
        //Only the Treasurer can call this function
        require_keys_eq!(ctx.accounts.signer.key(), treasurer.address.key(), AuthorizationError::NotTreasurer);

        msg!("The Chat Protocol Treasurer has passed on the title to a new Treasurer");
        msg!("New Treasurer: {}", new_treasurer_address.key());

        treasurer.address = new_treasurer_address.key();

        Ok(())
    }

    pub fn add_fee_token_entry(ctx: Context<AddFeeTokenEntry>, token_mint_address: Pubkey, decimal_amount: u8) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let fee_token_entry = &mut ctx.accounts.fee_token_entry;
        fee_token_entry.token_mint_address = token_mint_address;
        fee_token_entry.decimal_amount = decimal_amount;

        msg!("Added Fee Token Entry");
        msg!("Mint Address: {}", token_mint_address.key());
        msg!("Decimal Amount: {}", decimal_amount);
            
        Ok(())
    }

    pub fn remove_fee_token_entry(ctx: Context<RemoveFeeTokenEntry>, token_mint_address: Pubkey) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        msg!("Removed Fee Token Entry");
        msg!("Mint Address: {}", token_mint_address.key());
            
        Ok(())
    }

    pub fn initialize_quality_of_life_accounts(ctx: Context<InitializeQualityOfLifeAccounts>) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        msg!("Quality of life accounts initialized");

        Ok(())
    }

    pub fn initialize_chat_protocol(ctx: Context<InitializeChatProtocol>) -> Result<()> 
    {
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        chat_protocol.chat_protocol_initiator_address = ctx.accounts.signer.key();

        msg!("Chat Protocol Initialized");
        msg!("Initialized By User: {}", ctx.accounts.signer.key());
        Ok(())
    }

    pub fn initialize_m4a_chat(ctx: Context<InitializeM4AChat>) -> Result<()> 
    {
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        m4a_chat.chat_initiator_address = ctx.accounts.signer.key();

        msg!("M4A Chat Initialized");
        msg!("Initialized By User: {}", ctx.accounts.signer.key());
        Ok(())
    }

    pub fn initialize_pli_chat(ctx: Context<InitializePLIChat>) -> Result<()> 
    {
        let pli_chat = &mut ctx.accounts.pli_chat;
        pli_chat.chat_initiator_address = ctx.accounts.signer.key();

        msg!("PLI Chat Initialized");
        msg!("Initialized By User: {}", ctx.accounts.signer.key());
        Ok(())
    }

    pub fn initialize_about_chat(ctx: Context<InitializeAboutChat>) -> Result<()> 
    {
        let about_chat = &mut ctx.accounts.about_chat;
        about_chat.chat_initiator_address = ctx.accounts.signer.key();

        msg!("About Chat Initialized");
        msg!("Initialized By User: {}", ctx.accounts.signer.key());
        Ok(())
    }

    pub fn initialize_lo_chat(ctx: Context<InitializeLOChat>) -> Result<()> 
    {
        let lo_chat = &mut ctx.accounts.lo_chat;
        lo_chat.chat_initiator_address = ctx.accounts.signer.key();

        msg!("LO Chat Initialized");
        msg!("Initialized By User: {}", ctx.accounts.signer.key());
        Ok(())
    }

    pub fn create_chat_account(ctx: Context<CreateChatAccount>) -> Result<()> 
    {
        let chat_account_stats = &mut ctx.accounts.chat_account_stats;
        let chat_account = &mut ctx.accounts.chat_account;

        chat_account_stats.chat_account_count += 1;
        chat_account.id = chat_account_stats.chat_account_count;
        chat_account.user_address = ctx.accounts.signer.key();

        msg!("Chat Account Created For: {}", ctx.accounts.signer.key());
        msg!("Chat Account Number: {}", chat_account_stats.chat_account_count);

        Ok(())
    }

    pub fn create_comment_section(ctx: Context<CreateCommentSection>,
        comment_section_name_prefix: String,
        comment_section_name: String) -> Result<()> 
    {
        //Comment section prefix name string must not be longer than 32 characters
        require!(comment_section_name_prefix.len() <= MAX_COMMENT_SECTION_PREFIX_OR_NAME_LENGTH, InvalidLengthError::CommentSectionNamePrefixTooLong);

        //Comment section name string must not be longer than 32 characters
        require!(comment_section_name.len() <= MAX_COMMENT_SECTION_PREFIX_OR_NAME_LENGTH, InvalidLengthError::CommentSectionNameTooLong);

        let comment_section_stats = &mut ctx.accounts.comment_section_stats;
        let comment_section = &mut ctx.accounts.comment_section;

        comment_section_stats.comment_section_count += 1;
        comment_section.id = comment_section_stats.comment_section_count;
        comment_section.comment_section_initiator_address = ctx.accounts.signer.key();
        comment_section.comment_section_name_prefix = comment_section_name_prefix.clone();
        comment_section.comment_section_name = comment_section_name.clone();

        msg!("New Comment Section Created");
        msg!("Comment Section Count: {}", comment_section_stats.comment_section_count);
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Created By User: {}", ctx.accounts.signer.key());

        let chat_account = &mut ctx.accounts.chat_account;
        if !chat_account.has_good_ending
        {
            chat_account.has_good_ending = true;
        }
       
        Ok(())
    }

    pub fn set_comment_section_flag(ctx: Context<SetCommentSectionFlag>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        is_enabled: bool) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let comment_section_stats = &mut ctx.accounts.comment_section_stats;
        let comment_section = &mut ctx.accounts.comment_section;

        comment_section_stats.toggle_flag_count += 1;

        //Can't set flag to the same state
        require!(comment_section.is_disabled != is_enabled, InvalidOperationError::FlagSameState);

        comment_section.is_disabled = is_enabled;

        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Disabled Flag Updated To: {}", is_enabled);
        
        Ok(())
    }

    //This vote could be for a video, or what ever is on the page of the comment section
    pub fn comment_section_vote(ctx: Context<CommentSectionVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section_stats = &mut ctx.accounts.comment_section_stats;
        let comment_section = &mut ctx.accounts.comment_section;
        let video_vote_stats = &mut ctx.accounts.video_vote_stats;
        let video_vote_record = &mut ctx.accounts.video_vote_record;

        if !chat_account.has_good_ending
        {
            chat_account.has_good_ending = true;
        }
    
        if is_up_vote
        {
            video_vote_stats.video_up_vote_count += 1;
            comment_section_stats.video_up_vote_count += 1;
            comment_section.video_up_vote_score += vote_amount as u128;
            comment_section.video_up_vote_count += 1;
        
            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted Comment Section Video");
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            video_vote_stats.video_down_vote_count += 1;
            comment_section_stats.video_down_vote_count += 1;
            comment_section.video_down_vote_score += vote_amount.abs() as u128;
            comment_section.video_down_vote_count += 1;
            
            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted Comment Section Video");
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        let time_stamp = Clock::get()?.unix_timestamp as u64;
        
        video_vote_record.protocol_record_id = comment_section_stats.video_up_vote_count + comment_section_stats.video_down_vote_count;
        video_vote_record.comment_section_record_id = comment_section.video_up_vote_count + comment_section.video_down_vote_count;
        video_vote_record.voter_address = ctx.accounts.signer.key();
        video_vote_record.comment_section_name_prefix = comment_section_name_prefix;
        video_vote_record.comment_section_name = comment_section_name;
        
        video_vote_record.vote_amount = vote_amount;
        video_vote_record.unix_creation_time_stamp = time_stamp;

        chat_account.video_vote_count += 1;

        let accounts = &ctx.accounts;
        let treasurer = ctx.accounts.treasurer.clone();

        //Call the helper function to transfer the fee
        apply_fee(
            accounts.user_fee_ata.to_account_info(),
            accounts.treasurer_fee_ata.to_account_info(),
            accounts.signer.to_account_info(),
            accounts.token_program.to_account_info(),
            treasurer,
            FEE_4CENTS * vote_amount.abs() as f64,
            accounts.fee_token_entry.decimal_amount
        )?;

        Ok(())
    }

    pub fn update_user_name(ctx: Context<UpdateUserName>, _token_mint_address: Pubkey, user_name: String) -> Result<()> 
    {
        //User Name string must not be longer than 144 characters
        require!(user_name.len() <= MAX_CUSTOM_USER_NAME_LENGTH, InvalidLengthError::UserNameTooLong);

        let chat_account_stats = &mut ctx.accounts.chat_account_stats;
        chat_account_stats.updated_name_count += 1;

        let chat_account = &mut ctx.accounts.chat_account;
        chat_account.user_name = user_name.clone();
        chat_account.use_custom_name = true;

        if chat_account.has_had_custom_name == false
        {
            chat_account.has_had_custom_name = true
        }

        msg!("User Name Updated For: {}", ctx.accounts.signer.key());
        msg!("User Name: {}", user_name);

        let accounts = &ctx.accounts;
        let treasurer = ctx.accounts.treasurer.clone();

        //Call the helper function to transfer the fee
        apply_fee(
            accounts.user_fee_ata.to_account_info(),
            accounts.treasurer_fee_ata.to_account_info(),
            accounts.signer.to_account_info(),
            accounts.token_program.to_account_info(),
            treasurer,
            FEE_DOLLAR_TREE,
            accounts.fee_token_entry.decimal_amount
        )?;

        Ok(())
    }

    pub fn set_use_custom_name_flag(ctx: Context<SetUseCustomNameFlag>, _token_mint_address: Pubkey, is_enabled: bool, ) -> Result<()> 
    {
        let chat_account_stats = &mut ctx.accounts.chat_account_stats;
        chat_account_stats.set_flag_count += 1;

        let chat_account = &mut ctx.accounts.chat_account;
        //Can't set flag to the same state
        require!(chat_account.use_custom_name != is_enabled, InvalidOperationError::FlagSameState);

        chat_account.use_custom_name = is_enabled;

        msg!("User Name Flag Updated For: {}", ctx.accounts.signer.key());
        msg!("User Name Flag: {}", is_enabled);

        let accounts = &ctx.accounts;
        let treasurer = ctx.accounts.treasurer.clone();

        //Call the helper function to transfer the fee
        apply_fee(
            accounts.user_fee_ata.to_account_info(),
            accounts.treasurer_fee_ata.to_account_info(),
            accounts.signer.to_account_info(),
            accounts.token_program.to_account_info(),
            treasurer,
            FEE_4CENTS,
            accounts.fee_token_entry.decimal_amount
        )?;

        Ok(())
    }

    pub fn post_m4a_comment(ctx: Context<PostM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_comment = &mut ctx.accounts.m4a_comment;

        m4a_comment.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        m4a_chat.comment_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.comment_count += 1;
        comment_section.comment_and_reply_count += 1;

        m4a_comment.id = comment_section.comment_and_reply_count;
        m4a_comment.protocol_post_count = chat_protocol.comment_and_reply_count;
        m4a_comment.comment_section_name_prefix = comment_section_name_prefix.clone();
        m4a_comment.comment_section_name = comment_section_name.clone();
        m4a_comment.post_owner_address = ctx.accounts.signer.key();
        m4a_comment.msg = msg.clone();
        m4a_comment.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        
        msg!("New M4A Comment Posted");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Comment: {}", m4a_comment.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_DOLLAR_TREE,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_m4a_comment(ctx: Context<ReplyToM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_comment = &mut ctx.accounts.m4a_comment;
        let m4a_reply= &mut ctx.accounts.m4a_reply;

        m4a_comment.reply_count += 1;
        m4a_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        m4a_chat.reply_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_count += 1;
        comment_section.comment_and_reply_count += 1;

        m4a_reply.id = comment_section.comment_and_reply_count;
        m4a_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        m4a_reply.parent_id = m4a_comment.id;
        m4a_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        m4a_reply.comment_section_name = comment_section_name.clone();
        m4a_reply.post_owner_address = ctx.accounts.signer.key();
        m4a_reply.msg = msg.clone();
        m4a_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        
        msg!("New M4A Chat Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", m4a_reply.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_m4a_reply(ctx: Context<ReplyToM4AReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_reply = &mut ctx.accounts.m4a_reply;
        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;

        m4a_reply.reply_count += 1;
        m4a_lv3_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        m4a_chat.reply_lv3_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv3_count += 1;
        comment_section.comment_and_reply_count += 1;

        m4a_lv3_reply.id = comment_section.comment_and_reply_count;
        m4a_lv3_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        m4a_lv3_reply.parent_id = m4a_reply.id;
        m4a_lv3_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        m4a_lv3_reply.comment_section_name = comment_section_name.clone();
        m4a_lv3_reply.post_owner_address = ctx.accounts.signer.key();
        m4a_lv3_reply.msg = msg.clone();
        m4a_lv3_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        
        msg!("New M4A Chat Lv3 Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_m4a_lv3_reply(ctx: Context<ReplyToM4ALv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;
        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;

        m4a_lv3_reply.reply_count += 1;
        m4a_lv4_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        m4a_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        m4a_lv4_reply.id = comment_section.comment_and_reply_count;
        m4a_lv4_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        m4a_lv4_reply.parent_id = m4a_lv3_reply.id;
        m4a_lv4_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        m4a_lv4_reply.comment_section_name = comment_section_name.clone();
        m4a_lv4_reply.post_owner_address = ctx.accounts.signer.key();
        m4a_lv4_reply.msg = msg.clone();
        m4a_lv4_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New M4A Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_m4a_lv4_reply(ctx: Context<ReplyToM4ALv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;
        let m4a_lv4_plus_reply = &mut ctx.accounts.m4a_lv4_plus_reply;

        m4a_lv4_reply.reply_count += 1;
        m4a_lv4_plus_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        m4a_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        m4a_lv4_plus_reply.id = comment_section.comment_and_reply_count;
        m4a_lv4_plus_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        m4a_lv4_plus_reply.parent_id = m4a_lv4_reply.id;
        m4a_lv4_plus_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        m4a_lv4_plus_reply.comment_section_name = comment_section_name.clone();
        m4a_lv4_plus_reply.post_owner_address = ctx.accounts.signer.key();
        m4a_lv4_plus_reply.msg = msg.clone();
        m4a_lv4_plus_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New M4A Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn post_pli_comment(ctx: Context<PostPLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_comment = &mut ctx.accounts.pli_comment;
        
        pli_comment.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        pli_chat.comment_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.comment_count += 1;
        comment_section.comment_and_reply_count += 1;

        pli_comment.id = comment_section.comment_and_reply_count;
        pli_comment.protocol_post_count = chat_protocol.comment_and_reply_count;
        pli_comment.comment_section_name_prefix = comment_section_name_prefix.clone();
        pli_comment.comment_section_name = comment_section_name.clone();
        pli_comment.post_owner_address = ctx.accounts.signer.key();
        pli_comment.msg = msg.clone();
        pli_comment.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New PLI Comment Posted");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Comment: {}", pli_comment.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_DOLLAR_TREE,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_pli_comment(ctx: Context<ReplyToPLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_comment = &mut ctx.accounts.pli_comment;
        let pli_reply= &mut ctx.accounts.pli_reply;

        pli_comment.reply_count += 1;
        pli_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        pli_chat.reply_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_count += 1;
        comment_section.comment_and_reply_count += 1;

        pli_reply.id = comment_section.comment_and_reply_count;
        pli_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        pli_reply.parent_id = pli_comment.id;
        pli_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        pli_reply.comment_section_name = comment_section_name.clone();
        pli_reply.post_owner_address = ctx.accounts.signer.key();
        pli_reply.msg = msg.clone();
        pli_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New PLI Chat Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", pli_reply.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_pli_reply(ctx: Context<ReplyToPLIReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_reply = &mut ctx.accounts.pli_reply;
        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;

        pli_reply.reply_count += 1;
        pli_lv3_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        pli_chat.reply_lv3_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv3_count += 1;
        comment_section.comment_and_reply_count += 1;

        pli_lv3_reply.id = comment_section.comment_and_reply_count;
        pli_lv3_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        pli_lv3_reply.parent_id = pli_reply.id;
        pli_lv3_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        pli_lv3_reply.comment_section_name = comment_section_name.clone();
        pli_lv3_reply.post_owner_address = ctx.accounts.signer.key();
        pli_lv3_reply.msg = msg.clone();
        pli_lv3_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New PLI Chat Lv3 Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_pli_lv3_reply(ctx: Context<ReplyToPLILv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;
        let pli_lv4_reply= &mut ctx.accounts.pli_lv4_reply;

        pli_lv3_reply.reply_count += 1;
        pli_lv4_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        pli_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        pli_lv4_reply.id = comment_section.comment_and_reply_count;
        pli_lv4_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        pli_lv4_reply.parent_id = pli_lv3_reply.id;
        pli_lv4_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        pli_lv4_reply.comment_section_name = comment_section_name.clone();
        pli_lv4_reply.post_owner_address = ctx.accounts.signer.key();
        pli_lv4_reply.msg = msg.clone();
        pli_lv4_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New PLI Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_pli_lv4_reply(ctx: Context<ReplyToPLILv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;
        let pli_lv4_plus_reply = &mut ctx.accounts.pli_lv4_plus_reply;

        pli_lv4_reply.reply_count += 1;
        pli_lv4_plus_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        pli_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        pli_lv4_plus_reply.id = comment_section.comment_and_reply_count;
        pli_lv4_plus_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        pli_lv4_plus_reply.parent_id = pli_lv4_reply.id;
        pli_lv4_plus_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        pli_lv4_plus_reply.comment_section_name = comment_section_name.clone();
        pli_lv4_plus_reply.post_owner_address = ctx.accounts.signer.key();
        pli_lv4_plus_reply.msg = msg.clone();
        pli_lv4_plus_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New PLI Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn post_about_comment(ctx: Context<PostAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_comment = &mut ctx.accounts.about_comment;

        about_comment.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        about_chat.comment_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.comment_count += 1;
        comment_section.comment_and_reply_count += 1;

        about_comment.id = comment_section.comment_and_reply_count;
        about_comment.protocol_post_count = chat_protocol.comment_and_reply_count;
        about_comment.comment_section_name_prefix = comment_section_name_prefix.clone();
        about_comment.comment_section_name = comment_section_name.clone();
        about_comment.post_owner_address = ctx.accounts.signer.key();
        about_comment.msg = msg.clone();
        about_comment.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New About Comment Posted");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Comment: {}", about_comment.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_DOLLAR_TREE,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_about_comment(ctx: Context<ReplyToAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_comment = &mut ctx.accounts.about_comment;
        let about_reply= &mut ctx.accounts.about_reply;

        about_comment.reply_count += 1;
        about_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        about_chat.reply_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_count += 1;
        comment_section.comment_and_reply_count += 1;

        about_reply.id = comment_section.comment_and_reply_count;
        about_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        about_reply.parent_id = about_comment.id;
        about_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        about_reply.comment_section_name = comment_section_name.clone();
        about_reply.post_owner_address = ctx.accounts.signer.key();
        about_reply.msg = msg.clone();
        about_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New About Chat Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", about_reply.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_about_reply(ctx: Context<ReplyToAboutReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_reply = &mut ctx.accounts.about_reply;
        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;

        about_reply.reply_count += 1;
        about_lv3_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        about_chat.reply_lv3_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv3_count += 1;
        comment_section.comment_and_reply_count += 1;

        about_lv3_reply.id = comment_section.comment_and_reply_count;
        about_lv3_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        about_lv3_reply.parent_id = about_reply.id;
        about_lv3_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        about_lv3_reply.comment_section_name = comment_section_name.clone();
        about_lv3_reply.post_owner_address = ctx.accounts.signer.key();
        about_lv3_reply.msg = msg.clone();
        about_lv3_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New About Chat Lv3 Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_about_lv3_reply(ctx: Context<ReplyToAboutLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;
        let about_lv4_reply= &mut ctx.accounts.about_lv4_reply;

        about_lv3_reply.reply_count += 1;
        about_lv4_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        about_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        about_lv4_reply.id = comment_section.comment_and_reply_count;
        about_lv4_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        about_lv4_reply.parent_id = about_lv3_reply.id;
        about_lv4_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        about_lv4_reply.comment_section_name = comment_section_name.clone();
        about_lv4_reply.post_owner_address = ctx.accounts.signer.key();
        about_lv4_reply.msg = msg.clone();
        about_lv4_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New About Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_about_lv4_reply(ctx: Context<ReplyToAboutLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;
        let about_lv4_plus_reply = &mut ctx.accounts.about_lv4_plus_reply;

        about_lv4_reply.reply_count += 1;
        about_lv4_plus_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        about_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        about_lv4_plus_reply.id = comment_section.comment_and_reply_count;
        about_lv4_plus_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        about_lv4_plus_reply.parent_id = about_lv4_reply.id;
        about_lv4_plus_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        about_lv4_plus_reply.comment_section_name = comment_section_name.clone();
        about_lv4_plus_reply.post_owner_address = ctx.accounts.signer.key();
        about_lv4_plus_reply.msg = msg.clone();
        about_lv4_plus_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New About Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn post_lo_comment(ctx: Context<PostLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_comment = &mut ctx.accounts.lo_comment;

        lo_comment.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        lo_chat.comment_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.comment_count += 1;
        comment_section.comment_and_reply_count += 1;

        lo_comment.id = comment_section.comment_and_reply_count;
        lo_comment.protocol_post_count = chat_protocol.comment_and_reply_count;
        lo_comment.comment_section_name_prefix = comment_section_name_prefix.clone();
        lo_comment.comment_section_name = comment_section_name.clone();
        lo_comment.post_owner_address = ctx.accounts.signer.key();
        lo_comment.msg = msg.clone();
        lo_comment.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New LO Comment Posted");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Comment: {}", lo_comment.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_DOLLAR_TREE,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_lo_comment(ctx: Context<ReplyToLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_comment = &mut ctx.accounts.lo_comment;
        let lo_reply= &mut ctx.accounts.lo_reply;

        lo_comment.reply_count += 1;
        lo_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        lo_chat.reply_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_count += 1;
        comment_section.comment_and_reply_count += 1;

        lo_reply.id = comment_section.comment_and_reply_count;
        lo_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        lo_reply.parent_id = lo_comment.id;
        lo_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        lo_reply.comment_section_name = comment_section_name.clone();
        lo_reply.post_owner_address = ctx.accounts.signer.key();
        lo_reply.msg = msg.clone();
        lo_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New LO Chat Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", lo_reply.msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_lo_reply(ctx: Context<ReplyToLOReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_reply = &mut ctx.accounts.lo_reply;
        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;

        lo_reply.reply_count += 1;
        lo_lv3_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        lo_chat.reply_lv3_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv3_count += 1;
        comment_section.comment_and_reply_count += 1;

        lo_lv3_reply.id = comment_section.comment_and_reply_count;
        lo_lv3_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        lo_lv3_reply.parent_id = lo_reply.id;
        lo_lv3_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        lo_lv3_reply.comment_section_name = comment_section_name.clone();
        lo_lv3_reply.post_owner_address = ctx.accounts.signer.key();
        lo_lv3_reply.msg = msg.clone();
        lo_lv3_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New LO Chat Lv3 Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_lo_lv3_reply(ctx: Context<ReplyToLOLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;
        let lo_lv4_reply= &mut ctx.accounts.lo_lv4_reply;

        lo_lv3_reply.reply_count += 1;
        lo_lv4_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        lo_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        lo_lv4_reply.id = comment_section.comment_and_reply_count;
        lo_lv4_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        lo_lv4_reply.parent_id = lo_lv3_reply.id;
        lo_lv4_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        lo_lv4_reply.comment_section_name = comment_section_name.clone();
        lo_lv4_reply.post_owner_address = ctx.accounts.signer.key();
        lo_lv4_reply.msg = msg.clone();
        lo_lv4_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New LO Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn reply_to_lo_lv4_reply(ctx: Context<ReplyToLOLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
    
        let chat_protocol = &mut ctx.accounts.chat_protocol;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;
        let lo_lv4_plus_reply = &mut ctx.accounts.lo_lv4_plus_reply;

        lo_lv4_reply.reply_count += 1;
        lo_lv4_plus_reply.chat_account_post_count_index = chat_account.comment_and_reply_count;
        chat_protocol.comment_and_reply_count += 1;
        lo_chat.reply_lv4_count += 1;
        chat_account.comment_and_reply_count += 1;
        comment_section.reply_lv4_count += 1;
        comment_section.comment_and_reply_count += 1;

        lo_lv4_plus_reply.id = comment_section.comment_and_reply_count;
        lo_lv4_plus_reply.protocol_post_count = chat_protocol.comment_and_reply_count;
        lo_lv4_plus_reply.parent_id = lo_lv4_reply.id;
        lo_lv4_plus_reply.comment_section_name_prefix = comment_section_name_prefix.clone();
        lo_lv4_plus_reply.comment_section_name = comment_section_name.clone();
        lo_lv4_plus_reply.post_owner_address = ctx.accounts.signer.key();
        lo_lv4_plus_reply.msg = msg.clone();
        lo_lv4_plus_reply.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("New LO Chat Lv4+ Reply");
        msg!("Chat Protocol Comment And Reply Count: {}", chat_protocol.comment_and_reply_count);
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Message: {}", msg);

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn edit_m4a_comment(ctx: Context<EditM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_comment = &mut ctx.accounts.m4a_comment;

        //You can't edit a comment that has been deleted
        require!(m4a_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a comment that isn't yours
        require_keys_eq!(m4a_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        m4a_chat.edited_comment_count += 1;
        comment_section.edited_comment_count += 1;
 
        msg!("M4A Comment Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        m4a_comment.msg = msg;

        if m4a_comment.is_edited == false
        {
            m4a_comment.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_m4a_reply(ctx: Context<EditM4AReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);
  
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_reply = &mut ctx.accounts.m4a_reply;

        //You can't edit a reply that has been deleted
        require!(m4a_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(m4a_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        m4a_chat.edited_reply_count += 1;
        comment_section.edited_reply_count += 1;

        msg!("M4A Reply Edited By User");
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        m4a_reply.msg = msg;

        if m4a_reply.is_edited == false
        {
            m4a_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_m4a_lv3_reply(ctx: Context<EditM4ALv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;

        //You can't edit a reply that has been deleted
        require!(m4a_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(m4a_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        m4a_chat.edited_lv3_reply_count += 1;
        comment_section.edited_lv3_reply_count += 1;

        msg!("M4A Lv3 Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        m4a_lv3_reply.msg = msg;

        if m4a_lv3_reply.is_edited == false
        {
            m4a_lv3_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_m4a_lv4_reply(ctx: Context<EditM4ALv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;

        //You can't edit a reply that has been deleted
        require!(m4a_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(m4a_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        m4a_chat.edited_lv4_reply_count += 1;
        comment_section.edited_lv4_reply_count += 1;

        msg!("M4A Lv4+ Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        m4a_lv4_reply.msg = msg;

        if m4a_lv4_reply.is_edited == false
        {
            m4a_lv4_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_pli_comment(ctx: Context<EditPLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_comment = &mut ctx.accounts.pli_comment;

        //You can't edit a comment that has been deleted
        require!(pli_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a comment that isn't yours
        require_keys_eq!(pli_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        pli_chat.edited_comment_count += 1;
        comment_section.edited_comment_count += 1;

        msg!("PLI Comment Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        pli_comment.msg = msg;

        if pli_comment.is_edited == false
        {
            pli_comment.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_pli_reply(ctx: Context<EditPLIReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_reply = &mut ctx.accounts.pli_reply;

        //You can't edit a reply that has been deleted
        require!(pli_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(pli_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        pli_chat.edited_reply_count += 1;
        comment_section.edited_reply_count += 1;

        msg!("PLI Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        pli_reply.msg = msg;

        if pli_reply.is_edited == false
        {
            pli_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_pli_lv3_reply(ctx: Context<EditPLILv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;

        //You can't edit a reply that has been deleted
        require!(pli_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(pli_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        pli_chat.edited_lv3_reply_count += 1;
        comment_section.edited_lv3_reply_count += 1;

        msg!("PLI Lv3 Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        pli_lv3_reply.msg = msg;

        if pli_lv3_reply.is_edited == false
        {
            pli_lv3_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_pli_lv4_reply(ctx: Context<EditPLILv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;

        //You can't edit a reply that has been deleted
        require!(pli_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(pli_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        pli_chat.edited_lv4_reply_count += 1;
        comment_section.edited_lv4_reply_count += 1;

        msg!("PLI Lv4+ Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        pli_lv4_reply.msg = msg;

        if pli_lv4_reply.is_edited == false
        {
            pli_lv4_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_about_comment(ctx: Context<EditAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_comment = &mut ctx.accounts.about_comment;

        //You can't edit a comment that has been deleted
        require!(about_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a comment that isn't yours
        require_keys_eq!(about_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        about_chat.edited_comment_count += 1;
        comment_section.edited_comment_count += 1;

        msg!("About Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        about_comment.msg = msg;

        if about_comment.is_edited == false
        {
            about_comment.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_about_reply(ctx: Context<EditAboutReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_reply = &mut ctx.accounts.about_reply;

        //You can't edit a reply that has been deleted
        require!(about_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(about_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        about_chat.edited_reply_count += 1;
        comment_section.edited_reply_count += 1;

        msg!("About Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        about_reply.msg = msg;

        if about_reply.is_edited == false
        {
            about_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_about_lv3_reply(ctx: Context<EditAboutLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;

        //You can't edit a reply that has been deleted
        require!(about_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(about_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);

        about_chat.edited_lv3_reply_count += 1;
        comment_section.edited_lv3_reply_count += 1;

        msg!("About Lv3 Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        about_lv3_reply.msg = msg;

        if about_lv3_reply.is_edited == false
        {
            about_lv3_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_about_lv4_reply(ctx: Context<EditAboutLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;

        //You can't edit a reply that has been deleted
        require!(about_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(about_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        about_chat.edited_lv4_reply_count += 1;
        comment_section.edited_lv4_reply_count += 1;

        msg!("About Lv4+ Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        about_lv4_reply.msg = msg;

        if about_lv4_reply.is_edited == false
        {
            about_lv4_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_lo_comment(ctx: Context<EditLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_comment = &mut ctx.accounts.lo_comment;

        //You can't edit a comment that has been deleted
        require!(lo_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a comment that isn't yours
        require_keys_eq!(lo_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        lo_chat.edited_comment_count += 1;
        comment_section.edited_comment_count += 1;

        msg!("LO Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        lo_comment.msg = msg;

        if lo_comment.is_edited == false
        {
            lo_comment.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_lo_reply(ctx: Context<EditLOReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_reply = &mut ctx.accounts.lo_reply;

        //You can't edit a reply that has been deleted
        require!(lo_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(lo_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        lo_chat.edited_reply_count += 1;
        comment_section.edited_reply_count += 1;

        msg!("LO Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        lo_reply.msg = msg;

        if lo_reply.is_edited == false
        {
            lo_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_lo_lv3_reply(ctx: Context<EditLOLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;

        //You can't edit a reply that has been deleted
        require!(lo_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(lo_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        lo_chat.edited_lv3_reply_count += 1;
        comment_section.edited_lv3_reply_count += 1;

        msg!("LO Lv3 Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        lo_lv3_reply.msg = msg;

        if lo_lv3_reply.is_edited == false
        {
            lo_lv3_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn edit_lo_lv4_reply(ctx: Context<EditLOLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        msg: String) -> Result<()> 
    {
        //Message string must not be longer than 444 characters
        require!(msg.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;

        //You can't edit a reply that has been deleted
        require!(lo_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't edit a reply that isn't yours
        require_keys_eq!(lo_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        lo_chat.edited_lv4_reply_count += 1;
        comment_section.edited_lv4_reply_count += 1;

        msg!("LO Lv4+ Reply Edited By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Message: {}", msg);
        
        chat_account.edited_comment_and_reply_count += 1;
        lo_lv4_reply.msg = msg;

        if lo_lv4_reply.is_edited == false
        {
            lo_lv4_reply.is_edited = true; 
        }

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
     
        Ok(())
    }

    pub fn delete_m4a_comment(ctx: Context<DeleteM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_comment = &mut ctx.accounts.m4a_comment;

        //You can't delete a comment that has already been deleted
        require!(m4a_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a comment that isn't yours
        require_keys_eq!(m4a_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotCommentOwner);
        
        m4a_chat.deleted_comment_count += 1;
        comment_section.deleted_comment_count += 1;

        msg!("PLI Comment Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        m4a_comment.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_m4a_reply(ctx: Context<DeleteM4AReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_reply = &mut ctx.accounts.m4a_reply;

        //You can't delete a reply that has already been deleted
        require!(m4a_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(m4a_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        m4a_chat.deleted_reply_count += 1;
        comment_section.deleted_reply_count += 1;

        msg!("M4A Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        m4a_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_m4a_lv3_reply(ctx: Context<DeleteM4ALv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;

        //You can't delete a reply that has already been deleted
        require!(m4a_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(m4a_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        m4a_chat.deleted_lv3_reply_count += 1;
        comment_section.deleted_lv3_reply_count += 1;

        msg!("M4A Lv3 Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        m4a_lv3_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_m4a_lv4_reply(ctx: Context<DeleteM4ALv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;

        //You can't delete a reply that has already been deleted
        require!(m4a_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(m4a_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        m4a_chat.deleted_lv4_reply_count += 1;
        comment_section.deleted_lv4_reply_count += 1;

        msg!("M4A Lv4+ Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        m4a_lv4_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_pli_comment(ctx: Context<DeletePLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_comment = &mut ctx.accounts.pli_comment;

        //You can't delete a comment that has already been deleted
        require!(pli_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a comment that isn't yours
        require_keys_eq!(pli_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotCommentOwner);
        
        pli_chat.deleted_comment_count += 1;
        comment_section.deleted_comment_count += 1;

        msg!("PLI Comment Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        pli_comment.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_pli_reply(ctx: Context<DeletePLIReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_reply = &mut ctx.accounts.pli_reply;

        //You can't delete a reply that has already been deleted
        require!(pli_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(pli_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        pli_chat.deleted_reply_count += 1;
        comment_section.deleted_reply_count += 1;

        msg!("PLI Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        pli_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_pli_lv3_reply(ctx: Context<DeletePLILv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;

        //You can't delete a reply that has already been deleted
        require!(pli_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(pli_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        pli_chat.deleted_lv3_reply_count += 1;
        comment_section.deleted_lv3_reply_count += 1;

        msg!("PLI Lv3 Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        pli_lv3_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_pli_lv4_reply(ctx: Context<DeletePLILv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;

        //You can't delete a reply that has already been deleted
        require!(pli_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(pli_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        pli_chat.deleted_lv4_reply_count += 1;
        comment_section.deleted_lv4_reply_count += 1;

        msg!("PLI Lv4+ Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        pli_lv4_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_about_comment(ctx: Context<DeleteAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_comment = &mut ctx.accounts.about_comment;

        //You can't delete a comment that has already been deleted
        require!(about_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a comment that isn't yours
        require_keys_eq!(about_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotCommentOwner);
        
        about_chat.deleted_comment_count += 1;
        comment_section.deleted_comment_count += 1;

        msg!("About Comment Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        about_comment.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_about_reply(ctx: Context<DeleteAboutReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_reply = &mut ctx.accounts.about_reply;

        //You can't delete a reply that has already been deleted
        require!(about_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(about_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        about_chat.deleted_reply_count += 1;
        comment_section.deleted_reply_count += 1;

        msg!("About Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        about_reply.is_deleted = true;
              
        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn delete_about_lv3_reply(ctx: Context<DeleteAboutLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;

        //You can't delete a reply that has already been deleted
        require!(about_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(about_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        about_chat.deleted_lv3_reply_count += 1;
        comment_section.deleted_lv3_reply_count += 1;

        msg!("About Lv3 Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        about_lv3_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_about_lv4_reply(ctx: Context<DeleteAboutLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;

        //You can't delete a reply that has already been deleted
        require!(about_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(about_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        about_chat.deleted_lv4_reply_count += 1;
        comment_section.deleted_lv4_reply_count += 1;

        msg!("About Lv4+ Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        about_lv4_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_lo_comment(ctx: Context<DeleteLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_comment = &mut ctx.accounts.lo_comment;

        //You can't delete a comment that has already been deleted
        require!(lo_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a comment that isn't yours
        require_keys_eq!(lo_comment.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotCommentOwner);
        
        lo_chat.deleted_comment_count += 1;
        comment_section.deleted_comment_count += 1;

        msg!("LO Comment Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        lo_comment.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_lo_reply(ctx: Context<DeleteLOReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_reply = &mut ctx.accounts.lo_reply;

        //You can't delete a reply that has already been deleted
        require!(lo_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(lo_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        lo_chat.deleted_reply_count += 1;
        comment_section.deleted_reply_count += 1;

        msg!("LO Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        lo_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_lo_lv3_reply(ctx: Context<DeleteLOLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;

        //You can't delete a reply that has already been deleted
        require!(lo_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(lo_lv3_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        lo_chat.deleted_lv3_reply_count += 1;
        comment_section.deleted_lv3_reply_count += 1;

        msg!("LO Lv3 Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        lo_lv3_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn delete_lo_lv4_reply(ctx: Context<DeleteLOLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey) -> Result<()> 
    {
        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;

        //You can't delete a reply that has already been deleted
        require!(lo_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You can't delete a reply that isn't yours
        require_keys_eq!(lo_lv4_reply.post_owner_address.key(), ctx.accounts.signer.key(), AuthorizationError::NotReplyOwner);
        
        lo_chat.deleted_lv4_reply_count += 1;
        comment_section.deleted_lv4_reply_count += 1;

        msg!("LO Lv4+ Reply Deleted By User");
        msg!("User Address: {}", ctx.accounts.signer.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);

        chat_account.deleted_comment_and_reply_count += 1;
        lo_lv4_reply.is_deleted = true;

        let ceo = &mut ctx.accounts.ceo;
        if ctx.accounts.signer.key() != ceo.address.key()
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
              
        Ok(())
    }

    pub fn m4a_comment_vote(ctx: Context<M4ACommentVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_comment = &mut ctx.accounts.m4a_comment;
        let post_vote_record = &mut ctx.accounts.post_vote_record;

        //You can't vote for a comment that has been deleted
        require!(m4a_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the comment
        require_keys_eq!(m4a_comment.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;
    
        //Add code to account for voter voting for their own comment since can't duplicate accounts
        if m4a_comment.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }  
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            m4a_chat.comment_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.comment_up_vote_score += vote_amount as u128;
            comment_section.comment_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted M4A Comment From");
            msg!("User Address: {}", m4a_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            m4a_chat.comment_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.comment_down_vote_score += vote_amount.abs() as u128;
            comment_section.comment_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted M4A Comment From");
            msg!("User Address: {}", m4a_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        m4a_comment.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn m4a_reply_vote(ctx: Context<M4AReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_reply = &mut ctx.accounts.m4a_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(m4a_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(m4a_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't dum4acate accounts
        if m4a_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }  
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            m4a_chat.reply_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_up_vote_score += vote_amount as u128;
            comment_section.reply_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted M4A Reply From");
            msg!("User Address: {}", m4a_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            m4a_chat.reply_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted M4A Reply From");
            msg!("User Address: {}", m4a_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        m4a_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn m4a_lv3_reply_vote(ctx: Context<M4ALv3ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(m4a_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(m4a_lv3_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't dum4acate accounts
        if m4a_lv3_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            m4a_chat.reply_lv3_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv3_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted M4A Lv3 Reply From");
            msg!("User Address: {}", m4a_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            m4a_chat.reply_lv3_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv3_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted M4A Lv3 Reply From");
            msg!("User Address: {}", m4a_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        m4a_lv3_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn m4a_lv4_reply_vote(ctx: Context<M4ALv4ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(m4a_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(m4a_lv4_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't dum4acate accounts
        if m4a_lv4_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }  
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            m4a_chat.reply_lv4_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_lv3_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv4_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted M4A Lv4+ Reply From");
            msg!("User Address: {}", m4a_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            m4a_chat.reply_lv4_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_lv3_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv4_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted M4A Lv4+ Reply From");
            msg!("User Address: {}", m4a_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        m4a_lv4_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn pli_comment_vote(ctx: Context<PLICommentVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_comment = &mut ctx.accounts.pli_comment;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a comment that has been deleted
        require!(pli_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the comment
        require_keys_eq!(pli_comment.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own comment since can't duplicate accounts
        if pli_comment.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }  
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            pli_chat.comment_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.comment_up_vote_score += vote_amount as u128;
            comment_section.comment_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted PLI Comment From");
            msg!("User Address: {}", pli_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            pli_chat.comment_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.comment_down_vote_score += vote_amount.abs() as u128;
            comment_section.comment_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted PLI Comment From");
            msg!("User Address: {}", pli_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        pli_comment.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn pli_reply_vote(ctx: Context<PLIReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_reply = &mut ctx.accounts.pli_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(pli_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(pli_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if pli_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            pli_chat.reply_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_up_vote_score += vote_amount as u128;
            comment_section.reply_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted PLI Reply From");
            msg!("User Address: {}", pli_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            pli_chat.reply_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted PLI Reply From");
            msg!("User Address: {}", pli_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        pli_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn pli_lv3_reply_vote(ctx: Context<PLILv3ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(pli_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(pli_lv3_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if pli_lv3_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }  
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            pli_chat.reply_lv3_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv3_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted PLI Lv3 Reply From");
            msg!("User Address: {}", pli_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            pli_chat.reply_lv3_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv3_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted PLI Lv3 Reply From");
            msg!("User Address: {}", pli_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        pli_lv3_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn pli_lv4_reply_vote(ctx: Context<PLILv4ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let pli_chat = &mut ctx.accounts.pli_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(pli_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(pli_lv4_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if pli_lv4_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            pli_chat.reply_lv4_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_lv3_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv4_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted PLI Lv4+ Reply From");
            msg!("User Address: {}", pli_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            pli_chat.reply_lv4_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_lv3_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv4_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted PLI Lv4+ Reply From");
            msg!("User Address: {}", pli_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        pli_lv4_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn about_comment_vote(ctx: Context<AboutCommentVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let about_chat = &mut ctx.accounts.about_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_comment = &mut ctx.accounts.about_comment;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a comment that has been deleted
        require!(about_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the comment
        require_keys_eq!(about_comment.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own comment since can't duplicate accounts
        if about_comment.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            about_chat.comment_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.comment_up_vote_score += vote_amount as u128;
            comment_section.comment_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted About Comment From");
            msg!("User Address: {}", about_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            about_chat.comment_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.comment_down_vote_score += vote_amount.abs() as u128;
            comment_section.comment_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted About Comment From");
            msg!("User Address: {}", about_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        about_comment.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn about_reply_vote(ctx: Context<AboutReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let about_chat = &mut ctx.accounts.about_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_reply = &mut ctx.accounts.about_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(about_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(about_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if about_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            about_chat.reply_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_up_vote_score += vote_amount as u128;
            comment_section.reply_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted About Reply From");
            msg!("User Address: {}", about_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            about_chat.reply_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted About Reply From");
            msg!("User Address: {}", about_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        about_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn about_lv3_reply_vote(ctx: Context<AboutLv3ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let about_chat = &mut ctx.accounts.about_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(about_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(about_lv3_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if about_lv3_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            about_chat.reply_lv3_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv3_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted About Lv3 Reply From");
            msg!("User Address: {}", about_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            about_chat.reply_lv3_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv3_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted About Lv3 Reply From");
            msg!("User Address: {}", about_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;    
        about_lv3_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn about_lv4_reply_vote(ctx: Context<AboutLv4ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let about_chat = &mut ctx.accounts.about_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(about_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(about_lv4_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if about_lv4_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            about_chat.reply_lv4_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_lv3_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv4_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted About Lv4+ Reply From");
            msg!("User Address: {}", about_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            about_chat.reply_lv4_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_lv3_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv4_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted About Lv4+ Reply From");
            msg!("User Address: {}", about_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        about_lv4_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn lo_comment_vote(ctx: Context<LOCommentVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_comment = &mut ctx.accounts.lo_comment;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a comment that has been deleted
        require!(lo_comment.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the comment
        require_keys_eq!(lo_comment.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own comment since can't duplicate accounts
        if lo_comment.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            lo_chat.comment_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.comment_up_vote_score += vote_amount as u128;
            comment_section.comment_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted LO Comment From");
            msg!("User Address: {}", lo_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            lo_chat.comment_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.comment_down_vote_score += vote_amount.abs() as u128;
            comment_section.comment_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted LO Comment From");
            msg!("User Address: {}", lo_comment.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;  
        lo_comment.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn lo_reply_vote(ctx: Context<LOReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_reply = &mut ctx.accounts.lo_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(lo_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(lo_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if lo_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            lo_chat.reply_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_up_vote_score += vote_amount as u128;
            comment_section.reply_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted LO Reply From");
            msg!("User Address: {}", lo_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            lo_chat.reply_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted LO Reply From");
            msg!("User Address: {}", lo_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        lo_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn lo_lv3_reply_vote(ctx: Context<LOLv3ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(lo_lv3_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(lo_lv3_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if lo_lv3_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            lo_chat.reply_lv3_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv3_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted LO Lv3 Reply From");
            msg!("User Address: {}", lo_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            lo_chat.reply_lv3_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv3_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted LO Lv3 Reply From");
            msg!("User Address: {}", lo_lv3_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;
        lo_lv3_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn lo_lv4_reply_vote(ctx: Context<LOLv4ReplyVote>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        canidate_address: Pubkey,
        _chat_account_post_count_index: u128,
        _token_mint_address: Pubkey,
        vote_amount: i128) -> Result<()> 
    {
        let post_vote_stats = &mut ctx.accounts.post_vote_stats;
        let lo_chat = &mut ctx.accounts.lo_chat;
        let canidate_chat_account = &mut ctx.accounts.canidate_chat_account;
        let voter_chat_account = &mut ctx.accounts.voter_chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;
        let post_vote_record = &mut ctx.accounts.post_vote_record;
        
        //You can't vote for a reply that has been deleted
        require!(lo_lv4_reply.is_deleted == false, InvalidOperationError::Deleted);

        //You must vote for the person who wrote the reply
        require_keys_eq!(lo_lv4_reply.post_owner_address.key(), canidate_address.key(), InvalidOperationError::WrongDude);

        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        post_vote_record.voter_address = ctx.accounts.signer.key();
        post_vote_record.canidate_address = canidate_address.key();
        post_vote_record.unix_creation_time_stamp = Clock::get()?.unix_timestamp as u64;
        post_vote_record.vote_amount = vote_amount;

        //Add code to account for voter voting for their own reply since can't duplicate accounts
        if lo_lv4_reply.post_owner_address.key() == ctx.accounts.signer.key()
        {
            if is_up_vote
            {
                voter_chat_account.received_up_vote_score += vote_amount.abs() as u128;
                voter_chat_account.up_vote_received_count += 1;
            }
            else
            {
                voter_chat_account.received_down_vote_score += vote_amount.abs() as u128;
                voter_chat_account.down_vote_received_count += 1;
            }   
        }

        if is_up_vote
        {
            post_vote_stats.post_up_vote_count += 1;
            lo_chat.reply_lv4_up_vote_count += 1;
            comment_section.post_up_vote_score += vote_amount as u128;
            comment_section.post_up_vote_count += 1;
            comment_section.reply_to_lv3_reply_up_vote_score += vote_amount as u128;
            comment_section.reply_lv4_up_vote_count += 1;

            voter_chat_account.casted_up_vote_score += vote_amount as u128;
            voter_chat_account.up_vote_casted_count += 1;
            canidate_chat_account.received_up_vote_score += vote_amount as u128;
            canidate_chat_account.up_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted LO Lv4+ Reply From");
            msg!("User Address: {}", lo_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            post_vote_stats.post_down_vote_count += 1;
            lo_chat.reply_lv4_down_vote_count += 1;
            comment_section.post_down_vote_score += vote_amount.abs() as u128;
            comment_section.post_down_vote_count += 1;
            comment_section.reply_to_lv3_reply_down_vote_score += vote_amount.abs() as u128;
            comment_section.reply_lv4_down_vote_count += 1;

            voter_chat_account.casted_down_vote_score += vote_amount.abs() as u128;
            voter_chat_account.down_vote_casted_count += 1;
            canidate_chat_account.received_down_vote_score += vote_amount.abs() as u128;
            canidate_chat_account.down_vote_received_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted LO Lv4+ Reply From");
            msg!("User Address: {}", lo_lv4_reply.post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        post_vote_record.id = post_vote_stats.post_up_vote_count + post_vote_stats.post_down_vote_count;   
        lo_lv4_reply.net_vote_score += vote_amount;
        voter_chat_account.post_vote_casted_count += 1; //This is needed for the PostVoteRecord account. Couldn't add the up_vote_casted_count and down_vote_casted_count properties in the derived account seeds

        //This is in its own if block because it caused mutable to immutable borrow errors in the previous is_up_vote if block
        if is_up_vote
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_3CENTS * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;

            //Call the helper function to transfer the fee to the post owner
            send_turd_of_tree(
                accounts.user_fee_ata.to_account_info(),
                accounts.post_owner_usdc_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                canidate_address.key(),
                FEE_1CENT * vote_amount as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }
        else
        {
            let accounts = &ctx.accounts;
            let treasurer = ctx.accounts.treasurer.clone();

            //Call the helper function to transfer the fee
            apply_fee(
                accounts.user_fee_ata.to_account_info(),
                accounts.treasurer_fee_ata.to_account_info(),
                accounts.signer.to_account_info(),
                accounts.token_program.to_account_info(),
                treasurer,
                FEE_4CENTS * vote_amount.abs() as f64,
                accounts.fee_token_entry.decimal_amount
            )?;
        }

        Ok(())
    }

    pub fn star_m4a_comment(ctx: Context<StarM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_comment = &mut ctx.accounts.m4a_comment;
        //Can't set flag to the same state because of the counters
        require!(m4a_comment.is_starred != true, InvalidOperationError::FlagSameState);
    
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = m4a_comment.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        m4a_chat.ceo_starred_comment_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_comment_count += 1;
        m4a_comment.is_starred = true;  

        msg!("M4A Comment Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
  
        Ok(())
    }

    pub fn unstar_m4a_comment(ctx: Context<UnstarM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_comment = &mut ctx.accounts.m4a_comment;
        //Can't set flag to the same state because of the counters
        require!(m4a_comment.is_starred != false, InvalidOperationError::FlagSameState);
    
        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        
        idea_stats.protocol_deleted_idea_count += 1;

        m4a_chat.ceo_starred_comment_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_comment_count -= 1;
        m4a_comment.is_starred = false;  

        msg!("M4A Comment Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
  
        Ok(())
    }

    pub fn star_m4a_reply(ctx: Context<StarM4AReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_reply = &mut ctx.accounts.m4a_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = m4a_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        m4a_chat.ceo_starred_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_reply_count += 1;
        m4a_reply.is_starred = true;

        msg!("M4A Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_m4a_reply(ctx: Context<UnstarM4AReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_reply = &mut ctx.accounts.m4a_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        m4a_chat.ceo_starred_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_reply_count -= 1;
        m4a_reply.is_starred = false;

        msg!("M4A Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_m4a_lv3_reply(ctx: Context<StarM4ALv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv3_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = m4a_lv3_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        m4a_chat.ceo_starred_lv3_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv3_reply_count += 1;
        m4a_lv3_reply.is_starred = true;

        msg!("M4A Lv3 Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_m4a_lv3_reply(ctx: Context<UnstarM4ALv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv3_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        m4a_chat.ceo_starred_lv3_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv3_reply_count -= 1;
        m4a_lv3_reply.is_starred = false;

        msg!("M4A Lv3 Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_m4a_lv4_reply(ctx: Context<StarM4ALv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv4_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = m4a_lv4_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        m4a_chat.ceo_starred_lv4_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv4_reply_count += 1;
        m4a_lv4_reply.is_starred = true;

        msg!("M4A Lv4+ Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn unstar_m4a_lv4_reply(ctx: Context<UnstarM4ALv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv4_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        m4a_chat.ceo_starred_lv4_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv4_reply_count -= 1;
        m4a_lv4_reply.is_starred = false;

        msg!("M4A Lv4+ Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn star_pli_comment(ctx: Context<StarPLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_comment = &mut ctx.accounts.pli_comment;
        //Can't set flag to the same state because of the counters
        require!(pli_comment.is_starred != true, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = pli_comment.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;
        
        pli_chat.ceo_starred_comment_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_comment_count += 1;
        pli_comment.is_starred = true;

        msg!("PLI Comment Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_pli_comment(ctx: Context<UnstarPLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_comment = &mut ctx.accounts.pli_comment;
        //Can't set flag to the same state because of the counters
        require!(pli_comment.is_starred != false, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;
        
        pli_chat.ceo_starred_comment_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_comment_count -= 1;
        pli_comment.is_starred = false;

        msg!("PLI Comment Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn star_pli_reply(ctx: Context<StarPLIReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_reply = &mut ctx.accounts.pli_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = pli_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        pli_chat.ceo_starred_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_reply_count += 1;
        pli_reply.is_starred = true;

        msg!("PLI Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_pli_reply(ctx: Context<UnstarPLIReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_reply = &mut ctx.accounts.pli_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        pli_chat.ceo_starred_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_reply_count -= 1;
        pli_reply.is_starred = false;

        msg!("PLI Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
  
        Ok(())
    }

    pub fn star_pli_lv3_reply(ctx: Context<StarPLILv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv3_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = pli_lv3_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        pli_chat.ceo_starred_lv3_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv3_reply_count += 1;
        pli_lv3_reply.is_starred = true;

        msg!("PLI Lv3 Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_pli_lv3_reply(ctx: Context<UnstarPLILv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv3_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        pli_chat.ceo_starred_lv3_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv3_reply_count -= 1;
        pli_lv3_reply.is_starred = false;

        msg!("PLI Lv3 Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_pli_lv4_reply(ctx: Context<StarPLILv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv4_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = pli_lv4_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        pli_chat.ceo_starred_lv4_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv4_reply_count += 1;
        pli_lv4_reply.is_starred = true;

        msg!("PLI Lv4+ Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn unstar_pli_lv4_reply(ctx: Context<UnstarPLILv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv4_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        pli_chat.ceo_starred_lv4_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv4_reply_count -= 1;
        pli_lv4_reply.is_starred = false;

        msg!("PLI Lv4+ Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_about_comment(ctx: Context<StarAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_comment = &mut ctx.accounts.about_comment;
        //Can't set flag to the same state because of the counters
        require!(about_comment.is_starred != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = about_comment.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        about_chat.ceo_starred_comment_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_comment_count += 1;
        about_comment.is_starred = true;

        msg!("About Comment Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_about_comment(ctx: Context<UnstarAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_comment = &mut ctx.accounts.about_comment;
        //Can't set flag to the same state because of the counters
        require!(about_comment.is_starred != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        about_chat.ceo_starred_comment_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_comment_count -= 1;
        about_comment.is_starred = false;

        msg!("About Comment Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_about_reply(ctx: Context<StarAboutReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_reply = &mut ctx.accounts.about_reply;
        //Can't set flag to the same state because of the counters
        require!(about_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = about_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        about_chat.ceo_starred_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_reply_count += 1;
        about_reply.is_starred = true;

        msg!("About Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_about_reply(ctx: Context<UnstarAboutReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_reply = &mut ctx.accounts.about_reply;
        //Can't set flag to the same state because of the counters
        require!(about_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        about_chat.ceo_starred_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_reply_count -= 1;
        about_reply.is_starred = false;

        msg!("About Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn star_about_lv3_reply(ctx: Context<StarAboutLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv3_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = about_lv3_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        about_chat.ceo_starred_lv3_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv3_reply_count += 1;
        about_lv3_reply.is_starred = true;

        msg!("About Lv3 Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_about_lv3_reply(ctx: Context<UnstarAboutLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv3_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        about_chat.ceo_starred_lv3_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv3_reply_count -= 1;
        about_lv3_reply.is_starred = false;

        msg!("About Lv3 Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_about_lv4_reply(ctx: Context<StarAboutLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv4_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = about_lv4_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        about_chat.ceo_starred_lv4_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv4_reply_count += 1;
        about_lv4_reply.is_starred = true;

        msg!("About Lv4+ Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn unstar_about_lv4_reply(ctx: Context<UnstarAboutLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv4_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        about_chat.ceo_starred_lv4_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv4_reply_count -= 1;
        about_lv4_reply.is_starred = false;

        msg!("About Lv4+ Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn star_lo_comment(ctx: Context<StarLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_comment = &mut ctx.accounts.lo_comment;
        //Can't set flag to the same state because of the counters
        require!(lo_comment.is_starred != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = lo_comment.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        lo_chat.ceo_starred_comment_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_comment_count += 1;
        lo_comment.is_starred = true;

        msg!("LO Comment Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_lo_comment(ctx: Context<UnstarLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_comment = &mut ctx.accounts.lo_comment;
        //Can't set flag to the same state because of the counters
        require!(lo_comment.is_starred != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        lo_chat.ceo_starred_comment_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_comment_count -= 1;
        lo_comment.is_starred = false;

        msg!("LO Comment Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_lo_reply(ctx: Context<StarLOReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_reply = &mut ctx.accounts.lo_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = lo_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        lo_chat.ceo_starred_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_reply_count += 1;
        lo_reply.is_starred = true;

        msg!("LO Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_lo_reply(ctx: Context<UnstarLOReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_reply = &mut ctx.accounts.lo_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        lo_chat.ceo_starred_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_reply_count -= 1;
        lo_reply.is_starred = false;

        msg!("LO Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn star_lo_lv3_reply(ctx: Context<StarLOLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv3_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = lo_lv3_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        lo_chat.ceo_starred_lv3_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv3_reply_count += 1;
        lo_lv3_reply.is_starred = true;

        msg!("LO Lv3 Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unstar_lo_lv3_reply(ctx: Context<UnstarLOLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv3_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        lo_chat.ceo_starred_lv3_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv3_reply_count -= 1;
        lo_lv3_reply.is_starred = false;

        msg!("LO Lv3 Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn star_lo_lv4_reply(ctx: Context<StarLOLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv4_reply.is_starred != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;
        
        idea_stats.protocol_idea_count += 1;
        idea.id = idea_stats.protocol_idea_count;
        idea.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        idea.comment_section_name_prefix = comment_section_name_prefix.clone();
        idea.comment_section_name = comment_section_name.clone();
        idea.post_owner_address = post_owner_address;
        idea.chat_account_post_count_index = chat_account_post_count_index;
        idea.idea = lo_lv4_reply.msg.clone();
        idea.unix_creation_time_stamp = time_stamp;

        lo_chat.ceo_starred_lv4_reply_count += 1;
        chat_account.ceo_starred_comment_and_reply_count += 1;
        comment_section.ceo_starred_lv4_reply_count += 1;
        lo_lv4_reply.is_starred = true;

        msg!("LO Lv4+ Reply Starred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn unstar_lo_lv4_reply(ctx: Context<UnstarLOLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv4_reply.is_starred != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let idea_stats = &mut ctx.accounts.idea_stats;

        idea_stats.protocol_deleted_idea_count += 1;

        lo_chat.ceo_starred_lv4_reply_count -= 1;
        chat_account.ceo_starred_comment_and_reply_count -= 1;
        comment_section.ceo_starred_lv4_reply_count -= 1;
        lo_lv4_reply.is_starred = false;

        msg!("About Lv4+ Reply Unstarred By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
            
        Ok(())
    }

    pub fn set_idea_implemented_flag(ctx: Context<SetIdeaImplementedFlag>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        is_implemented: bool) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let idea = &mut ctx.accounts.idea;
        //Can't set flag to the same state
        require!(idea.is_implemented != is_implemented, InvalidOperationError::FlagSameState);

        let idea_stats = &mut ctx.accounts.idea_stats;
        idea_stats.updated_idea_count += 1;

        if is_implemented
        {
            let time_stamp = Clock::get()?.unix_timestamp as u64;

            idea.is_implemented = true;
            idea.implementation_time = time_stamp;

            msg!("Idea Implemented By CEO");
            msg!("For User: {}", post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Idea: {}", idea.idea);
        }
        else
        {
            idea.is_implemented = false;
            idea.implementation_time = 0;

            msg!("Idea Unimplemented By CEO");
            msg!("For User: {}", post_owner_address.key());
            msg!("Comment Section Prefix: {}", comment_section_name_prefix);
            msg!("Comment Section: {}", comment_section_name);
            msg!("Idea: {}", idea.idea);
        }
            
        Ok(())
    }

    pub fn update_idea(ctx: Context<UpdateIdea>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128,
        updated_idea: String) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        //Message string must not be longer than 444 characters
        require!(updated_idea.len() <= MAX_POST_LENGTH, InvalidLengthError::MSGTooLong);

        let idea_stats = &mut ctx.accounts.idea_stats;
        let idea = &mut ctx.accounts.idea;

        if idea.is_updated == false
        {
            idea.is_updated = true
        }

        idea.idea = updated_idea;
        idea_stats.updated_idea_count += 1;

        msg!("Idea Edited By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);
        msg!("Edited Idea: {}", idea.idea);
            
        Ok(())
    }

    pub fn fed_m4a_comment(ctx: Context<FEDM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()>
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_comment = &mut ctx.accounts.m4a_comment;
        //Can't set flag to the same state because of the counters
        require!(m4a_comment.is_fed != true, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = m4a_comment.msg.clone();
        fed_record.mark_time = time_stamp;

        if m4a_comment.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }
      
        m4a_chat.ceo_marked_fed_comment_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_comment_count += 1;
        m4a_comment.is_fed = true;

        msg!("M4A Comment Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_m4a_comment(ctx: Context<UnFEDM4AComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()>
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_comment = &mut ctx.accounts.m4a_comment;
        //Can't set flag to the same state because of the counters
        require!(m4a_comment.is_fed != false, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        m4a_chat.ceo_marked_fed_comment_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_comment_count -= 1;
        m4a_comment.is_fed = false;

        msg!("M4A Comment Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name);      
        
        Ok(())
    }

    pub fn fed_m4a_reply(ctx: Context<FEDM4AReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_reply = &mut ctx.accounts.m4a_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = m4a_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if m4a_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        m4a_chat.ceo_marked_fed_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_reply_count += 1;
        m4a_reply.is_fed = true;

        msg!("M4A Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_m4a_reply(ctx: Context<UnFEDM4AReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_reply = &mut ctx.accounts.m4a_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        m4a_chat.ceo_marked_fed_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_reply_count -= 1;
        m4a_reply.is_fed = false;
  
        msg!("M4A Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_m4a_lv3_reply(ctx: Context<FEDM4ALv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv3_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = m4a_lv3_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if m4a_lv3_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        m4a_chat.ceo_marked_fed_lv3_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv3_reply_count += 1;
        m4a_lv3_reply.is_fed = true;

        msg!("M4A Lv3 Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_m4a_lv3_reply(ctx: Context<UnFEDM4ALv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv3_reply = &mut ctx.accounts.m4a_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv3_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        m4a_chat.ceo_marked_fed_lv3_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv3_reply_count -= 1;
        m4a_lv3_reply.is_fed = false;

        msg!("M4A Lv3 Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_m4a_lv4_reply(ctx: Context<FEDM4ALv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv4_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = m4a_lv4_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if m4a_lv4_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        m4a_chat.ceo_marked_fed_lv4_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv4_reply_count += 1;
        m4a_lv4_reply.is_fed = true;

        msg!("M4A Lv4+ Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
 
        Ok(())
    }

    pub fn unfed_m4a_lv4_reply(ctx: Context<UnFEDM4ALv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let m4a_lv4_reply = &mut ctx.accounts.m4a_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(m4a_lv4_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let m4a_chat = &mut ctx.accounts.m4a_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        m4a_chat.ceo_marked_fed_lv4_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv4_reply_count -= 1;
        m4a_lv4_reply.is_fed = false;

        msg!("M4A Lv4+ Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_pli_comment(ctx: Context<FEDPLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_comment = &mut ctx.accounts.pli_comment;
        //Can't set flag to the same state because of the counters
        require!(pli_comment.is_fed != true, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = pli_comment.msg.clone();
        fed_record.mark_time = time_stamp;

        if pli_comment.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        pli_chat.ceo_marked_fed_comment_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_comment_count += 1;
        pli_comment.is_fed = true;

        msg!("PLI Comment Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_pli_comment(ctx: Context<UnFEDPLIComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_comment = &mut ctx.accounts.pli_comment;
        //Can't set flag to the same state because of the counters
        require!(pli_comment.is_fed != false, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        pli_chat.ceo_marked_fed_comment_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_comment_count -= 1;
        pli_comment.is_fed = false;

        msg!("PLI Comment Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_pli_reply(ctx: Context<FEDPLIReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_reply = &mut ctx.accounts.pli_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_reply.is_fed != true, InvalidOperationError::FlagSameState);
    
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = pli_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if pli_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        pli_chat.ceo_marked_fed_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_reply_count += 1;
        pli_reply.is_fed = true;

        msg!("PLI Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_pli_reply(ctx: Context<UnFEDPLIReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_reply = &mut ctx.accounts.pli_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_reply.is_fed != false, InvalidOperationError::FlagSameState);
    
        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        pli_chat.ceo_marked_fed_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_reply_count -= 1;
        pli_reply.is_fed = false;

        msg!("PLI Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_pli_lv3_reply(ctx: Context<FEDPLILv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv3_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = pli_lv3_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if pli_lv3_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        pli_chat.ceo_marked_fed_lv3_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv3_reply_count += 1;
        pli_lv3_reply.is_fed = true;

        msg!("PLI Lv3 Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_pli_lv3_reply(ctx: Context<UnFEDPLILv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv3_reply = &mut ctx.accounts.pli_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv3_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        pli_chat.ceo_marked_fed_lv3_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv3_reply_count -= 1;
        pli_lv3_reply.is_fed = false;

        msg!("PLI Lv3 Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_pli_lv4_reply(ctx: Context<FEDPLILv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv4_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = pli_lv4_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if pli_lv4_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        pli_chat.ceo_marked_fed_lv4_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv4_reply_count += 1;
        pli_lv4_reply.is_fed = true;

        msg!("PLI Lv4+ Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_pli_lv4_reply(ctx: Context<UnFEDPLILv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let pli_lv4_reply = &mut ctx.accounts.pli_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(pli_lv4_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let pli_chat = &mut ctx.accounts.pli_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        pli_chat.ceo_marked_fed_lv4_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv4_reply_count -= 1;
        pli_lv4_reply.is_fed = false;

        msg!("PLI Lv4+ Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_about_comment(ctx: Context<FEDAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_comment = &mut ctx.accounts.about_comment;
        //Can't set flag to the same state because of the counters
        require!(about_comment.is_fed != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = about_comment.msg.clone();
        fed_record.mark_time = time_stamp;

        if about_comment.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        about_chat.ceo_marked_fed_comment_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_comment_count += 1;
        about_comment.is_fed = true;

        msg!("About Comment Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_about_comment(ctx: Context<UnFEDAboutComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_comment = &mut ctx.accounts.about_comment;
        //Can't set flag to the same state because of the counters
        require!(about_comment.is_fed != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        about_chat.ceo_marked_fed_comment_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_comment_count -= 1;
        about_comment.is_fed = false;

        msg!("About Comment Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_about_reply(ctx: Context<FEDAboutReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_reply = &mut ctx.accounts.about_reply;
        //Can't set flag to the same state because of the counters
        require!(about_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = about_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if about_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        about_chat.ceo_marked_fed_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_reply_count += 1;
        about_reply.is_fed = true;

        msg!("About Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_about_reply(ctx: Context<UnFEDAboutReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_reply = &mut ctx.accounts.about_reply;
        //Can't set flag to the same state because of the counters
        require!(about_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        about_chat.ceo_marked_fed_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_reply_count -= 1;
        about_reply.is_fed = false;

        msg!("About Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_about_lv3_reply(ctx: Context<FEDAboutLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv3_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = about_lv3_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if about_lv3_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        about_chat.ceo_marked_fed_lv3_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv3_reply_count += 1;
        about_lv3_reply.is_fed = true;

        msg!("About Lv3 Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_about_lv3_reply(ctx: Context<UnFEDAboutLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv3_reply = &mut ctx.accounts.about_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv3_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        about_chat.ceo_marked_fed_lv3_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv3_reply_count -= 1;
        about_lv3_reply.is_fed = false;

        msg!("About Lv3 Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_about_lv4_reply(ctx: Context<FEDAboutLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv4_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = about_lv4_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if about_lv4_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        about_chat.ceo_marked_fed_lv4_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv4_reply_count += 1;
        about_lv4_reply.is_fed = true;

        msg!("About Lv4+ Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_about_lv4_reply(ctx: Context<UnFEDAboutLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let about_lv4_reply = &mut ctx.accounts.about_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(about_lv4_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let about_chat = &mut ctx.accounts.about_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;
        
        about_chat.ceo_marked_fed_lv4_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv4_reply_count -= 1;
        about_lv4_reply.is_fed = false;

        msg!("About Lv4+ Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_lo_comment(ctx: Context<FEDLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_comment = &mut ctx.accounts.lo_comment;
        //Can't set flag to the same state because of the counters
        require!(lo_comment.is_fed != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Comment as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = lo_comment.msg.clone();
        fed_record.mark_time = time_stamp;

        if lo_comment.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        lo_chat.ceo_marked_fed_comment_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_comment_count += 1;
        lo_comment.is_fed = true;

        msg!("LO Comment Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_lo_comment(ctx: Context<UnFEDLOComment>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_comment = &mut ctx.accounts.lo_comment;
        //Can't set flag to the same state because of the counters
        require!(lo_comment.is_fed != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        lo_chat.ceo_marked_fed_comment_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_comment_count -= 1;
        lo_comment.is_fed = false;

        msg!("LO Comment Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_lo_reply(ctx: Context<FEDLOReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_reply = &mut ctx.accounts.lo_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = lo_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if lo_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        lo_chat.ceo_marked_fed_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_reply_count += 1;
        lo_reply.is_fed = true;

        msg!("LO Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_lo_reply(ctx: Context<UnFEDLOReply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_reply = &mut ctx.accounts.lo_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        lo_chat.ceo_marked_fed_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_reply_count -= 1;
        lo_reply.is_fed = false;

        msg!("LO Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_lo_lv3_reply(ctx: Context<FEDLOLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv3_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv3Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = lo_lv3_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if lo_lv3_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        lo_chat.ceo_marked_fed_lv3_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv3_reply_count += 1;
        lo_lv3_reply.is_fed = true;

        msg!("LO Lv3 Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_lo_lv3_reply(ctx: Context<UnFEDLOLv3Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv3_reply = &mut ctx.accounts.lo_lv3_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv3_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;

        lo_chat.ceo_marked_fed_lv3_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv3_reply_count -= 1;
        lo_lv3_reply.is_fed = false;

        msg!("LO Lv3 Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn fed_lo_lv4_reply(ctx: Context<FEDLOLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv4_reply.is_fed != true, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        let fed_record = &mut ctx.accounts.fed_record;
        
        fed_stats.federal_agent_post_count += 1;
        fed_record.id = fed_stats.federal_agent_post_count;
        fed_record.post_type = PostType::Lv4Reply as u8;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        fed_record.comment_section_name_prefix = comment_section_name_prefix.clone();
        fed_record.comment_section_name = comment_section_name.clone();
        fed_record.post_owner_address = post_owner_address;
        fed_record.chat_account_post_count_index = chat_account_post_count_index;
        fed_record.post = lo_lv4_reply.msg.clone();
        fed_record.mark_time = time_stamp;

        if lo_lv4_reply.is_edited == true
        {
            fed_record.was_edited_before_mark = true;
        }

        lo_chat.ceo_marked_fed_lv4_reply_count += 1;
        chat_account.ceo_marked_fed_comment_and_reply_count += 1;
        comment_section.ceo_marked_fed_lv4_reply_count += 1;
        lo_lv4_reply.is_fed = true;

        msg!("LO Lv4+ Reply Marked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn unfed_lo_lv4_reply(ctx: Context<UnFEDLOLv4Reply>,
        comment_section_name_prefix: String,
        comment_section_name: String,
        post_owner_address: Pubkey,
        _chat_account_post_count_index: u128) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let lo_lv4_reply = &mut ctx.accounts.lo_lv4_reply;
        //Can't set flag to the same state because of the counters
        require!(lo_lv4_reply.is_fed != false, InvalidOperationError::FlagSameState);

        let lo_chat = &mut ctx.accounts.lo_chat;
        let chat_account = &mut ctx.accounts.chat_account;
        let comment_section = &mut ctx.accounts.comment_section;
        let fed_stats = &mut ctx.accounts.fed_stats;
        
        fed_stats.deleted_federal_agent_post_count += 1;
        
        lo_chat.ceo_marked_fed_lv4_reply_count -= 1;
        chat_account.ceo_marked_fed_comment_and_reply_count -= 1;
        comment_section.ceo_marked_fed_lv4_reply_count -= 1;
        lo_lv4_reply.is_fed = false;

        msg!("LO Lv4+ Reply Unmarked As Federal Agent By CEO");
        msg!("For User: {}", post_owner_address.key());
        msg!("Comment Section Prefix: {}", comment_section_name_prefix);
        msg!("Comment Section: {}", comment_section_name); 
            
        Ok(())
    }

    pub fn create_poll(ctx: Context<CreatePoll>, poll_name: String) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        //Poll name string must not be longer than 144 characters
        require!(poll_name.len() <= MAX_POLL_AND_POLL_OPTION_NAME_LENGTH, InvalidLengthError::PollOrPollOptionNameTooLong);

        let poll_stats = &mut ctx.accounts.poll_stats;
        let poll = &mut ctx.accounts.poll;
        let time_stamp = Clock::get()?.unix_timestamp as u64;

        poll.is_active = true;
        poll.poll_index = poll_stats.poll_count;
        poll.poll_name = poll_name.clone();
        poll.unix_creation_time_stamp = time_stamp;

        poll_stats.poll_count += 1;

        msg!("New Poll Created: #{}", poll_stats.poll_count);
        msg!("Poll Name: {}", poll_name);

        Ok(())
    }

    pub fn edit_poll(ctx: Context<EditPoll>, _poll_index: u128, poll_name: String) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        //Poll name string must not be longer than 144 characters
        require!(poll_name.len() <= MAX_POLL_AND_POLL_OPTION_NAME_LENGTH, InvalidLengthError::PollOrPollOptionNameTooLong);

        let poll_stats = &mut ctx.accounts.poll_stats;
        let poll = &mut ctx.accounts.poll;

        poll.poll_name = poll_name;
        poll_stats.edited_poll_or_poll_option_count += 1;

        msg!("Edited Poll");
        msg!("New Poll Name: {}", poll.poll_name);

        Ok(())
    }

    pub fn set_poll_flag(ctx: Context<SetPollFlag>, _poll_index: u128, is_active: bool) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let poll = &mut ctx.accounts.poll;
        //Can't set flag to the same state
        require!(poll.is_active != is_active, InvalidOperationError::FlagSameState);

        let poll_stats = &mut ctx.accounts.poll_stats;

        poll.is_active = is_active;
        poll_stats.edited_poll_or_poll_option_count += 1;
        
        msg!("Updated Poll Active Flag To: {}", is_active);
        msg!("Poll Name: {}", poll.poll_name);

        Ok(())
    }

    pub fn create_poll_option(ctx: Context<CreatePollOption>, _poll_index: u128, poll_option_name: String) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        //Poll option name string must not be longer than 144 characters
        require!(poll_option_name.len() <= MAX_POLL_AND_POLL_OPTION_NAME_LENGTH, InvalidLengthError::PollOrPollOptionNameTooLong);

        let poll_stats = &mut ctx.accounts.poll_stats;
        let poll = &mut ctx.accounts.poll;
        let poll_option = &mut ctx.accounts.poll_option;
        let time_stamp = Clock::get()?.unix_timestamp as u64;

        poll_option.is_active = true;
        poll_option.poll_option_index = poll.option_count;
        poll_option.poll_option_name = poll_option_name;
        poll_option.unix_creation_time_stamp = time_stamp;

        poll_stats.option_count += 1;
        poll.option_count += 1;

        msg!("New Poll Option Created");
        msg!("Poll Name: {}", poll.poll_name);
        msg!("Poll Option Name: {}", poll_option.poll_option_name);

        Ok(())
    }

    pub fn edit_poll_option(ctx: Context<EditPollOption>, _poll_index: u128, _poll_option_index: u8, poll_option_name: String) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        //Poll option name string must not be longer than 144 characters
        require!(poll_option_name.len() <= MAX_POLL_AND_POLL_OPTION_NAME_LENGTH, InvalidLengthError::PollOrPollOptionNameTooLong);

        let poll_stats = &mut ctx.accounts.poll_stats;
        let poll = &mut ctx.accounts.poll;
        let poll_option = &mut ctx.accounts.poll_option;

        poll_stats.edited_poll_or_poll_option_count += 1;
        poll.edited_poll_option_count += 1;
        poll_option.poll_option_name = poll_option_name;

        msg!("Edited Poll Option");
        msg!("Poll: {}", poll.poll_name);
        msg!("New Poll Option Name: {}", poll_option.poll_option_name);

        Ok(())
    }

    pub fn set_poll_option_flag(ctx: Context<SetPollOptionFlag>,
        _poll_index: u128,
        _poll_option_index: u8,
        is_active: bool) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let poll_option = &mut ctx.accounts.poll_option;
        //Can't set flag to the same state
        require!(poll_option.is_active != is_active, InvalidOperationError::FlagSameState);

        let poll = &mut ctx.accounts.poll;
        let poll_stats = &mut ctx.accounts.poll_stats;
        
        poll_stats.edited_poll_or_poll_option_count += 1;
        poll.edited_poll_option_count += 1;
        poll_option.is_active = is_active;

        msg!("Updated Poll Option Active Flag To: {}", is_active);
        msg!("Poll: {}", poll.poll_name);
        msg!("Poll Option: {}", poll_option.poll_option_name);

        Ok(())
    }

    pub fn vote_poll_option(ctx: Context<VotePollOption>, poll_index: u128, poll_option_index: u8, _token_mint_address: Pubkey, vote_amount: i128) -> Result<()> 
    {
        //You can not vote a 0 ammount
        require!(vote_amount != 0, InvalidOperationError::CantVoteZeroAmount);

        let mut is_up_vote = false;

        if vote_amount > 0
        {
            is_up_vote = true;
        }

        let chat_account = &mut ctx.accounts.chat_account;
        
        let poll_stats = &mut ctx.accounts.poll_stats;
        let poll_vote_stats = &mut ctx.accounts.poll_vote_stats;
        let poll = &mut ctx.accounts.poll;
        let poll_option = &mut ctx.accounts.poll_option;
        let poll_vote_record = &mut ctx.accounts.poll_vote_record;

        let time_stamp = Clock::get()?.unix_timestamp as u64;

        if is_up_vote
        {
            poll_stats.up_vote_score += vote_amount as u128;
            poll_vote_stats.up_vote_count += 1;
            poll.up_vote_score += vote_amount as u128;
            poll.up_vote_count += 1;
            poll_option.up_vote_score += vote_amount as u128;
            poll_option.up_vote_count += 1;
           
            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Up Voted Poll Option");
            msg!("Poll: {}", poll.poll_name);
            msg!("Poll Option: {}", poll_option.poll_option_name);
            msg!("Vote Amount: {}", vote_amount);
        }
        else
        {
            poll_stats.down_vote_score += vote_amount.abs() as u128;
            poll_vote_stats.down_vote_count += 1;
            poll.down_vote_score += vote_amount.abs() as u128;
            poll.down_vote_count += 1;
            poll_option.down_vote_score += vote_amount.abs() as u128;
            poll_option.down_vote_count += 1;

            msg!("User Address: {}", ctx.accounts.signer.key());
            msg!("Down Voted Poll Option");
            msg!("Poll: {}", poll.poll_name);
            msg!("Poll Option: {}", poll_option.poll_option_name);
            msg!("Vote Amount: {}", vote_amount);
        }

        poll_vote_record.vote_amount = vote_amount;
        poll_vote_record.protocol_record_id = poll_vote_stats.up_vote_count + poll_vote_stats.down_vote_count;
        poll_vote_record.poll_record_id = poll.up_vote_count + poll.down_vote_count;
        poll_vote_record.poll_index = poll_index;
        poll_vote_record.poll_option_index = poll_option_index;
        poll_vote_record.voter_address = ctx.accounts.signer.key();
        poll_vote_record.unix_creation_time_stamp = time_stamp;

        chat_account.poll_vote_count += 1;

        let accounts = &ctx.accounts;
        let treasurer = ctx.accounts.treasurer.clone();

        //Call the helper function to transfer the fee
        apply_fee(
            accounts.user_fee_ata.to_account_info(),
            accounts.treasurer_fee_ata.to_account_info(),
            accounts.signer.to_account_info(),
            accounts.token_program.to_account_info(),
            treasurer,
            FEE_4CENTS * vote_amount.abs() as f64,
            accounts.fee_token_entry.decimal_amount
        )?;

        Ok(())
    }

    pub fn clock_in_dead_mans_break(ctx: Context<ClockInDeadMansBreak>) -> Result<()> 
    {
        let ceo = &mut ctx.accounts.ceo;
        //Only the CEO can call this function
        require_keys_eq!(ctx.accounts.signer.key(), ceo.address.key(), AuthorizationError::NotCEO);

        let dead_mans_break = &mut ctx.accounts.dead_mans_break;
        dead_mans_break.unix_clock_in_time_stamp = Clock::get()?.unix_timestamp as u64;

        msg!("Dead Mans Break Refreshed");

        Ok(())
    }
}   

//Derived Accounts
#[derive(Accounts)]
pub struct InitializeAdminAccounts<'info> 
{
    #[account(
        init, 
        payer = signer,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump,
        space = size_of::<ChatProtocolCEO>() + 8)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        init, 
        payer = signer,
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump,
        space = size_of::<ChatProtocolTreasurer>() + 8)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"ideaStats".as_ref()], 
        bump, 
        space = size_of::<IdeaStats>() + 8)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedStats".as_ref()], 
        bump, 
        space = size_of::<FEDStats>() + 8)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct PassOnChatProtocolCEO<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct PassOnChatProtocolTreasurer<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(token_mint_address: Pubkey)]
pub struct AddFeeTokenEntry<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump, 
        space = size_of::<FeeTokenEntry>() + 8)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(token_mint_address: Pubkey)]
pub struct RemoveFeeTokenEntry<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut,
        close = signer,
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeQualityOfLifeAccounts<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"chatAccountStats".as_ref()], 
        bump, 
        space = size_of::<ChatAccountStats>() + 8)]
    pub chat_account_stats: Account<'info, ChatAccountStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"commentSectionStats".as_ref()], 
        bump, 
        space = size_of::<CommentSectionStats>() + 8)]
    pub comment_section_stats: Account<'info, CommentSectionStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pollStats".as_ref()], 
        bump, 
        space = size_of::<PollStats>() + 8)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        init, 
        payer = signer,
        seeds = [b"deadMansBreak".as_ref()],
        bump,
        space = size_of::<DeadMansBreak>() + 8)]
    pub dead_mans_break_counter: Account<'info, DeadMansBreak>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ClockInDeadMansBreak<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut,
        seeds = [b"deadMansBreak".as_ref()],
        bump)]
    pub dead_mans_break: Account<'info, DeadMansBreak>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeChatProtocol<'info> 
{
    #[account(
        init, 
        payer = signer, 
        seeds = [b"chatProtocol".as_ref()], 
        bump, 
        space = size_of::<ChatProtocol>() + 8)]
    pub chat_protocol: Account<'info, ChatProtocol>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteStats".as_ref()], 
        bump, 
        space = size_of::<PostVoteStats>() + 8)]
    pub post_vote_stats: Account<'info, PostVoteStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"videoVoteStats".as_ref()], 
        bump, 
        space = size_of::<VideoVoteStats>() + 8)]
    pub video_vote_stats: Account<'info, VideoVoteStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pollVoteStats".as_ref()], 
        bump, 
        space = size_of::<PollVoteStats>() + 8)]
    pub poll_vote_stats: Account<'info, PollVoteStats>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeM4AChat<'info> 
{
    #[account(
        init, 
        payer = signer, 
        seeds = [b"m4aChat".as_ref()], 
        bump, 
        space = size_of::<M4AChat>() + 8)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializePLIChat<'info> 
{
    #[account(
        init, 
        payer = signer, 
        seeds = [b"pliChat".as_ref()], 
        bump, 
        space = size_of::<PLIChat>() + 8)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeAboutChat<'info> 
{
    #[account(
        init, 
        payer = signer, 
        seeds = [b"aboutChat".as_ref()], 
        bump, 
        space = size_of::<AboutChat>() + 8)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeLOChat<'info> 
{
    #[account(
        init, 
        payer = signer, 
        seeds = [b"loChat".as_ref()], 
        bump, 
        space = size_of::<LOChat>() + 8)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct CreateChatAccount<'info> 
{
    #[account(
        mut, 
        seeds = [b"chatAccountStats".as_ref()], 
        bump)]
    pub chat_account_stats: Account<'info, ChatAccountStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump, 
        space = size_of::<ChatAccount>() + CHAT_ACCOUNT_EXTRA_SIZE + 8)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(token_mint_address: Pubkey)]
pub struct UpdateUserName<'info> 
{
    #[account(
        mut, 
        seeds = [b"chatAccountStats".as_ref()], 
        bump)]
    pub chat_account_stats: Account<'info, ChatAccountStats>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(token_mint_address: Pubkey)]
pub struct SetUseCustomNameFlag<'info>
{
    #[account(
        mut, 
        seeds = [b"chatAccountStats".as_ref()], 
        bump)]
    pub chat_account_stats: Account<'info, ChatAccountStats>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String)]
pub struct CreateCommentSection<'info> 
{
    #[account(
        mut, 
        seeds = [b"commentSectionStats".as_ref()], 
        bump)]
    pub comment_section_stats: Account<'info, CommentSectionStats>,

    //Ensures that only someone with a chat account can create a comment section, update: I ended up setting the hasGoodEnding flag here. But you can still include a derived account and not actually do anything with it in the function to make sure it exists
    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump, 
        space = size_of::<CommentSection>() + COMMENT_SECTION_EXTRA_SIZE + 8)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String)]
pub struct SetCommentSectionFlag<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"commentSectionStats".as_ref()], 
        bump)]
    pub comment_section_stats: Account<'info, CommentSectionStats>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Account<'info, CommentSection>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, token_mint_address: Pubkey)]
pub struct CommentSectionVote<'info> 
{
    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSectionStats".as_ref()], 
        bump)]
    pub comment_section_stats: Account<'info, CommentSectionStats>,
    
    #[account(
        mut,
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut, 
        seeds = [b"videoVoteStats".as_ref()], 
        bump)]
    pub video_vote_stats: Account<'info, VideoVoteStats>,

     #[account(
        init, 
        payer = signer, 
        seeds = [b"videoVoteRecord".as_ref(), signer.key().as_ref(), chat_account.video_vote_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<VideoVoteRecord>() + 8)]
    pub video_vote_record: Account<'info, VideoVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, token_mint_address: Pubkey)]
pub struct PostM4AComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<M4AComment>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub m4a_comment: Account<'info, M4AComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Box<Account<'info, ChatProtocolCEO>>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToM4AComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub m4a_comment: Box<Account<'info, M4AComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<M4AReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub m4a_reply: Account<'info, M4AReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToM4AReply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub m4a_reply: Box<Account<'info, M4AReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<M4AReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub m4a_lv3_reply: Account<'info, M4ALv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToM4ALv3Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub m4a_lv3_reply: Box<Account<'info, M4ALv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<M4AReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub m4a_lv4_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToM4ALv4Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub m4a_lv4_reply: Box<Account<'info, M4ALv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<M4AReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub m4a_lv4_plus_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, token_mint_address: Pubkey)]
pub struct PostPLIComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<PLIComment>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub pli_comment: Account<'info, PLIComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Box<Account<'info, ChatProtocolCEO>>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToPLIComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub pli_comment: Box<Account<'info, PLIComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<PLIReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub pli_reply: Account<'info, PLIReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToPLIReply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub pli_reply: Box<Account<'info, PLIReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<PLIReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub pli_lv3_reply: Account<'info, PLILv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToPLILv3Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub pli_lv3_reply: Box<Account<'info, PLILv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<PLIReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub pli_lv4_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToPLILv4Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub pli_lv4_reply: Box<Account<'info, PLILv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<PLIReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub pli_lv4_plus_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, token_mint_address: Pubkey)]
pub struct PostAboutComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<AboutComment>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub about_comment: Account<'info, AboutComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Box<Account<'info, ChatProtocolCEO>>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToAboutComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()],
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub about_comment: Box<Account<'info, AboutComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<AboutReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub about_reply: Account<'info, AboutReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToAboutReply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub about_reply: Box<Account<'info, AboutReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<AboutReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub about_lv3_reply: Account<'info, AboutLv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToAboutLv3Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub about_lv3_reply: Box<Account<'info, AboutLv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<AboutReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub about_lv4_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToAboutLv4Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub about_lv4_reply: Box<Account<'info, AboutLv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<AboutReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub about_lv4_plus_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, token_mint_address: Pubkey)]
pub struct PostLOComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<LOComment>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub lo_comment: Account<'info, LOComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Box<Account<'info, ChatProtocolCEO>>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToLOComment<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()],
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub lo_comment: Box<Account<'info, LOComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<LOReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub lo_reply: Account<'info, LOReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToLOReply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub lo_reply: Box<Account<'info, LOReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<LOReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub lo_lv3_reply: Account<'info, LOLv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToLOLv3Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub lo_lv3_reply: Box<Account<'info, LOLv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<LOReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub lo_lv4_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct ReplyToLOLv4Reply<'info> 
{
    #[account(
        mut,
        seeds = [b"chatProtocol".as_ref()], 
        bump)]
    pub chat_protocol: Box<Account<'info, ChatProtocol>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub lo_lv4_reply: Box<Account<'info, LOLv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account.comment_and_reply_count.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump, 
        space = size_of::<LOReply>() + COMMENT_REPLY_OR_IDEA_EXTRA_SIZE + 8)]
    pub lo_lv4_plus_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditM4AComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_comment: Account<'info, M4AComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditM4AReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_reply: Account<'info, M4AReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditM4ALv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_lv3_reply: Account<'info, M4ALv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditM4ALv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_lv4_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditPLIComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_comment: Account<'info, PLIComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditPLIReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_reply: Account<'info, PLIReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditPLILv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_lv3_reply: Account<'info, PLILv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditPLILv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_lv4_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditAboutComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_comment: Account<'info, AboutComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditAboutReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_reply: Account<'info, AboutReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditAboutLv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_lv3_reply: Account<'info, AboutLv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditAboutLv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_lv4_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditLOComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_comment: Account<'info, LOComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditLOReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_reply: Account<'info, LOReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditLOLv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_lv3_reply: Account<'info, LOLv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct EditLOLv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_lv4_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteM4AComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_comment: Account<'info, M4AComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteM4AReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_reply: Account<'info, M4AReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteM4ALv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_lv3_reply: Account<'info, M4ALv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteM4ALv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub m4a_lv4_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeletePLIComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_comment: Account<'info, PLIComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeletePLIReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_reply: Account<'info, PLIReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeletePLILv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_lv3_reply: Account<'info, PLILv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeletePLILv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub pli_lv4_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteAboutComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_comment: Account<'info, AboutComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteAboutReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_reply: Account<'info, AboutReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteAboutLv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_lv3_reply: Account<'info, AboutLv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteAboutLv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub about_lv4_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteLOComment<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_comment: Account<'info, LOComment>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteLOReply<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_reply: Account<'info, LOReply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteLOLv3Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_lv3_reply: Account<'info, LOLv3Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String, comment_section_name: String, chat_account_post_count_index: u128, token_mint_address: Pubkey)]
pub struct DeleteLOLv4Reply<'info> 
{
    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        signer.key().as_ref()], 
        bump)]
    pub lo_lv4_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct M4ACommentVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub m4a_comment: Box<Account<'info, M4AComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), m4a_comment.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = m4a_comment.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,


    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}



#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct M4AReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub m4a_reply: Box<Account<'info, M4AReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), m4a_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = m4a_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct M4ALv3ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub m4a_lv3_reply: Box<Account<'info, M4ALv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), m4a_lv3_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = m4a_lv3_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct M4ALv4ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Box<Account<'info, M4AChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub m4a_lv4_reply: Box<Account<'info, M4ALv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), m4a_lv4_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = m4a_lv4_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct PLICommentVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub pli_comment: Box<Account<'info, PLIComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), pli_comment.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = pli_comment.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct PLIReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub pli_reply: Box<Account<'info, PLIReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), pli_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = pli_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct PLILv3ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub pli_lv3_reply: Box<Account<'info, PLILv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), pli_lv3_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = pli_lv3_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct PLILv4ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Box<Account<'info, PLIChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub pli_lv4_reply: Box<Account<'info, PLILv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), pli_lv4_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = pli_lv4_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct AboutCommentVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub about_comment: Box<Account<'info, AboutComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), about_comment.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = about_comment.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct AboutReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub about_reply: Box<Account<'info, AboutReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), about_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = about_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct AboutLv3ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub about_lv3_reply: Box<Account<'info, AboutLv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), about_lv3_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = about_lv3_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct AboutLv4ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Box<Account<'info, AboutChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub about_lv4_reply: Box<Account<'info, AboutLv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), about_lv4_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = about_lv4_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct LOCommentVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub lo_comment: Box<Account<'info, LOComment>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), lo_comment.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = lo_comment.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct LOReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub lo_reply: Box<Account<'info, LOReply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), lo_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = lo_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct LOLv3ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub lo_lv3_reply: Box<Account<'info, LOLv3Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), lo_lv3_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = lo_lv3_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    canidate_address: Pubkey,
    chat_account_post_count_index: u128,
    token_mint_address: Pubkey)]
pub struct LOLv4ReplyVote<'info> 
{
    #[account(
        mut,
        seeds = [b"postVoteStats".as_ref()], 
        bump)]
    pub post_vote_stats: Box<Account<'info, PostVoteStats>>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Box<Account<'info, LOChat>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), canidate_address.key().as_ref()], 
        bump)]
    pub canidate_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub voter_chat_account: Box<Account<'info, ChatAccount>>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        canidate_address.key().as_ref()],
        bump)]
    pub lo_lv4_reply: Box<Account<'info, LOLv4Reply>>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"postVoteRecord".as_ref(), signer.key().as_ref(), lo_lv4_reply.post_owner_address.key().as_ref(), voter_chat_account.post_vote_casted_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PostVoteRecord>() + 8)]
    pub post_vote_record: Account<'info, PostVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,
    
    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = lo_lv4_reply.post_owner_address
    )]
    pub post_owner_usdc_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarM4AComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_comment: Account<'info, M4AComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarM4AComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_comment: Account<'info, M4AComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarM4AReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_reply: Account<'info, M4AReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarM4AReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_reply: Account<'info, M4AReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarM4ALv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv3_reply: Account<'info, M4ALv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarM4ALv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv3_reply: Account<'info, M4ALv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarM4ALv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv4_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarM4ALv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv4_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarPLIComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_comment: Account<'info, PLIComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarPLIComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_comment: Account<'info, PLIComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarPLIReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_reply: Account<'info, PLIReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarPLIReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_reply: Account<'info, PLIReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarPLILv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv3_reply: Account<'info, PLILv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarPLILv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv3_reply: Account<'info, PLILv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarPLILv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv4_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarPLILv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv4_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarAboutComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_comment: Account<'info, AboutComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarAboutComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_comment: Account<'info, AboutComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarAboutReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_reply: Account<'info, AboutReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarAboutReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_reply: Account<'info, AboutReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarAboutLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv3_reply: Account<'info, AboutLv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarAboutLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv3_reply: Account<'info, AboutLv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarAboutLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv4_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarAboutLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv4_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarLOComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_comment: Account<'info, LOComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarLOComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_comment: Account<'info, LOComment>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarLOReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_reply: Account<'info, LOReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarLOReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_reply: Account<'info, LOReply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarLOLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv3_reply: Account<'info, LOLv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarLOLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv3_reply: Account<'info, LOLv3Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct StarLOLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv4_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<Idea>() + IDEA_EXTRA_SIZE + 8)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnstarLOLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv4_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct SetIdeaImplementedFlag<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UpdateIdea<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut,
        seeds = [b"ideaStats".as_ref()], 
        bump)]
    pub idea_stats: Account<'info, IdeaStats>,

    #[account(
        mut,
        seeds = [b"idea".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub idea: Account<'info, Idea>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDM4AComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_comment: Account<'info, M4AComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDM4AComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_comment: Account<'info, M4AComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDM4AReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_reply: Account<'info, M4AReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDM4AReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_reply: Account<'info, M4AReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDM4ALv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv3_reply: Account<'info, M4ALv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDM4ALv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv3_reply: Account<'info, M4ALv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDM4ALv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv4_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDM4ALv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"m4aChat".as_ref()], 
        bump)]
    pub m4a_chat: Account<'info, M4AChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"m4aLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub m4a_lv4_reply: Account<'info, M4ALv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDPLIComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_comment: Account<'info, PLIComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDPLIComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_comment: Account<'info, PLIComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDPLIReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_reply: Account<'info, PLIReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDPLIReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_reply: Account<'info, PLIReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDPLILv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv3_reply: Account<'info, PLILv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDPLILv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv3_reply: Account<'info, PLILv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDPLILv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv4_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDPLILv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pliChat".as_ref()], 
        bump)]
    pub pli_chat: Account<'info, PLIChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"pliLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub pli_lv4_reply: Account<'info, PLILv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDAboutComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_comment: Account<'info, AboutComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDAboutComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_comment: Account<'info, AboutComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDAboutReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_reply: Account<'info, AboutReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDAboutReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_reply: Account<'info, AboutReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDAboutLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv3_reply: Account<'info, AboutLv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDAboutLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv3_reply: Account<'info, AboutLv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDAboutLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv4_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDAboutLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"aboutChat".as_ref()], 
        bump)]
    pub about_chat: Account<'info, AboutChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"aboutLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub about_lv4_reply: Account<'info, AboutLv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDLOComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_comment: Account<'info, LOComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDLOComment<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loComment".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_comment: Account<'info, LOComment>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDLOReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_reply: Account<'info, LOReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDLOReply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loReply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_reply: Account<'info, LOReply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDLOLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv3_reply: Account<'info, LOLv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDLOLv3Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv3Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv3_reply: Account<'info, LOLv3Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct FEDLOLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv4_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump, 
        space = size_of::<FEDRecord>() + FEDERAL_AGENT_EXTRA_SIZE + 8)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(comment_section_name_prefix: String,
    comment_section_name: String,
    post_owner_address: Pubkey,
    chat_account_post_count_index: u128)]
pub struct UnFEDLOLv4Reply<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"loChat".as_ref()], 
        bump)]
    pub lo_chat: Account<'info, LOChat>,

    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), post_owner_address.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"commentSection".as_ref(), comment_section_name_prefix.as_ref(), comment_section_name.as_ref()], 
        bump)]
    pub comment_section: Box<Account<'info, CommentSection>>,

    #[account(
        mut,
        seeds = [b"loLv4Reply".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()],
        bump)]
    pub lo_lv4_reply: Account<'info, LOLv4Reply>,

    #[account(
        mut,
        seeds = [b"fedStats".as_ref()], 
        bump)]
    pub fed_stats: Account<'info, FEDStats>,

    #[account(
        mut,
        close = signer,
        seeds = [b"fedRecord".as_ref(),
        comment_section_name_prefix.as_ref(),
        comment_section_name.as_ref(),
        chat_account_post_count_index.to_le_bytes().as_ref(),
        post_owner_address.key().as_ref()], 
        bump)]
    pub fed_record: Account<'info, FEDRecord>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct CreatePoll<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pollStats".as_ref()], 
        bump)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"poll".as_ref(), poll_stats.poll_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<Poll>() + POLL_AND_POLL_OPTION_EXTRA_SIZE + 8)]
    pub poll: Account<'info, Poll>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(poll_index: u128)]
pub struct EditPoll<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pollStats".as_ref()], 
        bump)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        mut, 
        seeds = [b"poll".as_ref(), poll_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll: Account<'info, Poll>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(poll_index: u128)]
pub struct SetPollFlag<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pollStats".as_ref()], 
        bump)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        mut, 
        seeds = [b"poll".as_ref(), poll_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll: Account<'info, Poll>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(poll_index: u128)]
pub struct CreatePollOption<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pollStats".as_ref()], 
        bump)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        mut, 
        seeds = [b"poll".as_ref(), poll_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll: Account<'info, Poll>,

    #[account(
        init, 
        payer = signer, 
        seeds = [b"pollOption".as_ref(), poll_index.to_le_bytes().as_ref(), poll.option_count.to_le_bytes().as_ref(),], 
        bump, 
        space = size_of::<PollOption>() + POLL_AND_POLL_OPTION_EXTRA_SIZE + 8)]
    pub poll_option: Account<'info, PollOption>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(poll_index: u128, poll_option_index: u8)]
pub struct EditPollOption<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pollStats".as_ref()], 
        bump)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        mut, 
        seeds = [b"poll".as_ref(), poll_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll: Account<'info, Poll>,

    #[account(
        mut, 
        seeds = [b"pollOption".as_ref(), poll_index.to_le_bytes().as_ref(), poll_option_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll_option: Account<'info, PollOption>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(poll_index: u128, poll_option_index: u8)]
pub struct SetPollOptionFlag<'info> 
{
    #[account(
        seeds = [b"chatProtocolCEO".as_ref()],
        bump)]
    pub ceo: Account<'info, ChatProtocolCEO>,

    #[account(
        mut, 
        seeds = [b"pollStats".as_ref()], 
        bump)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        mut, 
        seeds = [b"poll".as_ref(), poll_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll: Account<'info, Poll>,

    #[account(
        mut,
        seeds = [b"pollOption".as_ref(), poll_index.to_le_bytes().as_ref(), poll_option_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll_option: Account<'info, PollOption>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(poll_index: u128, poll_option_index: u8, token_mint_address: Pubkey)]
pub struct VotePollOption<'info> 
{
    #[account(
        mut,
        seeds = [b"chatAccount".as_ref(), signer.key().as_ref()], 
        bump)]
    pub chat_account: Account<'info, ChatAccount>,

    #[account(
        mut, 
        seeds = [b"pollStats".as_ref()], 
        bump)]
    pub poll_stats: Account<'info, PollStats>,

    #[account(
        mut, 
        seeds = [b"pollVoteStats".as_ref()], 
        bump)]
    pub poll_vote_stats: Account<'info, PollVoteStats>,

    #[account(
        mut, 
        seeds = [b"poll".as_ref(), poll_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll: Account<'info, Poll>,

    #[account(
        mut, 
        seeds = [b"pollOption".as_ref(), poll_index.to_le_bytes().as_ref(), poll_option_index.to_le_bytes().as_ref()], 
        bump)]
    pub poll_option: Account<'info, PollOption>,

    #[account(
        init, 
        payer = signer,
        seeds = [b"pollVoteRecord".as_ref(), poll_index.to_le_bytes().as_ref(), poll_option_index.to_le_bytes().as_ref(), signer.key().as_ref(), chat_account.poll_vote_count.to_le_bytes().as_ref()], 
        bump, 
        space = size_of::<PollVoteRecord>() + 8)]
    pub poll_vote_record: Account<'info, PollVoteRecord>,

    #[account(
        seeds = [b"chatProtocolTreasurer".as_ref()],
        bump)]
    pub treasurer: Account<'info, ChatProtocolTreasurer>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = signer
    )]
    pub user_fee_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = fee_token_entry.token_mint_address,
        associated_token::authority = treasurer.address
    )]
    pub treasurer_fee_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"feeTokenEntry".as_ref(),
        token_mint_address.key().as_ref()], 
        bump)]
    pub fee_token_entry: Account<'info, FeeTokenEntry>,

    pub token_program: Program<'info, Token>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>
}

//Accounts
#[account]
pub struct ChatProtocolCEO
{
    pub address: Pubkey
}

#[account]
pub struct ChatProtocolTreasurer
{
    pub address: Pubkey
}

#[account]
pub struct FeeTokenEntry
{
    pub token_mint_address: Pubkey,
    pub decimal_amount: u8
}

#[account]
pub struct DeadMansBreak
{
    pub unix_clock_in_time_stamp: u64
}

#[account]
pub struct ChatProtocol
{
    pub chat_protocol_initiator_address: Pubkey,
    pub comment_and_reply_count: u128
}

#[account]
pub struct M4AChat
{
    pub chat_initiator_address: Pubkey,
    pub comment_up_vote_count: u128,
    pub comment_down_vote_count: u128,
    pub reply_up_vote_count: u128,
    pub reply_down_vote_count: u128,
    pub reply_lv3_up_vote_count: u128,
    pub reply_lv3_down_vote_count: u128,
    pub reply_lv4_up_vote_count: u128,
    pub reply_lv4_down_vote_count: u128,
    pub comment_count: u128,
    pub reply_count: u128,
    pub reply_lv3_count: u128,
    pub reply_lv4_count: u128,
    pub edited_comment_count: u128,
    pub edited_reply_count: u128,
    pub edited_lv3_reply_count: u128,
    pub edited_lv4_reply_count: u128,
    pub deleted_comment_count: u128,
    pub deleted_reply_count: u128,
    pub deleted_lv3_reply_count: u128,
    pub deleted_lv4_reply_count: u128,
    pub ceo_starred_comment_count: u128,
    pub ceo_starred_reply_count: u128,
    pub ceo_starred_lv3_reply_count: u128,
    pub ceo_starred_lv4_reply_count: u128,
    pub ceo_marked_fed_comment_count: u128,
    pub ceo_marked_fed_reply_count: u128,
    pub ceo_marked_fed_lv3_reply_count: u128,
    pub ceo_marked_fed_lv4_reply_count: u128
}

#[account]
pub struct PLIChat
{
    pub chat_initiator_address: Pubkey,
    pub comment_up_vote_count: u128,
    pub comment_down_vote_count: u128,
    pub reply_up_vote_count: u128,
    pub reply_down_vote_count: u128,
    pub reply_lv3_up_vote_count: u128,
    pub reply_lv3_down_vote_count: u128,
    pub reply_lv4_up_vote_count: u128,
    pub reply_lv4_down_vote_count: u128,
    pub comment_count: u128,
    pub reply_count: u128,
    pub reply_lv3_count: u128,
    pub reply_lv4_count: u128,
    pub edited_comment_count: u128,
    pub edited_reply_count: u128,
    pub edited_lv3_reply_count: u128,
    pub edited_lv4_reply_count: u128,
    pub deleted_comment_count: u128,
    pub deleted_reply_count: u128,
    pub deleted_lv3_reply_count: u128,
    pub deleted_lv4_reply_count: u128,
    pub ceo_starred_comment_count: u128,
    pub ceo_starred_reply_count: u128,
    pub ceo_starred_lv3_reply_count: u128,
    pub ceo_starred_lv4_reply_count: u128,
    pub ceo_marked_fed_comment_count: u128,
    pub ceo_marked_fed_reply_count: u128,
    pub ceo_marked_fed_lv3_reply_count: u128,
    pub ceo_marked_fed_lv4_reply_count: u128
}

#[account]
pub struct AboutChat
{
    pub chat_initiator_address: Pubkey,
    pub comment_up_vote_count: u128,
    pub comment_down_vote_count: u128,
    pub reply_up_vote_count: u128,
    pub reply_down_vote_count: u128,
    pub reply_lv3_up_vote_count: u128,
    pub reply_lv3_down_vote_count: u128,
    pub reply_lv4_up_vote_count: u128,
    pub reply_lv4_down_vote_count: u128,
    pub comment_count: u128,
    pub reply_count: u128,
    pub reply_lv3_count: u128,
    pub reply_lv4_count: u128,
    pub edited_comment_count: u128,
    pub edited_reply_count: u128,
    pub edited_lv3_reply_count: u128,
    pub edited_lv4_reply_count: u128,
    pub deleted_comment_count: u128,
    pub deleted_reply_count: u128,
    pub deleted_lv3_reply_count: u128,
    pub deleted_lv4_reply_count: u128,
    pub ceo_starred_comment_count: u128,
    pub ceo_starred_reply_count: u128,
    pub ceo_starred_lv3_reply_count: u128,
    pub ceo_starred_lv4_reply_count: u128,
    pub ceo_marked_fed_comment_count: u128,
    pub ceo_marked_fed_reply_count: u128,
    pub ceo_marked_fed_lv3_reply_count: u128,
    pub ceo_marked_fed_lv4_reply_count: u128
}

#[account]
pub struct LOChat
{
    pub chat_initiator_address: Pubkey,
    pub comment_up_vote_count: u128,
    pub comment_down_vote_count: u128,
    pub reply_up_vote_count: u128,
    pub reply_down_vote_count: u128,
    pub reply_lv3_up_vote_count: u128,
    pub reply_lv3_down_vote_count: u128,
    pub reply_lv4_up_vote_count: u128,
    pub reply_lv4_down_vote_count: u128,
    pub comment_count: u128,
    pub reply_count: u128,
    pub reply_lv3_count: u128,
    pub reply_lv4_count: u128,
    pub edited_comment_count: u128,
    pub edited_reply_count: u128,
    pub edited_lv3_reply_count: u128,
    pub edited_lv4_reply_count: u128,
    pub deleted_comment_count: u128,
    pub deleted_reply_count: u128,
    pub deleted_lv3_reply_count: u128,
    pub deleted_lv4_reply_count: u128,
    pub ceo_starred_comment_count: u128,
    pub ceo_starred_reply_count: u128,
    pub ceo_starred_lv3_reply_count: u128,
    pub ceo_starred_lv4_reply_count: u128,
    pub ceo_marked_fed_comment_count: u128,
    pub ceo_marked_fed_reply_count: u128,
    pub ceo_marked_fed_lv3_reply_count: u128,
    pub ceo_marked_fed_lv4_reply_count: u128
}

//Use for helping to know when to refetch all of the chat accounts
#[account]
pub struct ChatAccountStats
{
    pub chat_account_count: u64,
    pub set_flag_count: u64,
    pub updated_name_count: u64
}

#[account]
pub struct ChatAccount
{
    pub id: u64,
    pub user_address: Pubkey,
    pub user_name: String,
    pub use_custom_name: bool,
    pub has_had_custom_name: bool,
    pub has_good_ending: bool,
    pub poll_vote_count: u128,
    pub video_vote_count: u128,
    pub post_vote_casted_count: u128, //This is needed for deriving the account for the vote record. Yes it is just the net sum of up_vote_casted_count and down_vote_casted_count. Couldn't get them to add together in the seeds Q_Q
    pub received_up_vote_score: u128,
    pub received_down_vote_score: u128,
    pub casted_up_vote_score: u128,
    pub casted_down_vote_score: u128,
    pub up_vote_received_count: u128,
    pub down_vote_received_count: u128,
    pub up_vote_casted_count: u128,
    pub down_vote_casted_count: u128,
    pub comment_and_reply_count: u128,
    pub edited_comment_and_reply_count: u128,
    pub deleted_comment_and_reply_count: u128,
    pub ceo_starred_comment_and_reply_count: u128,
    pub ceo_marked_fed_comment_and_reply_count: u128
}

#[account]
pub struct CommentSectionStats
{
    pub comment_section_count: u128,
    pub video_up_vote_count: u128,
    pub video_down_vote_count: u128,
    pub toggle_flag_count: u32
}

#[account]
pub struct CommentSection
{
    pub id: u128,
    pub is_disabled: bool,
    pub comment_section_initiator_address: Pubkey,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub video_up_vote_score: u128,
    pub video_down_vote_score: u128,
    pub video_up_vote_count: u128,
    pub video_down_vote_count: u128,
    pub post_up_vote_score: u128,
    pub post_down_vote_score: u128,
    pub post_up_vote_count: u128,
    pub post_down_vote_count: u128,
    pub comment_up_vote_score: u128,
    pub comment_down_vote_score: u128,
    pub comment_up_vote_count: u128,
    pub comment_down_vote_count: u128,
    pub reply_up_vote_score: u128,
    pub reply_down_vote_score: u128,
    pub reply_up_vote_count: u128,
    pub reply_down_vote_count: u128,
    pub reply_to_reply_up_vote_score: u128,
    pub reply_to_reply_down_vote_score: u128,
    pub reply_lv3_up_vote_count: u128,
    pub reply_lv3_down_vote_count: u128,
    pub reply_to_lv3_reply_up_vote_score: u128,
    pub reply_to_lv3_reply_down_vote_score: u128,
    pub reply_lv4_up_vote_count: u128,
    pub reply_lv4_down_vote_count: u128,
    pub comment_and_reply_count: u128,
    pub comment_count: u128,
    pub reply_count: u128,
    pub reply_lv3_count: u128,
    pub reply_lv4_count: u128,
    pub edited_comment_count: u128,
    pub edited_reply_count: u128,
    pub edited_lv3_reply_count: u128,
    pub edited_lv4_reply_count: u128,
    pub deleted_comment_count: u128,
    pub deleted_reply_count: u128,
    pub deleted_lv3_reply_count: u128,
    pub deleted_lv4_reply_count: u128,
    pub ceo_starred_comment_count: u128,
    pub ceo_starred_reply_count: u128,
    pub ceo_starred_lv3_reply_count: u128,
    pub ceo_starred_lv4_reply_count: u128,
    pub ceo_marked_fed_comment_count: u128,
    pub ceo_marked_fed_reply_count: u128,
    pub ceo_marked_fed_lv3_reply_count: u128,
    pub ceo_marked_fed_lv4_reply_count: u128
}

#[account]
pub struct M4AComment
{
    pub id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct M4AReply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct M4ALv3Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct M4ALv4Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct PLIComment
{
    pub id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct PLIReply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct PLILv3Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct PLILv4Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct AboutComment
{
    pub id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct AboutReply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct AboutLv3Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct AboutLv4Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct LOComment
{
    pub id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct LOReply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct LOLv3Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct LOLv4Reply
{
    pub id: u128,
    pub parent_id: u128,
    pub protocol_post_count: u128,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub msg: String,
    pub net_vote_score: i128,
    pub unix_creation_time_stamp: u64,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_starred: bool,
    pub is_fed: bool,
    pub reply_count: u32
}

#[account]
pub struct IdeaStats
{
    pub protocol_idea_count: u128,
    pub protocol_deleted_idea_count: u128,
    pub updated_idea_count: u64
}

#[account]
pub struct Idea
{
    pub id: u128,
    pub post_type: u8, //Needed to know which function to call on the front end
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub idea: String,
    pub unix_creation_time_stamp: u64,
    pub implementation_time: u64,
    pub is_implemented: bool,
    pub is_updated: bool
}

#[account]
pub struct FEDStats
{
    pub federal_agent_post_count: u128,
    pub deleted_federal_agent_post_count: u128
}

#[account]
pub struct FEDRecord
{
    pub id: u128,
    pub post_type: u8, //Needed to know which function to call on the front end
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub post_owner_address: Pubkey,
    pub chat_account_post_count_index: u128,
    pub post: String,
    pub mark_time: u64,
    pub was_edited_before_mark: bool
}

#[account]
pub struct VideoVoteStats
{
    pub video_up_vote_count: u128,
    pub video_down_vote_count: u128
}

#[account]
pub struct VideoVoteRecord
{
    pub protocol_record_id: u128,
    pub comment_section_record_id: u128,
    pub voter_address: Pubkey,
    pub comment_section_name_prefix: String, 
    pub comment_section_name: String,
    pub vote_amount: i128,
    pub unix_creation_time_stamp: u64
}

#[account]
pub struct PostVoteStats
{
    pub post_up_vote_count: u128,
    pub post_down_vote_count: u128
}

#[account]
pub struct PostVoteRecord
{
    pub id: u128,
    pub voter_address: Pubkey,
    pub canidate_address: Pubkey,
    pub vote_amount: i128,
    pub unix_creation_time_stamp: u64
}

#[account]
pub struct PollStats
{
    pub poll_count: u128,
    pub option_count: u128,
    pub edited_poll_or_poll_option_count: u128,
    pub up_vote_score: u128,
    pub down_vote_score: u128
}

#[account]
pub struct Poll
{
    pub is_active: bool,
    pub poll_index: u128,
    pub poll_name: String,
    pub up_vote_score: u128,
    pub down_vote_score: u128,
    pub up_vote_count: u128,
    pub down_vote_count: u128,
    pub unix_creation_time_stamp: u64,
    pub option_count: u8,
    pub edited_poll_option_count: u128
}

#[account]
pub struct PollOption
{
    pub is_active: bool,
    pub poll_option_index: u8,
    pub poll_option_name: String,
    pub up_vote_score: u128,
    pub down_vote_score: u128,
    pub up_vote_count: u128,
    pub down_vote_count: u128,
    pub unix_creation_time_stamp: u64,
}

#[account]
pub struct PollVoteStats
{
    pub up_vote_count: u128,
    pub down_vote_count: u128
}

#[account]
pub struct PollVoteRecord
{
    pub protocol_record_id: u128,
    pub poll_record_id: u128,
    pub poll_index: u128,
    pub poll_option_index: u8,
    pub voter_address: Pubkey,
    pub unix_creation_time_stamp: u64,
    pub vote_amount: i128
}