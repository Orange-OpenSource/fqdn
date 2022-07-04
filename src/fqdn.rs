use core::ops;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Formatter;
use std::convert::TryInto;

use std::borrow::Borrow;
use std::str::FromStr;
use std::hash::{Hash, Hasher};

use crate::*;
use crate::check::*;

/// A FQDN string.
///
/// The inner byte sequence is conformed with the RFC-1035: each label of the FQDN
/// is prefixed by a length byte and the sequence is nul-terminated.
///
/// For instance, the FQDN `github.com.` is exactly represented as `b"\x06github\x03com\x00"`.
///
/// [`FQDN`] is to [`&Fqdn`](`crate::Fqdn`) as [`String`] is to [`&str`]: the former
/// in each pair are owned data; the latter are borrowed references.
#[derive(Debug,Clone,Eq,Default)]
pub struct FQDN(pub(crate) CString);

impl FQDN
{
    fn from_vec(bytes: Vec<u8>) ->  Result<FQDN, Self::Error>
    {
        crate::check::check_byte_sequence(bytes.as_slice())
            .map(|_| FQDN(unsafe{CString::from_vec_unchecked(bytes)}))
    }

    fn from_vec_with_nul(mut bytes: Vec<u8>) -> Result<FQDN, Self::Error>
    {
        match bytes.last() {
            Some(0) => {
                bytes.pop(); // remaining trailing nul char
                FQDN::from_vec(bytes)
            }
            _ => Err(Error::TrailingNulCharMissing)
        }
    }

}
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
            return Err(Error::TooLongDomainName)
        }

        // check the trailing dot and remove it
        // (the empty FQDN '.' is also managed here)
        let s = s.as_bytes();
        let toparse =  match s.last() {
            None => {
                #[cfg(feature="domain-name-should-have-trailing-dot")]
                return Err(Error::TrailingDotMissing);
                #[cfg(not(feature="domain-name-should-have-trailing-dot"))]
                return Ok(Self(CString::default()));
            }
            Some(&c) if c == '.' as u8 => {
                // ok, there is a trailing dot
                if s.len() == 1 {
                    return Ok(Self(CString::default()));
                }
                &s[..s.len()-1]
            }
            _ => {
                #[cfg(feature="domain-name-should-have-trailing-dot")]
                return Err(Error::TrailingDotMissing);
                #[cfg(not(feature="domain-name-should-have-trailing-dot"))]
                s // no trailing dot to remove
            }
        };

        // now, check each FQDN subpart and concatenate them
        toparse
            .split(|&c| c == '.' as u8)
            .try_fold(Vec::with_capacity(s.len()+1),
            |mut bytes, label|
                match label.len() {
                    #[cfg(feature="domain-label-length-limited-to-63")]
                    l if l > 63 => Err(Error::TooLongLabel),
                    #[cfg(not(feature="domain-label-length-limited-to-63"))]
                    l if l > 255 => Err(Error::TooLongLabel),
                    l => {
                        let mut iter = label.iter();
                        #[cfg(feature="domain-label-should-start-with-letter")]
                        // check the first character (which can’t be a digit in some config)
                        iter.next().ok_or(Error::EmptyLabel).map(check_is_letter)?;
                        // check all the other characters...
                        iter.try_for_each(check_any_char)?;
                        // and concatenate to the fqdn to build
                        bytes.push(l as u8); // first, prepend the label length
                        bytes.extend_from_slice(label);
                        Ok(bytes)
                    }
                })
            .map(|bytes| {
                Self(unsafe { CString::from_vec_unchecked(bytes)})
            })
    }
}
