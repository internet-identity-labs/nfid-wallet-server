use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Upper bound of each favorites list per user.
pub const MAX_FAVORITES: usize = 100;
/// Upper bound of a favorite id in bytes.
pub const MAX_FAVORITE_ID_LENGTH: usize = 256;

/// The user's favorite tokens and contacts, in the order they were added.
///
/// Ids are opaque to the canister. The wallet keys a token by chain and address
/// (for example `icp:<ledger>`, `evm:<chain id>:<contract>`, `btc`) and a contact
/// by `AddressBookUserAddress.id`.
#[derive(CandidType, Deserialize, Clone, Serialize, Debug, Default, PartialEq, Eq)]
pub struct Favorites {
    pub tokens: Vec<String>,
    pub contacts: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug, PartialEq, Eq)]
pub enum FavoritesError {
    EmptyId,
    IdTooLong,
    MaxFavoritesReached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FavoriteKind {
    Token,
    Contact,
}
