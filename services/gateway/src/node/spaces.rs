//! Public spaces in the commons (ADR 0065): a directory listing's seed names one as
//! `<creator>.space.<charter hash>` (`project_integrity::space_in_seed`), and every call about it
//! goes to the project role's base cell as `{space, input}`.

use holo_hash::ActionHashB64;
use holochain::prelude::ActionHash;
use serde::Serialize;

/// A call's input with the space it is about (the coordinators' `In<T>`).
#[derive(Serialize, Debug)]
pub struct Scoped<T> {
    pub space: ActionHash,
    pub input: T,
}

/// The space a listing's seed names, if it names one.
pub fn space_of_seed(seed: &str) -> Option<ActionHash> {
    let mut parts = seed.splitn(3, '.');
    let (_creator, word, id) = (parts.next()?, parts.next()?, parts.next()?);
    if word != "space" {
        return None;
    }
    ActionHashB64::from_b64_str(id).ok().map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::space_of_seed;
    use holo_hash::ActionHashB64;
    use holochain::prelude::ActionHash;

    #[test]
    fn reads_the_space_from_a_listing_seed() {
        let hash = ActionHash::from_raw_32(vec![7; 32]);
        let b64 = ActionHashB64::from(hash.clone());
        assert_eq!(space_of_seed(&format!("uhCAkAnn.space.{b64}")), Some(hash));
        assert_eq!(space_of_seed("uhCAkAnn.private.abc"), None);
        assert_eq!(space_of_seed("uhCAkAnn.abc"), None);
    }
}
