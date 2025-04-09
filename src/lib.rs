use instructions::EscrowInstructions;
use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

mod instructions;
mod state;


entrypoint!(process_instruction);

const ID: Pubkey = five8_const::decode_32_const("77777777777777777777777777777777777777777777");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data.split_first().ok_or(ProgramError::InvalidAccountData)?;

    match EscrowInstructions::try_from(*discriminator)? {
        EscrowInstructions::Make => instructions::process_make_instruction(accounts, data)?,
        EscrowInstructions::Take => instructions::process_take_instruction(accounts, data)?,
        EscrowInstructions::Refund => instructions::process_refund_instruction(accounts, data)?,
    }
    Ok(())
}

