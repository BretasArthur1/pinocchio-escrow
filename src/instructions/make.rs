use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_log::log;
use pinocchio_token::state::TokenAccount;

use crate::state::Escrow;

/// # Make Instruction
/// 
/// This function creates a new escrow for token exchange between two parties.
/// The maker creates an offer to trade their token X for someone else's token Y.
/// 
/// ## Business Logic:
/// 1. Maker offers token X in exchange for token Y
/// 2. Tokens are locked in a vault until someone takes the trade
/// 3. Escrow account stores the trade parameters and ownership information
/// 
/// ## Accounts expected:
/// 0. `[signer]` maker - The account initiating the escrow trade
/// 1. `[]` mint_x - The mint of the token being offered
/// 2. `[]` mint_y - The mint of the token requested in exchange
/// 3. `[mut]` maker_ata - Maker's associated token account for mint_x
/// 4. `[mut]` vault - Token account to temporarily hold the offered tokens
/// 5. `[mut]` escrow - Account to store the escrow state data
/// 6. `[]` system_program - System program for account creation
/// 7. `[]` token_program - SPL Token program for token operations
///
/// ## Data parameters:
/// 0. [u8; 1] - Bump seed for PDA derivation
/// 1. [u64; 1] - Amount of token_y the maker wants to receive
/// 9. [u64; 1] - Amount of token_x the maker is offering
pub fn process_make_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Unpack the required accounts from the accounts array
    let [maker, mint_x, mint_y, maker_ata, vault, escrow, _system_program, _token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    // Extract the bump seed from instruction data and prepare seeds for PDA validation
    let bump = unsafe{ *(data.as_ptr() as *const u8) }.to_le_bytes();
    let seed = [(b"escrow"), maker.key().as_slice(), bump.as_ref()];
    let seeds = &seed[..];

    // Derive the expected PDA and verify it matches the provided escrow account
    // This ensures the escrow account is derived correctly for this maker and trade
    let pda = pubkey::checked_create_program_address(seeds, &crate::ID).unwrap();
    assert_eq!(&pda, escrow.key());

    unsafe {
        // Verify that the provided mint accounts are legitimate SPL token mints
        assert_eq!(mint_x.owner(), &pinocchio_token::ID);
        assert_eq!(mint_y.owner(), &pinocchio_token::ID);
        
        // Verify that the vault is owned by the escrow account (for later token operations)
        assert!(TokenAccount::from_account_info_unchecked(vault).unwrap().owner() == escrow.key());

        // Check if the escrow account needs to be created (first-time initialization)
        if escrow.owner() != &crate::ID {
            log!("Creating Escrow Account");

            // Create the escrow account with enough space for the state data
            // This account will store all the relevant trade details
            pinocchio_system::instructions::CreateAccount {
                from: maker,
                to: escrow,
                lamports: Rent::get()?.minimum_balance(Escrow::SIZE),
                space: Escrow::SIZE as u64,
                owner: &crate::ID,
            }.invoke()?;

            // Initialize the escrow data with the trade parameters
            let escrow_account = Escrow::from_account_info_unchecked(&escrow);
            escrow_account.maker = *maker.key();
            escrow_account.mint_x = *mint_x.key();
            escrow_account.mint_y = *mint_y.key();
            escrow_account.amount = *(data.as_ptr().add(1) as *const u64); // Amount of token Y to receive
            escrow_account.bump = *data.as_ptr(); // Store bump for future PDA derivation

            log!("Amount to deposit: {}", *(data.as_ptr().add(1 + 8) as *const u64));

            // Transfer the offered tokens from maker's account to the vault
            // These tokens will be locked until someone takes the trade or the maker refunds
            pinocchio_token::instructions::Transfer {
                from: maker_ata,
                to: vault,
                authority: maker,
                amount: *(data.as_ptr().add(1 + 8) as *const u64), // Amount of token X to deposit
            }.invoke()?;
        }
        else {
            // If the escrow account already exists, prevent duplicate initialization
            return Err(ProgramError::AccountAlreadyInitialized)
        }
    }
    
    // Escrow successfully created and tokens deposited
    Ok(())
}