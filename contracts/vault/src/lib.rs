use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol};

/// Maximum length (in bytes) allowed for a vault note.
pub const MAX_NOTE_LENGTH: u32 = 256;

/// Maximum length (in bytes) allowed for a vault tag.
pub const MAX_TAG_LENGTH: u32 = 64;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultEntry {
    pub owner: Address,
    pub note: String,
    pub tag: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultError {
    NoteTooLong = 1,
    TagTooLong = 2,
}

const ENTRY_KEY: Symbol = symbol_short!("ENTRY");

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    /// Store a vault entry, rejecting notes/tags that exceed the bounded lengths.
    pub fn set_entry(env: Env, owner: Address, note: String, tag: String) -> Result<(), VaultError> {
        owner.require_auth();

        if note.len() > MAX_NOTE_LENGTH {
            return Err(VaultError::NoteTooLong);
        }
        if tag.len() > MAX_TAG_LENGTH {
            return Err(VaultError::TagTooLong);
        }

        let entry = VaultEntry { owner, note, tag };
        env.storage().persistent().set(&ENTRY_KEY, &entry);
        Ok(())
    }

    /// Retrieve the stored vault entry, if any.
    pub fn get_entry(env: Env) -> Option<VaultEntry> {
        env.storage().persistent().get(&ENTRY_KEY)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    fn setup() -> (Env, VaultContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, VaultContract);
        let client = VaultContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        (env, client, owner)
    }

    #[test]
    fn accepts_notes_and_tags_at_the_boundary() {
        let (env, client, owner) = setup();
        let note = String::from_str(&env, &"n".repeat(MAX_NOTE_LENGTH as usize));
        let tag = String::from_str(&env, &"t".repeat(MAX_TAG_LENGTH as usize));

        assert_eq!(client.set_entry(&owner, &note, &tag), Ok(()));

        let entry = client.get_entry().unwrap();
        assert_eq!(entry.note, note);
        assert_eq!(entry.tag, tag);
    }

    #[test]
    fn rejects_oversized_note() {
        let (env, client, owner) = setup();
        let note = String::from_str(&env, &"n".repeat(MAX_NOTE_LENGTH as usize + 1));
        let tag = String::from_str(&env, "ok");

        assert_eq!(client.set_entry(&owner, &note, &tag), Err(VaultError::NoteTooLong));
        assert!(client.get_entry().is_none());
    }

    #[test]
    fn rejects_oversized_tag() {
        let (env, client, owner) = setup();
        let note = String::from_str(&env, "ok");
        let tag = String::from_str(&env, &"t".repeat(MAX_TAG_LENGTH as usize + 1));

        assert_eq!(client.set_entry(&owner, &note, &tag), Err(VaultError::TagTooLong));
        assert!(client.get_entry().is_none());
    }
}
