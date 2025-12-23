use captcha::fonts::Default as DefaultFont;
use captcha::fonts::Font;
use ic_cdk::{call, trap};
use lazy_static::lazy_static;
use rand_core::{RngCore, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use candid::Principal;
use ic_cdk::api::time;
use crate::http::requests::Challenge;
use crate::repository::repo::{CAPTCHA_CAHLLENGES, CONFIGURATION};

// Some time helpers
const fn secs_to_nanos(secs: u64) -> u64 {
    secs * 1_000_000_000
}
const MINUTE_NS: u64 = secs_to_nanos(60);
const HOUR_NS: u64 = 60 * MINUTE_NS;
const DAY_NS: u64 = 24 * HOUR_NS;
pub type Salt = [u8; 32];

#[derive(Clone, Debug)]
pub struct Base64(pub String);

lazy_static! {
    /// Problematic characters that are easily mixed up by humans to "normalized" replacement.
    /// I.e. the captcha will only contain a "replaced" character (values below in map) if the
    /// character also appears as a "replacement" (keys below in map). All occurrences of
    /// "replaced" characters in the user's challenge result will be replaced with the
    /// "replacements".
    /// Note: the captcha library already excludes the characters o, O and 0.
    static ref CHAR_REPLACEMENTS: HashMap<char, Vec<char>> = vec![
        ('c', vec!['c', 'C']),
        ('i', vec!['1', 'i', 'l', 'I', 'j']),
        ('s', vec!['s', 'S']),
        ('x', vec!['x', 'X']),
        ('y', vec!['y', 'Y']),
        ('z', vec!['z', 'Z']),
        ('p', vec!['p', 'P']),
        ('w', vec!['w', 'W']),
    ].into_iter().collect();


    /// The font (glyphs) used when creating captchas
    static ref CAPTCHA_FONT: DefaultFont = DefaultFont::new();

    /// The character set used in CAPTCHA challenges (font charset with replacements)
    static ref CHALLENGE_CHARSET: Vec<char> = {
        // To get the final charset:
        // * Start with all chars supported by the font by default
        // * Remove all the chars that will be "replaced"
        // * Add (potentially re-add) replacement chars
        let mut chars = CAPTCHA_FONT.chars();
        {
          let dropped: HashSet<char> = CHAR_REPLACEMENTS.values().flat_map(|x| x.clone()).collect();
          chars.retain(|c| !dropped.contains(c));
        }

        {
          chars.append(&mut CHAR_REPLACEMENTS.keys().copied().collect());
        }

        chars
    };

    static ref TEST_CAPTCHA: Vec<char>  = vec!['a'];

}

pub async fn generate_captcha() -> Challenge{
    let time = time();
    let mut rng = &mut make_rng().await;
    let key = random_string(&mut rng, 10);
    let challenges_in_progress = CAPTCHA_CAHLLENGES.with(|challenges| {
        challenges.borrow_mut().clean_expired_entries(time);
        challenges.borrow().count()
    });
    let chars: Option<String>;
    let challenge: Challenge;
    let max_free_captcha_per_minute = CONFIGURATION.with(|config| config.borrow().max_free_captcha_per_minute);
    if challenges_in_progress <= max_free_captcha_per_minute as usize {
        challenge = Challenge {
            png_base64: None,
            challenge_key: key.to_string(),
        };
        chars = None;
    } else {
        let (Base64(png_base64), res_chars) = create_captcha(rng);

        challenge = Challenge {
            png_base64: Some(png_base64),
            challenge_key: key.to_string(),
        };
        chars = Some(res_chars);
    }
    CAPTCHA_CAHLLENGES.with(|challenges| {
        challenges.borrow_mut().clean_expired_entries(time);
        challenges.borrow_mut().insert(
            key.clone(),
            chars,
            time,
        );
    });
    challenge
}

// Get a random number generator based on 'raw_rand'
pub async fn make_rng() -> rand_chacha::ChaCha20Rng {
    let seed = random_salt().await;
    rand_chacha::ChaCha20Rng::from_seed(seed)
}

// Generate an n-char long string of random characters. The characters are sampled from the rang
// a-z.
//
// NOTE: The 'rand' crate (currently) does not build on wasm32-unknown-unknown so we have to
// make-do with the RngCore trait (as opposed to Rng), therefore we have to implement this
// ourselves as opposed to using one of rand's distributions.
pub fn random_string<T: RngCore>(rng: &mut T, n: usize) -> String {
    let mut chars: Vec<u8> = vec![];

    // The range
    let a: u8 = b'a';
    let z: u8 = b'z';

    // n times, get a random number as u32, then shrink to u8, and finally shrink to the size of
    // our range. Finally, offset by the start of our range.
    for _ in 0..n {
        let next: u8 = rng.next_u32() as u8 % (z - a) + a;
        chars.push(next);
    }

    String::from_utf8_lossy(&chars).to_string()
}
const CAPTCHA_LENGTH: usize = 5;

pub fn create_captcha<T: RngCore>(rng: T) -> (Base64, String) {
    use captcha::filters::Wave;

    let mut captcha = captcha::new_captcha_with(rng, CAPTCHA_FONT.clone());

    let is_test = CONFIGURATION.with(|c| c.borrow().test_captcha);

    let captcha = captcha
        .set_charset(if is_test { &TEST_CAPTCHA } else { &CHALLENGE_CHARSET })
        .add_chars(CAPTCHA_LENGTH as u32)
        .apply_filter(Wave::new(2.0, 20.0).horizontal())
        .apply_filter(Wave::new(2.0, 20.0).vertical())
        .view(220, 120);

    let resp = match captcha.as_base64() {
        Some(png_base64) => Base64(png_base64),
        None => trap("Could not get base64 of captcha"),
    };

    (resp, captcha.chars_as_string())
}

/// Check whether the supplied CAPTCHA solution attempt matches the expected solution (after
/// normalizing ambiguous characters).
pub fn check_captcha_solution(solution_attempt: String, solution: String) -> Result<(), ()> {
    // avoid processing too many characters
    if solution_attempt.len() > CAPTCHA_LENGTH {
        return Err(());
    }
    // Normalize challenge attempts by replacing characters that are not in the captcha character set
    // with the respective replacement from CHAR_REPLACEMENTS.
    let normalized_solution_attempt: String = solution_attempt
        .chars()
        .map(|c| {
            // Apply all replacements
            *CHAR_REPLACEMENTS
                .iter()
                // For each key, see if the char matches any of the values (replaced chars) and if
                // so replace with the key itself (replacement char)
                .find_map(|(k, v)| if v.contains(&c) { Some(k) } else { None })
                .unwrap_or(&c)
        })
        .collect();

    if normalized_solution_attempt != solution {
        return Err(());
    }
    Ok(())
}


/// Calls raw rand to retrieve a random salt (32 bytes).
async fn random_salt() -> Salt {
    let res: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
        Ok((res,)) => res,
        Err((_, err)) => trap(&format!("failed to get salt: {err}")),
    };
    let salt: Salt = res[..].try_into().unwrap_or_else(|_| {
        trap(&format!(
            "expected raw randomness to be of length 32, got {}",
            res.len()
        ));
    });
    salt
}
