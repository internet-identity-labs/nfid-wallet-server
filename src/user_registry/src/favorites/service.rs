use super::types::{FavoriteKind, Favorites, FavoritesError, MAX_FAVORITES, MAX_FAVORITE_ID_LENGTH};
use crate::FAVORITES;

pub fn get(root: &str) -> Favorites {
    FAVORITES.with(|favorites| favorites.borrow().get(root).cloned().unwrap_or_default())
}

/// Marks `id` as favorite or not. Idempotent: adding a favorite twice or removing
/// one that isn't there changes nothing. Returns the user's favorites after the change.
pub fn set(root: String, kind: FavoriteKind, id: String, favorite: bool) -> Result<Favorites, FavoritesError> {
    if favorite {
        validate_id(&id)?;
    }

    FAVORITES.with(|favorites| {
        let mut favorites = favorites.borrow_mut();
        let user = favorites.entry(root.clone()).or_default();
        let list = match kind {
            FavoriteKind::Token => &mut user.tokens,
            FavoriteKind::Contact => &mut user.contacts,
        };

        if favorite {
            if !list.contains(&id) {
                if list.len() >= MAX_FAVORITES {
                    return Err(FavoritesError::MaxFavoritesReached);
                }
                list.push(id);
            }
        } else {
            list.retain(|existing| existing != &id);
        }

        let result = user.clone();
        if result == Favorites::default() {
            favorites.remove(&root);
        }
        Ok(result)
    })
}

/// Drops contacts that no longer exist in the address book.
pub fn remove_contacts(root: &str, ids: &[String]) {
    FAVORITES.with(|favorites| {
        let mut favorites = favorites.borrow_mut();
        if let Some(user) = favorites.get_mut(root) {
            user.contacts.retain(|id| !ids.contains(id));
            if *user == Favorites::default() {
                favorites.remove(root);
            }
        }
    });
}

pub fn remove_all_contacts(root: &str) {
    FAVORITES.with(|favorites| {
        let mut favorites = favorites.borrow_mut();
        if let Some(user) = favorites.get_mut(root) {
            user.contacts.clear();
            if *user == Favorites::default() {
                favorites.remove(root);
            }
        }
    });
}

fn validate_id(id: &str) -> Result<(), FavoritesError> {
    if id.trim().is_empty() {
        return Err(FavoritesError::EmptyId);
    }
    if id.len() > MAX_FAVORITE_ID_LENGTH {
        return Err(FavoritesError::IdTooLong);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> String {
        format!("root-{}", std::thread::current().name().unwrap_or("t"))
    }

    #[test]
    fn adds_in_order_and_ignores_duplicates() {
        let r = root();
        set(r.clone(), FavoriteKind::Token, "icp:a".into(), true).unwrap();
        set(r.clone(), FavoriteKind::Token, "btc".into(), true).unwrap();
        let f = set(r.clone(), FavoriteKind::Token, "icp:a".into(), true).unwrap();
        assert_eq!(f.tokens, vec!["icp:a".to_string(), "btc".to_string()]);
        assert!(f.contacts.is_empty());
    }

    #[test]
    fn removes_and_cleans_up_empty_user() {
        let r = root();
        set(r.clone(), FavoriteKind::Contact, "c1".into(), true).unwrap();
        let f = set(r.clone(), FavoriteKind::Contact, "c1".into(), false).unwrap();
        assert_eq!(f, Favorites::default());
        assert!(FAVORITES.with(|fav| !fav.borrow().contains_key(&r)));
        // Removing a missing id is not an error.
        assert!(set(r, FavoriteKind::Contact, "missing".into(), false).is_ok());
    }

    #[test]
    fn validates_ids() {
        let r = root();
        assert_eq!(set(r.clone(), FavoriteKind::Token, " ".into(), true), Err(FavoritesError::EmptyId));
        let long = "x".repeat(MAX_FAVORITE_ID_LENGTH + 1);
        assert_eq!(set(r, FavoriteKind::Token, long, true), Err(FavoritesError::IdTooLong));
    }

    #[test]
    fn enforces_limit_per_list() {
        let r = root();
        for i in 0..MAX_FAVORITES {
            set(r.clone(), FavoriteKind::Token, format!("t{i}"), true).unwrap();
        }
        assert_eq!(
            set(r.clone(), FavoriteKind::Token, "one-more".into(), true),
            Err(FavoritesError::MaxFavoritesReached)
        );
        // The other list has its own limit.
        assert!(set(r, FavoriteKind::Contact, "c".into(), true).is_ok());
    }

    #[test]
    fn removes_deleted_contacts() {
        let r = root();
        set(r.clone(), FavoriteKind::Contact, "c1".into(), true).unwrap();
        set(r.clone(), FavoriteKind::Contact, "c2".into(), true).unwrap();
        set(r.clone(), FavoriteKind::Token, "btc".into(), true).unwrap();
        remove_contacts(&r, &["c1".to_string()]);
        assert_eq!(get(&r).contacts, vec!["c2".to_string()]);
        remove_all_contacts(&r);
        assert!(get(&r).contacts.is_empty());
        assert_eq!(get(&r).tokens, vec!["btc".to_string()]);
    }
}
