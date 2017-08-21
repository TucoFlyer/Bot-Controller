use rand::os::OsRng;
use rand::Rng;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use base64;

pub fn make_random_string() -> String {
    let mut rng = OsRng::new().expect("can't access the OS random number generator");
    rng.gen_ascii_chars().take(30).collect()
}

pub fn authenticate(challenge: &str, key: &str, digest: &str) -> bool {
    match base64::decode(digest) {
        Err(_) => false,
        Ok(bin_digest) => {
            if bin_digest.len() != 64 {
                false
            } else {
                let mut mac = Hmac::<Sha512>::new(key.as_bytes());
                mac.input(challenge.as_bytes());
                mac.verify(&bin_digest)
            }
        }
    }
}
