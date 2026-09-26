use crate::{
    entrypoint::Tag,
    state::{read_i64, read_u64, require_len, Config, Player, MAX_PRIZES, NAME_LEN},
};
use ephemeral_rollups_pinocchio::vrf::{
    program_identity_pda, scoped_vrf_identity, RequestRandomness, RequestRandomnessCpi,
    IDENTITY_SEED,
};
use pinocchio::{
    account::AccountView,
    cpi::{Seed, Signer},
    error::ProgramError,
    instruction::InstructionAccount,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

const DAY_SECONDS: i64 = 86_400;
const STATUS_PENDING: u8 = 1;
const STATUS_READY: u8 = 2;

fn signer(a: &AccountView) -> ProgramResult {
    if !a.is_signer() {
        Err(ProgramError::MissingRequiredSignature)
    } else {
        Ok(())
    }
}
fn owned(a: &AccountView, p: &Address) -> ProgramResult {
    if unsafe { a.owner() } != p {
        Err(ProgramError::IllegalOwner)
    } else {
        Ok(())
    }
}
fn parse_config_payload(payload: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
    let count = *payload
        .first()
        .ok_or(ProgramError::InvalidInstructionData)? as usize;
    if count == 0 || count > MAX_PRIZES || payload.len() != 1 + count * (2 + NAME_LEN) {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut total: u32 = 0;
    for i in 0..count {
        let at = 1 + i * (2 + NAME_LEN);
        total = total
            .checked_add(u16::from_le_bytes([payload[at], payload[at + 1]]) as u32)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }
    if total != 10_000 {
        return Err(ProgramError::InvalidInstructionData);
    }
    Ok((count as u8, payload))
}
fn write_config(data: &mut [u8], admin: &Address, payload: &[u8]) -> ProgramResult {
    let (count, p) = parse_config_payload(payload)?;
    data.fill(0);
    data[0] = 1;
    data[1..33].copy_from_slice(admin.as_ref());
    data[33] = count;
    for i in 0..count as usize {
        let src = 1 + i * (2 + NAME_LEN);
        let w = 34 + i * 2;
        data[w..w + 2].copy_from_slice(&p[src..src + 2]);
        let n = 34 + MAX_PRIZES * 2 + i * NAME_LEN;
        data[n..n + NAME_LEN].copy_from_slice(&p[src + 2..src + 2 + NAME_LEN]);
    }
    Ok(())
}
pub fn initialize_config(pid: &Address, a: &[AccountView], payload: &[u8]) -> ProgramResult {
    let [payer, config, _] = a else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer(payer)?;
    let pda = Config::find(pid);
    if config.address() != &pda.0 {
        return Err(ProgramError::InvalidSeeds);
    }
    if config.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let bump = [pda.1];
    CreateAccount {
        from: payer,
        to: config,
        lamports: Rent::get()?.try_minimum_balance(Config::SIZE)?,
        space: Config::SIZE as u64,
        owner: pid,
    }
    .invoke_signed(&[Signer::from(&Config::signer_seeds(&bump))])?;
    let mut d = config.try_borrow_mut()?;
    require_len(&d, Config::SIZE)?;
    write_config(&mut d, payer.address(), payload)
}
pub fn update_config(pid: &Address, a: &[AccountView], payload: &[u8]) -> ProgramResult {
    let [admin, config] = a else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer(admin)?;
    owned(config, pid)?;
    if config.address() != &Config::find(pid).0 {
        return Err(ProgramError::InvalidSeeds);
    }
    let mut d = config.try_borrow_mut()?;
    require_len(&d, Config::SIZE)?;
    if &d[1..33] != admin.address().as_ref() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    write_config(&mut d, admin.address(), payload)
}
pub fn initialize_player(pid: &Address, a: &[AccountView]) -> ProgramResult {
    let [payer, player, _] = a else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer(payer)?;
    let pda = Player::find(payer.address(), pid);
    if player.address() != &pda.0 {
        return Err(ProgramError::InvalidSeeds);
    }
    if player.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let bump = [pda.1];
    CreateAccount {
        from: payer,
        to: player,
        lamports: Rent::get()?.try_minimum_balance(Player::SIZE)?,
        space: Player::SIZE as u64,
        owner: pid,
    }
    .invoke_signed(&[Signer::from(&Player::signer_seeds(payer.address(), &bump))])?;
    let mut d = player.try_borrow_mut()?;
    d.fill(0);
    d[0] = 1;
    d[1] = pda.1;
    Ok(())
}
pub fn request_draw(pid: &Address, a: &[AccountView], payload: &[u8]) -> ProgramResult {
    let [payer, player, config, oracle, identity, vrf, slot_hashes, system] = a else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer(payer)?;
    if payload.len() != 2 || payload[0] > 1 {
        return Err(ProgramError::InvalidInstructionData);
    }
    owned(player, pid)?;
    owned(config, pid)?;
    if player.address() != &Player::find(payer.address(), pid).0
        || config.address() != &Config::find(pid).0
    {
        return Err(ProgramError::InvalidSeeds);
    }
    let day = Clock::get()?.unix_timestamp.div_euclid(DAY_SECONDS);
    {
        let mut d = player.try_borrow_mut()?;
        require_len(&d, Player::SIZE)?;
        if d[2] == STATUS_PENDING || read_i64(&d, 8)? == day {
            return Err(ProgramError::InvalidArgument);
        }
        d[2] = STATUS_PENDING;
        d[3] = payload[0];
        d[8..16].copy_from_slice(&day.to_le_bytes());
        let nonce = read_u64(&d, 16)?
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        d[16..24].copy_from_slice(&nonce.to_le_bytes());
        d[24..32].copy_from_slice(&Clock::get()?.slot.to_le_bytes());
    }
    let ip = program_identity_pda(pid);
    if identity.address() != &ip.0 {
        return Err(ProgramError::InvalidSeeds);
    }
    let bump = [ip.1];
    let seeds = [Seed::from(IDENTITY_SEED), Seed::from(&bump)];
    let mut buf = [0u8; RequestRandomness::serialized_size_for(8, 2, 1)];
    RequestRandomnessCpi {
        payer,
        oracle_queue: oracle,
        program_identity: identity,
        vrf_program: vrf,
        slot_hashes,
        system_program: system,
        request: RequestRandomness {
            high_priority: true,
            caller_seed: [payload[1]; 32],
            callback_discriminator: &Tag::CallbackDraw.bytes(),
            callback_args: &[payload[0]],
            callback_program_id: pid,
            callback_accounts_metas: &[
                InstructionAccount {
                    address: config.address(),
                    is_signer: false,
                    is_writable: false,
                },
                InstructionAccount {
                    address: player.address(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
        },
    }
    .invoke_signed(&mut buf, &[Signer::from(&seeds)])
}
pub fn callback_draw(pid: &Address, a: &[AccountView], payload: &[u8]) -> ProgramResult {
    let [identity, config, player] = a else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if identity.address() != &scoped_vrf_identity(pid).0 {
        return Err(ProgramError::InvalidSeeds);
    }
    signer(identity)?;
    owned(config, pid)?;
    owned(player, pid)?;
    if config.address() != &Config::find(pid).0 {
        return Err(ProgramError::InvalidSeeds);
    }
    if payload.len() != 33 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let random: [u8; 32] = payload[..32]
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    let cfg = config.try_borrow()?;
    require_len(&cfg, Config::SIZE)?;
    let count = cfg[33] as usize;
    if count == 0 || count > MAX_PRIZES {
        return Err(ProgramError::InvalidAccountData);
    }
    let roll = u16::from_le_bytes([random[0], random[1]]) % 10_000;
    let mut sum = 0u16;
    let mut prize = 0u8;
    for i in 0..count {
        let at = 34 + i * 2;
        sum = sum
            .checked_add(u16::from_le_bytes([cfg[at], cfg[at + 1]]))
            .ok_or(ProgramError::ArithmeticOverflow)?;
        if roll < sum {
            prize = i as u8;
            break;
        }
    }
    let mut d = player.try_borrow_mut()?;
    require_len(&d, Player::SIZE)?;
    if d[2] != STATUS_PENDING || d[3] != payload[32] {
        return Err(ProgramError::InvalidAccountData);
    }
    d[2] = STATUS_READY;
    d[4] = prize;
    Ok(())
}
