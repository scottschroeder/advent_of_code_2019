use std::fmt;
#[derive(Clone, Copy, PartialEq)]
pub struct Key(u32);

impl From<char> for Key {
    fn from(c: char) -> Self {
        let idx = (c.to_ascii_lowercase() as u8 - 'a' as u8) as usize;
        assert!(idx < 32);
        let a = u32::MAX >> idx;
        let b = u32::MAX >> (idx + 1);
        Key(a ^ b)
    }
}

impl From<Key> for char {
    fn from(k: Key) -> Self {
        ((k.0.leading_zeros() as u8) + ('a' as u8)) as char
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct KeySet(u32);

impl KeySet {
    pub fn new() -> Self {
        KeySet(0)
    }
    #[inline]
    pub fn insert(self, k: Key) -> KeySet {
        KeySet(self.0 | k.0)
    }
    #[inline]
    pub fn contains(self, k: Key) -> bool {
        (self.0 & k.0) > 0
    }
    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn iter(self) -> impl Iterator<Item = Key> {
        ('a'..='z')
            .map(|c| Key::from(c))
            .filter(move |k| self.contains(*k))
    }
}

impl fmt::Debug for KeySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeySet(")?;
        for k in self.iter() {
            write!(f, "{}", char::from(k))?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_key_reflective() {
        for c in 'a'..='z' {
            let k = Key::from(c);
            let rc = char::from(k);
            assert_eq!(rc, c);
        }
        for (cap_c, c) in ('A'..='Z').into_iter().zip('a'..='z') {
            let k = Key::from(cap_c);
            let rc = char::from(k);
            assert_eq!(rc, c);
        }
    }
}
