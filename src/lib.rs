//! A fully qualified domain name representation
//!
//! Notice that a fully qualified domain name (or FQDN) is case-insensivite.
//! By this way, the implementation of traits like `Hash` or `PartialEq` do the same.
//!
//! # Crate features
//! Some limitations are enforced by the Internet RFC but some of them are defaultly relaxed to fit
//! with more applicative contexts. Features are available in order to activate or not these
//! limitations, depending on applicative purposes.
//!
//! These features control how the parsing of a String should be done.
//! Violation of one of these activated limitations raises an error (see [`Error`]).
//!
//! ### `domain-label-length-limited-to-63`
//! The internet standards specifies that each label of a FQDN is limited to 63 characters.
//! By default, this crate allows up to 256 characters but the 63 limitation could be set
//! through the activation of this feature.
//!
//! ### `domain-name-length-limited-to-255`
//! The internet standards specifies that the total length of a FQDN is limited to 255 characters.
//! By default, the only limit is the available memory but the 255 limitation could be set
//! through the activation of this feature.
//!
//! ### `domain-name-without-special-chars`
//! The internet standards specifies that a FQDN should only contains digits, letters and hyphen (`-`).
//! But, many network equipment accepts also `_` (underscore) without problems. If this crate is used to design
//! something like a firewall, it could be necessary to deal with this, so do this crate.
//! At the contrary, the activation of this feature refuses such special characters.
//!
//! ### `domain-label-should-start-with-letter`
//! The internet standards specifies that FQDN should always start with a letter (nor a digit, nor a hyphen).
//! By default, this crate accept any of theses characters event at the first position.
//! The activation of this feature enforces the use of a letter at the beginning of FQDN.
//!
//! # RFC 1035
//! The RFC 1035 has some restrictions that are not activated by default.
//! The feature `strict-rfc-1035` activates all of them:
//! * `domain-label-length-limited-to-63`
//! * `domain-name-length-limited-to-255`
//! * `domain-name-without-special-chars`
//! * `domain-label-should-start-with-letter`
//!
//! See above for more details.
//!
use core::ops;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::{Formatter, Write};
use std::convert::TryInto;

use std::borrow::Borrow;
use std::str::FromStr;
use std::hash::{Hash, Hasher};


/// Parses a list of strings and creates an new
/// FQDN by concatenating them.
///
/// If the trailing dot is missing, it is automatically added.
///
/// # Examples
/// ```
/// use fqdn::fqdn;
///
/// let fqdn = fqdn!("rust-lang", "github.io");
/// ```
/// # Panics
/// If one of the elements is not a valid symbol, the macro panics.
/// ```should_panic
/// use fqdn::fqdn;
///
/// let s = fqdn!("w@w","fr"); // panics !!
/// ```
#[macro_export]
macro_rules! fqdn {
    ($($args:expr),*) => {{
        #[allow(unused_mut)]
        let mut str = std::string::String::new();
        $( str += $args; )*
        match str.as_str().as_bytes().last() {
            None => $crate::FQDN::default(),
            Some(b'.') => str.parse::<$crate::FQDN>().unwrap(),
            _ => (str + ".").parse::<$crate::FQDN>().unwrap(),
        }
    }}
}

/// A FQDN string.
///
/// The inner byte sequence is conformed with the RFC-1035: each label of the FQDN
/// is prefixed by a length byte and the sequence is nul-terminated.
///
/// For instance, the FQDN `github.com.` is exactly represented as `b"\x06github\x03com\x00"`.
///
/// `FQDN` is to [`&Fqdn`] as [`String`] is to [`&str`]: the former
/// in each pair are owned data; the latter are borrowed references.
#[derive(Debug,Clone,Eq,Default)]
pub struct FQDN(CString);


/// A borrowed FQDN (as a slice).
#[derive(Debug,Eq)]
pub struct Fqdn(CStr);

impl Fqdn {

    /// Checks if this is the top domain.
    ///
    /// The human-readable representation of the top domain is the single dot `.`.
    #[inline]
    pub fn is_root(&self) -> bool { self.0.to_bytes_with_nul()[0] == 0 }

    /// Gets the top level domain.
    ///
    /// The TLD is the last part of a FQDN.
    /// For instance, the TLD of `rust-lang.github.io.` is `io.`
    ///
    /// If the FQDN is already a TLD, `self` is returned.
    /// If the FQDN is the top domain, `None` is returned.
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = fqdn!("rust-lang.github.com.");
    /// assert_eq![ fqdn.tld(), Some(fqdn!("com.").as_ref()) ];
    /// ```
    #[inline]
    pub fn tld(&self) -> Option<&Fqdn>
    {
        let mut index = 0;
        let mut jump = unsafe { *self.as_bytes().get_unchecked(index) } as usize;
        if jump == 0 {
            if self.is_root() { None } else { Some(&self) }
        } else {
            loop {
                let next_index = index + jump + 1;
                let next_jump = unsafe { *self.as_bytes().get_unchecked(next_index) } as usize;
                if next_jump == 0 {
                    return Some ( unsafe { &*(&self.0[index..] as *const CStr as *const Fqdn) } );
                }
                index = next_index;
                jump = next_jump;
            }
        }
    }

    /// Extracts a `Fqdn` slice with contains the immediate parent domain.
    ///
    /// The parent is the domain after remaining the first label.
    /// If it is already the top domain, then `None` is returned.
    ///
    /// To iterate over the hierarchy of a FQDN, consider [`Self::hierarchy`]
    ///
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = fqdn!("github.com");
    /// assert_eq![ fqdn.parent(), Some(fqdn!("com").as_ref()) ];
    /// assert_eq![ fqdn.parent().unwrap().parent(), None ];
    /// ```
    #[inline]
    pub fn parent(&self) -> Option<&Fqdn>
    {
        match unsafe { *self.as_bytes().get_unchecked(0) } as usize {
            0 => None,
            len => {
                let parent = unsafe { &*(&self.0[1+len..] as *const CStr as *const Fqdn) };
                if parent.is_root() { None } else { Some(parent) }
            }
        }
    }


    /// Iterates over the parents of the FQDN.
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = "rust-lang.github.com.".parse::<FQDN>().unwrap();
    /// let mut iter = fqdn.hierarchy();
    /// assert_eq![ iter.next(), Some(fqdn!("rust-lang.github.com.").as_ref()) ];
    /// assert_eq![ iter.next(), Some(fqdn!("github.com.").as_ref()) ];
    /// assert_eq![ iter.next(), Some(fqdn!("com.").as_ref()) ];
    /// assert_eq![ iter.next(), None ];
    /// ```
    #[inline]
    pub fn hierarchy(&self) -> impl '_ + Iterator<Item=&Fqdn>
    {
        struct Iter<'a>(&'a Fqdn);

        impl<'a> Iterator for Iter<'a>
        {
            type Item = &'a Fqdn;
            fn next(&mut self) -> Option<<Self as Iterator>::Item>
            {
                match unsafe { *self.0.as_bytes().get_unchecked(0) } as usize {
                    0 => None,
                    len => {
                        let current = self.0;
                        self.0 = unsafe { &*(&self.0.0[1 + len..] as *const CStr as *const Fqdn) };
                        Some(current)
                    }
                }
            }
        }
        Iter(&self)
    }

    #[inline]
    pub fn is_subdomain_of(&self, suffix:&Fqdn) -> bool
    {
         self.as_bytes().len() >= suffix.as_bytes().len()
            && are_equivalent(&self.as_bytes()[self.as_bytes().len()-suffix.as_bytes().len()..],suffix.as_bytes())
    }

    /// Computes the depth of this domain (i.e. counts the labels)
    #[inline]
    pub fn depth(&self) -> usize { self.segments().count() }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self,Error>
    {
        check_byte_sequence(bytes)
            .map(|_| unsafe {
                // it is safe because check does the necessary stuff... (including trailing nul char)
                // and because Fqdn is just a wrapper around CStr
                &*(CStr::from_bytes_with_nul_unchecked(bytes) as *const CStr as *const Fqdn)
            })
    }

    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self
    {
        &*(CStr::from_bytes_with_nul_unchecked(bytes) as *const CStr as *const Fqdn)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] { self.0.to_bytes_with_nul() }

    #[inline]
    pub fn as_c_str(&self) -> &CStr { &self.0 }

    /// An iterator visiting the label position of the FQDN.
    ///
    /// A label is identified by the start and the end index
    /// of the range of its bytes.
    /// Note that a label starts on the byte which defines its length,
    /// not on its first character byte.
    ///
    /// The returned item of this iterator is a pair of `usize`.
    /// The first one is the start position of the segment and the
    /// second one is the immediate position after the last byte of the
    /// segment.
    ///
    /// Except for the empty domain, the iteration always starts
    /// with (0,n).
    ///
    /// # Examples
    /// ```ignore
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = "mail.orange.com.".parse::<FQDN>().unwrap();
    /// let mut iter = fqdn.segments();
    /// assert_eq![ iter.next(), Some((0,5)) ];
    /// assert_eq![ iter.next(), Some((5,12)) ];
    /// assert_eq![ iter.next(), Some((12,16)) ];
    /// assert_eq![ iter.next(), None ];
    /// ```
    #[inline]
    fn segments(&self) -> impl '_ + Iterator<Item = (usize,usize)>
    {
        struct Iter<'a> {
            fqdn: &'a[u8], // nul terminated
            pos: usize
        }
        impl<'a> Iterator for Iter<'a>
        {
            type Item = (usize,usize);
            fn next(&mut self) -> Option<<Self as Iterator>::Item>
            {
                match unsafe { *self.fqdn.get_unchecked(self.pos) } as usize {
                    0 => None,
                    n => {
                        let start = self.pos;
                        self.pos += 1 + n;
                        Some((start, self.pos))
                    }
                }
            }

        }
        Iter { fqdn: self.as_bytes(), pos: 0 }
    }


    /// Iterates over the labels which constitutes the FQDN.
    /// # Example
    /// ```
    /// # use fqdn::*;
    /// # use std::str::FromStr;
    /// let fqdn = fqdn!("rust-lang.github.com.");
    /// let mut iter = fqdn.labels();
    /// assert_eq![ iter.next(), Some("rust-lang") ];
    /// assert_eq![ iter.next(), Some("github") ];
    /// assert_eq![ iter.next(), Some("com") ];
    /// assert_eq![ iter.next(), None ];
    /// # assert_eq![ iter.next(), None ];
    /// ```
    #[inline]
    pub fn labels(&self) -> impl '_ + Iterator<Item=&str>
    {
        let bytes = self.as_bytes();
        self.segments().map(move |(s,e)| unsafe {
            std::str::from_utf8_unchecked(&bytes[s+1..e])
        })
    }
}




impl ToOwned for Fqdn {
    type Owned = FQDN;
    fn to_owned(&self) -> FQDN { FQDN(self.0.to_owned()) }
}

impl fmt::Display for Fqdn
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
    {
        let bytes = self.as_bytes();
        if bytes[0] == 0 { // root domain
            f.write_char('.')
        } else {
            let mut iter = self.segments();
            iter.try_for_each(|(s, e)| {
                bytes[s + 1..e].iter().try_for_each(|&c| f.write_char(c as char))?;
                f.write_char('.')
            })
        }
    }
}

impl PartialEq for Fqdn
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        are_equivalent(&self.as_bytes(), &other.as_bytes())
    }
}

impl PartialEq<FQDN> for Fqdn
{
    #[inline]
    fn eq(&self, other: &FQDN) -> bool { self.eq(other.as_ref()) }
}

impl PartialEq<Fqdn> for FQDN
{
    #[inline]
    fn eq(&self, other: &Fqdn) -> bool { self.as_ref().eq(other) }
}


impl AsRef<Fqdn> for FQDN
{
    fn as_ref(&self) -> &Fqdn {
        // SAFE because Fqdn is just a wrapper around CStr
        unsafe { &*(self.0.as_c_str() as *const CStr as *const Fqdn) }
    }
}

impl ops::Deref for FQDN
{
    type Target = Fqdn;
    fn deref(&self) -> &Self::Target { self.as_ref() }
}

impl Hash for FQDN
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_bytes().iter().for_each(|c| c.to_ascii_lowercase().hash(state))
    }
}

impl PartialEq for FQDN
{
    fn eq(&self, other: &Self) -> bool {
        are_equivalent(&self.0.as_bytes_with_nul(), &other.0.as_bytes_with_nul())
    }
}

impl From<&Fqdn> for FQDN
{
    fn from(s: &Fqdn) -> FQDN {
        s.to_owned()
    }
}

impl TryInto<FQDN> for CString
{
    type Error = Error;
    fn try_into(self) -> Result<FQDN, Self::Error> {
        check_byte_sequence(self.as_bytes_with_nul()).map(|_| FQDN(self))
    }
}

impl TryInto<FQDN> for Vec<u8>
{
    type Error = Error;
    fn try_into(mut self) -> Result<FQDN, Self::Error> {
        check_byte_sequence(self.as_slice())
            .map(|_| {
                self.pop(); // pops the terminated last nul char since
                // from_vec_unchecked will add a new one...
                FQDN(unsafe{CString::from_vec_unchecked(self)})
            })
    }
}

impl Borrow<Fqdn> for FQDN {
    #[inline]
    fn borrow(&self) -> &Fqdn { self.as_ref() }
}

impl fmt::Display for FQDN
{
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { self.as_ref().fmt(f) }
}

impl FromStr for FQDN
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        // check against 255 since we expected the trailing dot
        #[cfg(feature="domain-name-length-limited-to-255")]
        if s.len() > 255 {
            return Err(Error::TooLongDomainName { len: s.len() })
        }
        // if unlimited, then the radix trie limits it to u32::MAX
        #[cfg(not(feature="domain-name-length-limited-to-255"))]
        if s.len() > u32::MAX as usize {
            return Err(Error::TooLongDomainName)
        }

        let mut bytes = Vec::with_capacity(s.len()+1);
        let mut toparse = s.as_bytes();
        loop {
            // search next dot...
            let stop = toparse.into_iter().enumerate()
                .find(|(_,&c)| c == '.' as u8)
                .map(|(n,_)| n);

            match stop {
                None if toparse.is_empty() => { // yes, parsing is done !
                    return Ok(Self(unsafe { CString::from_vec_unchecked(bytes)}))
                }
                None => {
                    return Err(Error::TrailingDotMissing)
                }
                Some(0) if s.len() == 1 => {
                    return Ok(Self(CString::default()));
                }
                Some(0) => {
                    return Err(Error::EmptyLabel)
                }
                #[cfg(feature="domain-label-length-limited-to-63")]
                Some(len) if len > 63 => {
                    return Err(Error::TooLongLabel)
                }
                #[cfg(not(feature="domain-label-length-limited-to-63"))]
                Some(len) if len > 255 => {
                    return Err(Error::TooLongLabel)
                }
                Some(n) => {
                    bytes.push(n as u8);
                    (0..n).into_iter().try_for_each(|i| {
                        let c = unsafe { *toparse.get_unchecked(i) };
                        check_char(i == 0, c)?;
                        Ok(bytes.push(c))
                    })?;
                    toparse = &toparse[n+1..];
                }
            }
        }
    }
}

/*
impl FQDN {
    pub fn from_str_without_trailing_dot(s: &str) -> Result<Self, Error>
    {
        // to improve (i.e. without creating a string)
        let s = s.to_string() + ".";
        FQDN::from_str(&s)
    }
}*/

// value of the _ depends if we apply the rfc strictly or not
#[cfg(feature="domain-name-without-special-chars")] const __: u8 = 0;
#[cfg(not(feature="domain-name-without-special-chars"))] const __: u8 = 38;

// size of the alphabet
#[cfg(feature="domain-name-without-special-chars")]
#[allow(dead_code)] pub(crate) const ALPHABET_SIZE: usize = 38; // 26 letters + 10 digits + '-' + others (0)
#[cfg(not(feature="domain-name-without-special-chars"))]
#[allow(dead_code)] pub(crate) const ALPHABET_SIZE: usize = 39; // we should also count the '_'

// in order to decrease the necessary memory, this table reduces the search space only
// to allowed chars in FQDN, i.e. a-zA-Z, 0-9 and -.
// -> underscore is exceptionnally added since it often appears (control plane ?)
// all the others are treated equally (i.e. as a dot)
// this is case insensitive (lower and upper case give the same index)

pub(crate) const ALPHABET: [u8;256] = [
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,   //  16
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,   //  32
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0,37, 0, 0,   //  48 (-)
    27,28,29,30,31,32,33,34,    35,36, 0, 0, 0, 0, 0, 0,   //  64 (0-9)
    0, 1, 2, 3, 4, 5, 6, 7,     8, 9,10,11,12,13,14,15,   //  80 (A-O)
    16,17,18,19,20,21,22,23,    24,25,26, 0, 0, 0, 0,__,   //  96 (P-Z et _)
    0, 1, 2, 3, 4, 5, 6, 7,     8, 9,10,11,12,13,14,15,   // 112 (a-o)
    16,17,18,19,20,21,22,23,    24,25,26, 0, 0, 0, 0, 0,   // 128 (p-z)
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,     0, 0, 0, 0, 0, 0, 0, 0
];

pub(crate) fn are_equivalent(bytes1:&[u8], bytes2:&[u8]) -> bool
{
    let mut i1 = bytes1.iter();
    let mut i2 = bytes2.iter();

    loop {
        match i1.next() {
            None => return i2.next().is_none(),
            Some(&step) =>  match i2.next() {
                None => return false, // fqdn have different number of labels
                Some(&n) if n != step => return false, // labels have different sizes
                Some(_) => if (0..step as usize).into_iter()// check label characters (according to alphabet)
                    .any(|_| ALPHABET[*i1.next().unwrap() as usize] != ALPHABET[*i2.next().unwrap() as usize]) { return false; }
            }
        }
    }
}

/// Error when FQDN parsing goes wrong
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub enum Error {

    /// The trailing dot of the FQDN string is missing.
    ///
    /// A valid FQDN string should be ended by a dot (e.g. `github.com.`).
    TrailingDotMissing,

    /// The trailing nul byte of the FQDN bytes is missing.
    ///
    /// A valid FQDN array of bytes should be ended by the nul byte (e.g. `b"\x06github\x03com\x00"`)
    TrailingNulCharMissing,

    /// An invalid character is found in a label of the FQDN.
    ///
    /// The allowed characters in a FQDN label are letters, digits and `'-'`.
    /// By default, this crate also accepts `'_'` in FQDN but this behavior could be deactivated with
    /// the `strict-rfc-1035` feature.
    InvalidLabelChar,

    /// The analysed bytes are not consistent with a FQDN sequence of bytes.
    ///
    /// Typically, the length bytes of labels are not consistent.
    InvalidStructure,
    /// The name of the domain is too long
    ///
    /// By default, there is no limit except if the `strict-rfc-1035` feature is selected and
    /// then, the domain name should be less than 255 characters (including the trailing dot).
    TooLongDomainName,

    /// One label of the FQDN is too long
    ///
    /// The returned error contains the excessive length.
    ///
    /// By default, the limit is set to 255 characters but if the `strict-rfc-1035` feature is selected,
    /// then this limit is set to `63` (as said in the RFC).
    TooLongLabel,

    /// One label does not start with a letter
    ///
    /// The returned error contains the start position of the involved label
    LabelDoesNotStartWithLetter,

    /// One label is empty (e.g. starting dot as `.github.com.` or two following dots as `github..com.`)
    EmptyLabel
}

// Checks if the bytes are really a FQDN (without nul char)
pub(crate) fn check_byte_sequence(bytes: &[u8]) -> Result<(),Error>
{
    // check against 256 since we have the trailing char and the first label length to consider
    #[cfg(feature="domain-name-length-limited-to-255")]
    if bytes.len() > 256 {
        return Err(Error::TooLongDomainName)
    }
    // if unlimited, then the radix trie limits it to u32::MAX
    #[cfg(not(feature="domain-name-length-limited-to-255"))]
    if bytes.len() > u32::MAX as usize {
        return Err(Error::TooLongDomainName)
    }

    match bytes.last() {
        Some(0) => {
            let mut iter = bytes.iter();
            let remaining = bytes.len() - 1;
            while let Some(&c) = iter.next() {
                if remaining < c as usize {
                    return Err(Error::InvalidStructure)
                }
                #[cfg(feature="domain-label-length-limited-to-63")]
                if c > 63 {
                    return Err(Error::TooLongLabel)
                }
                (0..c as usize).into_iter().try_for_each(|c| Ok({check_char(c==0, *iter.next().unwrap())?;}))?;
            }
            Ok(())
        }
        Some(_) => Err(Error::TrailingNulCharMissing),
        None => Err(Error::TrailingNulCharMissing),
    }

}

pub(crate) fn check_char(_first: bool, c: u8) -> Result<u8,Error>
{
    match ALPHABET[c as usize] {
        0 => Err(Error::InvalidLabelChar),
        #[cfg(feature="domain-label-should-start-with-letter")]
        _ if _first && !c.is_ascii_alphabetic() => Err(Error::LabelDoesNotStartWithLetter),
        n => Ok(n)
    }
}


#[cfg(test)]
mod tests {
    use crate as fqdn;
    use fqdn::*;

    #[test]
    fn parsing_string()
    {
        assert!(FQDN::default().is_root());
        assert!("github.com.".parse::<FQDN>().is_ok());

        assert_eq!("github.com".parse::<FQDN>(), Err(fqdn::Error::TrailingDotMissing));
        assert_eq!("github..com.".parse::<FQDN>(), Err(fqdn::Error::EmptyLabel));
        assert_eq!(".github.com.".parse::<FQDN>(), Err(fqdn::Error::EmptyLabel));
        assert_eq!("git@ub.com.".parse::<FQDN>(), Err(fqdn::Error::InvalidLabelChar));

    }

    #[test]
    fn parsing_bytes()
    {
        assert!(Fqdn::from_bytes(b"\x06github\x03com\x00").is_ok());

        assert_eq!(Fqdn::from_bytes(b"\x06github\x03com"), Err(fqdn::Error::TrailingNulCharMissing));
        assert_eq!(Fqdn::from_bytes(b"\x06g|thub\x03com\x00"), Err(fqdn::Error::InvalidLabelChar));

        #[cfg(feature="domain-label-should-start-with-letter")]
        assert_eq!(Fqdn::from_bytes(b"\x04yeah\x0512345\x03com\x00"), Err(fqdn::Error::LabelDoesNotStartWithLetter));

    }


    #[test]
    fn depth()
    {
        assert_eq!(".".parse::<FQDN>().map(|f| f.is_root()), Ok(true));
        assert_eq!(".".parse::<FQDN>().map(|f| f.depth()), Ok(0));
        assert_eq!("github.com.".parse::<FQDN>().map(|f| f.depth()), Ok(2));
        assert_eq!("rust-lang.github.com.".parse::<FQDN>().map(|f| f.depth()), Ok(3));
    }

    #[test]
    fn subdomains()
    {
        let a = "rust-lang.github.com.".parse::<FQDN>().unwrap();
        let b = "GitHub.com.".parse::<FQDN>().unwrap();

        assert!( a.is_subdomain_of(&a));
        assert!( a.is_subdomain_of(&b));
        assert!( !b.is_subdomain_of(&a));
    }
}

