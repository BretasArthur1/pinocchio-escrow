use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Escrow {
    pub maker: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl Escrow {
    pub const SIZE: usize = 32 + 32 + 32 + 8 + 1;

    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> &mut Self {
        unsafe { &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> &mut Self {
        unsafe {
            assert_eq!(account_info.data_len(), Escrow::SIZE);
            assert_eq!(account_info.owner(), &crate::ID);
            &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self)
        }
    }
}