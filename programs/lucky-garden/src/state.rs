use pinocchio::{cpi::Seed, error::ProgramError, Address};

pub const MAX_PRIZES: usize = 8;
pub const NAME_LEN: usize = 16;

pub struct Config;
impl Config {
    pub const SEED: &'static [u8] = b"config";
    pub const SIZE: usize = 2 + 32 + 1 + MAX_PRIZES * 2 + MAX_PRIZES * NAME_LEN;
    pub fn find(program_id: &Address) -> (Address, u8) {
        Address::find_program_address(&[Self::SEED], program_id)
    }
    pub fn signer_seeds(bump: &[u8]) -> [Seed<'_>; 2] {
        [Seed::from(Self::SEED), Seed::from(bump)]
    }
}

pub struct Player;
impl Player {
    pub const SEED: &'static [u8] = b"player";
    pub const SIZE: usize = 32; // version,bump,status,game,prize,reserved(3),day(i64),nonce(u64),requested_slot(u64)
    pub fn find(user: &Address, program_id: &Address) -> (Address, u8) {
        Address::find_program_address(&[Self::SEED, user.as_ref()], program_id)
    }
    pub fn signer_seeds<'a>(user: &'a Address, bump: &'a [u8]) -> [Seed<'a>; 3] {
        [
            Seed::from(Self::SEED),
            Seed::from(user.as_ref()),
            Seed::from(bump),
        ]
    }
}

pub fn require_len(data: &[u8], size: usize) -> Result<(), ProgramError> {
    if data.len() != size {
        Err(ProgramError::InvalidAccountData)
    } else {
        Ok(())
    }
}
pub fn read_i64(data: &[u8], at: usize) -> Result<i64, ProgramError> {
    Ok(i64::from_le_bytes(
        data.get(at..at + 8)
            .ok_or(ProgramError::InvalidAccountData)?
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    ))
}
pub fn read_u64(data: &[u8], at: usize) -> Result<u64, ProgramError> {
    Ok(u64::from_le_bytes(
        data.get(at..at + 8)
            .ok_or(ProgramError::InvalidAccountData)?
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    ))
}
