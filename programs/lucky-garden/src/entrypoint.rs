use crate::processor::*;
use core::{mem::MaybeUninit, slice::from_raw_parts};
use pinocchio::{
    entrypoint::deserialize, error::ProgramError, no_allocator, nostd_panic_handler, AccountView,
    Address, ProgramResult, MAX_TX_ACCOUNTS, SUCCESS,
};

#[derive(Copy, Clone)]
pub enum Tag {
    InitializeConfig,
    UpdateConfig,
    InitializePlayer,
    RequestDraw,
    CallbackDraw,
}
impl Tag {
    pub fn bytes(self) -> [u8; 8] {
        (self as u64).to_le_bytes()
    }
    fn parse(v: [u8; 8]) -> Result<Self, ProgramError> {
        match u64::from_le_bytes(v) {
            0 => Ok(Self::InitializeConfig),
            1 => Ok(Self::UpdateConfig),
            2 => Ok(Self::InitializePlayer),
            3 => Ok(Self::RequestDraw),
            4 => Ok(Self::CallbackDraw),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
no_allocator!();
nostd_panic_handler!();
#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    const U: MaybeUninit<AccountView> = MaybeUninit::uninit();
    let mut accounts = [U; MAX_TX_ACCOUNTS];
    let (pid, n, data) = deserialize::<MAX_TX_ACCOUNTS>(input, &mut accounts);
    match process_instruction(pid, from_raw_parts(accounts.as_ptr() as _, n), data) {
        Ok(()) => SUCCESS,
        Err(e) => e.into(),
    }
}
pub fn process_instruction(pid: &Address, accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let disc: [u8; 8] = data
        .get(..8)
        .ok_or(ProgramError::InvalidInstructionData)?
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    match Tag::parse(disc)? {
        Tag::InitializeConfig => initialize_config(pid, accounts, &data[8..]),
        Tag::UpdateConfig => update_config(pid, accounts, &data[8..]),
        Tag::InitializePlayer => initialize_player(pid, accounts),
        Tag::RequestDraw => request_draw(pid, accounts, &data[8..]),
        Tag::CallbackDraw => callback_draw(pid, accounts, &data[8..]),
    }
}
