
use _common::*;

pub trait Hash
{
	fn hash(&self) -> u64;
}

impl Hash for usize {
	fn hash(&self) -> u64 {
		*self as u64
	}
}

pub struct HashMap<K: Hash,V>
{
	ents: Vec<Option<(K,V)>>,
}
pub struct Iter<'a, K: 'a, V: 'a>
{
	pos: usize,
	ents: &'a [Option<(K,V)>],
}

impl<K: Hash, V> HashMap<K,V>
{
	pub fn new() -> HashMap<K,V> {
		HashMap {
			ents: Vec::new(),
		}
	}
	
	/// Returns the previous item (replaced), if any
	pub fn insert(&mut self, key: K, value: V) -> Option<V> {
		unimplemented!();
	}
	
	pub fn iter(&self) -> Iter<K,V> {
		Iter {
			pos: 0,
			ents: &*self.ents,
		}
	}
}

impl<'a, K, V> ::core::iter::Iterator for Iter<'a, K, V>
{
	type Item = (&'a K, &'a V);
	
	fn next(&mut self) -> Option<(&'a K, &'a V)>
	{
		while self.pos < self.ents.len()
		{
			let e = self.ents[self.pos].as_ref();
			self.pos += 1;
			if let Some(e) = (e)
			{
				return Some( (&e.0, &e.1) );
			}
		}
		None
	}
}

