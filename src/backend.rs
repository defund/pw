use std::fmt;

use argonautica::Hasher;
use ring::{self, aead, digest};
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Entry {
	pub long: String,
	pub short: String,
	pub extra: String,
	pub salt: [u8; 16],
	pub sealed: Vec<u8>,
}

impl fmt::Display for Entry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.short.is_empty() && self.extra.is_empty() {
			write!(f, "- {}", self.long)
		} else if self.short.is_empty() {
			write!(f, "- {}: {}", self.long, self.extra)
		} else if self.extra.is_empty() {
			write!(f, "- {} ({})", self.long, self.short)
		} else {
			write!(f, "- {} ({}): {}", self.long, self.short, self.extra)
		}
	}
}

struct Counter {
	counter: u128,
}

impl Counter {
	fn new() -> Self {
		Self {
			counter: 0,
		}
	}
}

impl NonceSequence for Counter {
	fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
		let mut nonce = [0; 12];
		nonce.copy_from_slice(&self.counter.to_le_bytes()[..12]);
		self.counter += 1;
		Ok(Nonce::assume_unique_for_key(nonce))
	}
}

fn derive(master: &str, salt: &[u8]) -> Result<UnboundKey, ring::error::Unspecified> {
	let mut hasher = Hasher::default();
	hasher.opt_out_of_secret_key(true);
	let key = hasher
		.with_password(master)
		.with_salt(salt)
		.hash_raw()
		.unwrap();
	aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key.raw_hash_bytes())
}

fn hash(entry: &Entry) -> Aad<Vec<u8>> {
	let mut data = Vec::new();
	data.extend(digest::digest(&digest::SHA256, entry.long.as_bytes()).as_ref());
	data.extend(digest::digest(&digest::SHA256, entry.short.as_bytes()).as_ref());
	data.extend(digest::digest(&digest::SHA256, entry.extra.as_bytes()).as_ref());
	Aad::from(data)
}

pub fn seal(entry: &mut Entry, master: &str) -> Result<(), ring::error::Unspecified> {
	let mut key = SealingKey::new(derive(master, &entry.salt)?, Counter::new());
	let aad = hash(entry);
	key.seal_in_place_append_tag(aad, &mut entry.sealed)?;
	Ok(())
}

pub fn open(entry: &Entry, master: &str) -> Result<Vec<u8>, ring::error::Unspecified> {
	let mut key = OpeningKey::new(derive(master, &entry.salt)?, Counter::new());
	let aad = hash(entry);
	key.open_in_place(aad, &mut entry.sealed.clone())
		.map(|password| password.to_vec())
}
