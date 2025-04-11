use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, pubkey::find_program_address, ProgramResult};

use crate::state::Escrow;

/// # Take Instruction
/// 
/// This function allows a taker to accept an existing escrow offer and complete the trade.
/// The taker sends their token Y and receives the maker's token X that was locked in the vault.
/// 
/// ## Business Logic:
/// 1. Taker verifies the escrow details and agrees to the trade
/// 2. Taker sends the requested amount of token Y to the maker
/// 3. Taker receives the offered token X from the vault
/// 4. The escrow and vault accounts are closed, returning rent to the maker
/// 
/// ## Accounts expected:
/// 0. `[signer]` taker - The account accepting the escrow trade
/// 1. `[]` maker - The original creator of the escrow
/// 2. `[]` mint_x - The mint of the token being offered by the maker
/// 3. `[]` mint_y - The mint of the token requested by the maker
/// 4. `[mut]` taker_ata_x - Taker's associated token account for mint_x
/// 5. `[mut]` taker_ata_y - Taker's associated token account for mint_y
/// 6. `[mut]` maker_ata_y - Maker's associated token account for mint_y
/// 7. `[mut]` vault - Token account holding the locked tokens from the maker
/// 8. `[mut]` escrow - Account storing the escrow state data
/// 9. `[]` token_program - SPL Token program for token operations
/// 10. `[]` system_program - System program
pub fn process_take_instruction(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    // Unpack all required accounts for the take operation
    let [
        taker, maker, mint_x, mint_y, taker_ata_x, taker_ata_y, maker_ata_y, vault, escrow,
        _token_program, _system_program, _remaining @ ..
    ] = accounts else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };
    
    // Access the escrow data to verify trade parameters
    let escrow_account = Escrow::from_account_info_unchecked(escrow);
    
    // Verify that the provided token mints match what's stored in the escrow
    // This prevents trading with incorrect tokens
    assert_eq!(escrow_account.mint_x, *mint_x.key());
    assert_eq!(escrow_account.mint_y, *mint_y.key());

    // Load the vault account to access its token balance
    let vault_account = pinocchio_token::state::TokenAccount::from_account_info(vault)?;

    // Verify the escrow account is a valid PDA with the expected seeds
    // This ensures we're operating on a legitimate escrow created by our program
    let seed = [(b"escrow"), maker.key().as_slice(), &[escrow_account.bump]];
    let seeds = &seed[..];
    let escrow_pda = find_program_address(seeds, &crate::ID).0;
    assert_eq!(*escrow.key(), escrow_pda);

    // First leg of the trade: Taker sends tokens to the maker
    // The taker pays the requested amount of token Y directly to the maker
    pinocchio_token::instructions::Transfer {
        from: taker_ata_y,
        to: maker_ata_y,
        authority: taker,
        amount: escrow_account.amount,
    }.invoke()?;

    // Prepare the PDA signer seeds for the escrow
    // This allows the program to sign for operations on behalf of the escrow PDA
    let bump = [escrow_account.bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.key()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);

    // Second leg of the trade: Send tokens from vault to taker
    // The escrow PDA signs to release the tokens to the taker
    pinocchio_token::instructions::Transfer {
        from: vault,
        to: taker_ata_x,
        authority: escrow,
        amount: vault_account.amount(),
    }.invoke_signed(&[seeds.clone()])?;

    // Close the vault account and return the rent to the maker
    // The maker paid to create this account, so they receive the lamports
    pinocchio_token::instructions::CloseAccount {
        account: vault,
        destination: maker,
        authority: escrow,
    }.invoke_signed(&[seeds])?;

    // Manually close the escrow account and return rent to the maker
    // This completes the trade by cleaning up all accounts
    unsafe { 
        *maker.borrow_mut_lamports_unchecked() += *escrow.borrow_lamports_unchecked();
        *escrow.borrow_mut_lamports_unchecked() = 0 
    };

    // Trade successfully completed
    Ok(())
}