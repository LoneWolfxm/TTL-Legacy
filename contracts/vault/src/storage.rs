use soroban_sdk::{contracttype, Address, Env, Symbol};

/// Number of ledgers in a day, assuming ~5s close time.
pub const LEDGERS_PER_DAY: u32 = 17_280;

/// Approximate number of bytes a single vault entry occupies in persistent
/// storage. This is a conservative estimate of the serialized footprint of a
/// `Vault` record (id, owner, balance, timestamps, and metadata).
///
/// NOTE: Soroban charges rent per byte of persistent storage per ledger. The
/// exact footprint depends on the serialized XDR size of the stored value; this
/// constant is a documented approximation used for estimation only.
pub const VAULT_STORAGE_BYTES: u64 = 256;

/// Rent (in stroops) charged per byte of persistent storage per ledger.
///
/// This mirrors the network's storage rent rate. It is expressed here as a
/// constant so the estimation formula is explicit and testable. If the network
/// rate changes, update this value.
pub const RENT_PER_BYTE_PER_LEDGER: u64 = 1;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vault {
    pub id: u64,
    pub owner: Address,
    pub balance: i128,
    pub created_at: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RentEstimate {
    /// Number of ledgers the estimate covers.
    pub ledgers: u32,
    /// Estimated storage footprint in bytes.
    pub storage_bytes: u64,
    /// Estimated total rent in stroops for the requested duration.
    pub total_rent: u64,
}

/// Storage key namespace for vault records.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Vault(u64),
}

/// Estimate the storage rent required to keep a vault alive for `ledgers`
/// ledgers.
///
/// # Calculation
///
/// ```text
/// total_rent = storage_bytes * RENT_PER_BYTE_PER_LEDGER * ledgers
/// ```
///
/// * `storage_bytes` is the estimated persistent footprint of the vault
///   (`VAULT_STORAGE_BYTES`).
/// * `RENT_PER_BYTE_PER_LEDGER` is the network rent rate in stroops.
/// * `ledgers` is the requested lifetime in ledgers (use `LEDGERS_PER_DAY` to
///   convert days to ledgers).
///
/// # Inputs
/// * `vault_id` - identifier of the vault to estimate rent for. The vault must
///   exist; otherwise this returns `None`.
/// * `ledgers` - number of ledgers to keep the vault alive. `0` yields a zero
///   estimate.
///
/// # Units
/// * Result is in stroops (1 XLM = 10_000_000 stroops).
///
/// # Assumptions
/// * The vault's storage footprint is constant for the requested duration.
/// * The rent rate is constant; it does not model network rate changes.
/// * Arithmetic saturates at `u64::MAX` for very large inputs rather than
///   panicking.
pub fn estimate_rent(env: &Env, vault_id: u64, ledgers: u32) -> Option<RentEstimate> {
    let key = DataKey::Vault(vault_id);
    if !env.storage().persistent().has(&key) {
        return None;
    }

    let storage_bytes = VAULT_STORAGE_BYTES;
    let total_rent = storage_bytes
        .saturating_mul(RENT_PER_BYTE_PER_LEDGER)
        .saturating_mul(ledgers as u64);

    Some(RentEstimate {
        ledgers,
        storage_bytes,
        total_rent,
    })
}

/// Convenience helper: estimate rent for a number of days.
pub fn estimate_rent_days(env: &Env, vault_id: u64, days: u32) -> Option<RentEstimate> {
    let ledgers = days.saturating_mul(LEDGERS_PER_DAY);
    estimate_rent(env, vault_id, ledgers)
}

/// Persist a vault record. Kept here so the storage module owns the key layout
/// used by `estimate_rent`.
pub fn save_vault(env: &Env, vault: &Vault) {
    let key = DataKey::Vault(vault.id);
    env.storage().persistent().set(&key, vault);
}

/// Load a vault record, if present.
pub fn load_vault(env: &Env, vault_id: u64) -> Option<Vault> {
    let key = DataKey::Vault(vault_id);
    env.storage().persistent().get(&key)
}

/// Remove a vault record.
pub fn remove_vault(env: &Env, vault_id: u64) {
    let key = DataKey::Vault(vault_id);
    env.storage().persistent().remove(&key);
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup_vault(env: &Env, id: u64) -> Vault {
        let owner = Address::generate(env);
        let vault = Vault {
            id,
            owner,
            balance: 0,
            created_at: 0,
            last_updated: 0,
        };
        save_vault(env, &vault);
        vault
    }

    #[test]
    fn estimate_rent_zero_ledgers_is_zero() {
        let env = Env::default();
        setup_vault(&env, 1);
        let est = estimate_rent(&env, 1, 0).unwrap();
        assert_eq!(est.ledgers, 0);
        assert_eq!(est.total_rent, 0);
    }

    #[test]
    fn estimate_rent_missing_vault_returns_none() {
        let env = Env::default();
        assert!(estimate_rent(&env, 42, LEDGERS_PER_DAY).is_none());
    }

    #[test]
    fn estimate_rent_typical_duration() {
        let env = Env::default();
        setup_vault(&env, 7);
        let est = estimate_rent(&env, 7, LEDGERS_PER_DAY).unwrap();
        assert_eq!(est.storage_bytes, VAULT_STORAGE_BYTES);
        assert_eq!(
            est.total_rent,
            VAULT_STORAGE_BYTES * RENT_PER_BYTE_PER_LEDGER * LEDGERS_PER_DAY as u64
        );
    }

    #[test]
    fn estimate_rent_very_large_ledgers_saturates() {
        let env = Env::default();
        setup_vault(&env, 9);
        let est = estimate_rent(&env, 9, u32::MAX).unwrap();
        assert_eq!(est.total_rent, u64::MAX);
    }

    #[test]
    fn estimate_rent_days_helper() {
        let env = Env::default();
        setup_vault(&env, 11);
        let est = estimate_rent_days(&env, 11, 1).unwrap();
        assert_eq!(est.ledgers, LEDGERS_PER_DAY);
    }
}
