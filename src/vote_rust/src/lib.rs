use byteorder::{ByteOrder, LittleEndian};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use std::mem;

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
fn process_instruction(
    program_id: &Pubkey,      // Public key of program account
    accounts: &[AccountInfo], // data accounts
    instruction_data: &[u8],  // 1 = vote for A, 2 = vote for B
) -> ProgramResult {
    msg!("Rust program entrypoint");

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account that holds the vote count
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Vote account is not owned by the program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // The data must be large enough to hold two u32 vote counts
    // in the next (slightly more complicated) version of the
    // program we will use solana_sdk::program_pack::Pack
    // to retrieve and deserialise the account data
    // and to check it is the correct length
    // for now, realise it's literally just 8 bytes of data.

    if account.try_data_len()? < 2 * mem::size_of::<u32>() {
        msg!("Vote account data length too small for u32");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut data = account.try_borrow_mut_data()?;

    if 1 == instruction_data[0] {
        // the first 4 bytes are a u32 (unsigned integer) in little endian format
        // holding the number of votes for candidate 1

        // we read the data from the account into the u32 variable vc
        let vc = LittleEndian::read_u32(&data[0..4]);

        // increment by 1

        // write the u32 number back to the first 4 bytes
        LittleEndian::write_u32(&mut data[0..4], vc + 1);

        msg!("Voted for 1");
    }

    if 2 == instruction_data[0] {
        let vc = LittleEndian::read_u32(&data[4..8]);
        LittleEndian::write_u32(&mut data[4..8], vc + 1);
        msg!("Voted for 2");
    }

    Ok(())
}

// tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;

    #[test]
    fn test_sanity() {
        // mock program id

        let program_id = Pubkey::default();

        // mock accounts array...

        let key = Pubkey::default(); // anything
        let mut lamports = 0;

        let mut data = vec![0; 2 * mem::size_of::<u32>()];
        LittleEndian::write_u32(&mut data[0..4], 0); // set storage to zero
        LittleEndian::write_u32(&mut data[4..8], 0);

        let owner = Pubkey::default();

        let account = AccountInfo::new(
            &key,             // account pubkey
            false,            // is_signer
            true,             // is_writable
            &mut lamports,    // balance in lamports
            &mut data,        // storage
            &owner,           // owner pubkey
            false,            // is_executable
            Epoch::default(), // rent_epoch
        );

        let mut instruction_data: Vec<u8> = vec![0];

        let accounts = vec![account];

        assert_eq!(LittleEndian::read_u32(&accounts[0].data.borrow()[0..4]), 0);
        assert_eq!(LittleEndian::read_u32(&accounts[0].data.borrow()[4..8]), 0);

        // vote for candidate 1

        instruction_data[0] = 1;
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(LittleEndian::read_u32(&accounts[0].data.borrow()[0..4]), 1);
        assert_eq!(LittleEndian::read_u32(&accounts[0].data.borrow()[4..8]), 0);

        // vote for candidate 2

        instruction_data[0] = 2;
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(LittleEndian::read_u32(&accounts[0].data.borrow()[0..4]), 1);
        assert_eq!(LittleEndian::read_u32(&accounts[0].data.borrow()[4..8]), 1);
    }
}
