use core::ops;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Formatter;
use std::convert::TryInto;

use std::borrow::Borrow;
use std::str::FromStr;
use std::hash::{Hash, Hasher};

use crate::*;

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
pub struct FQDN(pub(crate) CString);


impl AsRef<Fqdn> for FQDN
{
    #[inline]
    fn as_ref(&self) -> &Fqdn {
        // SAFE because Fqdn is just a wrapper around CStr
        unsafe { &*(self.0.as_c_str() as *const CStr as *const Fqdn) }
    }
}

impl ops::Deref for FQDN
{
    type Target = Fqdn;
    #[inline]
    fn deref(&self) -> &Self::Target { self.as_ref() }
}

impl Hash for FQDN
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl PartialEq for FQDN
{
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.as_ref().eq(other.as_ref()) }
}

impl PartialEq<Fqdn> for FQDN
{
    #[inline]
    fn eq(&self, other: &Fqdn) -> bool { self.as_ref().eq(other) }
}

impl From<&Fqdn> for FQDN
{
    #[inline]
    fn from(s: &Fqdn) -> FQDN { s.to_owned() }
}

impl TryInto<FQDN> for CString
{
    type Error = Error;
    #[inline]
    fn try_into(self) -> Result<FQDN, Self::Error> {
        crate::check::check_byte_sequence(self.as_bytes_with_nul()).map(|_| FQDN(self))
    }
}

impl TryInto<FQDN> for Vec<u8>
{
    type Error = Error;
    fn try_into(mut self) -> Result<FQDN, Self::Error> {
        crate::check::check_byte_sequence(self.as_slice())
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
                        crate::check::check_char(i == 0, c)?;
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
