use core::ops;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Formatter;

use std::borrow::Borrow;
use std::str::FromStr;
use std::hash::Hash;

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
#[derive(Debug,Clone,Default,Hash,PartialEq,Eq,PartialOrd,Ord)]
pub struct FQDN(pub(crate) CString);

impl FQDN {

    pub fn from_vec(mut bytes: Vec<u8>) -> Result<Self,Error>
    {
        // add a trailing 0 if not present
        if bytes.last() != Some(&0) {
            bytes.push(0);
        }

        // check against 254 since we have the trailing char and the first label length to consider
        // (the trailing null bytes is supposet to be there)
        #[cfg(feature="domain-name-length-limited-to-255")]
        if bytes.len() > 254 {
            return Err(Error::TooLongDomainName);
        }

        // now, check each FQDN subpart (excluding the last nul char)
        let tochecklen = bytes.len()-1;
        let mut tocheck = &mut bytes[..tochecklen];
        while !tocheck.is_empty() {
            match tocheck[0] as usize {
                l if l >= tocheck.len() => {
                    return Err(Error::InvalidStructure);
                }

                #[cfg(feature="domain-label-length-limited-to-63")]
                l if l > 63 => {
                    return Err(Error::TooLongLabel);
                }

                0 => {
                    return Err(Error::EmptyLabel);
                }

                l => {
                    tocheck
                        .iter_mut()
                        .skip(1) // skip the label length
                        .take(l) // only process the current label
                        .try_for_each(|c| {
                            *c = check_and_lower_any_char(*c)?;
                            Ok::<(),Error>(())
                        })?;
                    tocheck = &mut tocheck[l+1..];
                }
            }
        }
        Ok(unsafe { Self::from_vec_with_nul_unchecked(bytes)})
    }

    /// Creates a FQDN from a vector of bytes without any checking
    ///
    /// # Safety
    /// The behaviour is unpredictable if:
    /// * the last bytes is not 0, or
    /// * the structure of the FQDN is corrupted (should be a sequence of labels), or
    /// * the label length is too high, or
    /// * the total length is too high, or
    /// * a not allowed character is used
    unsafe fn from_vec_with_nul_unchecked(v: Vec<u8>) -> Self
    {
        FQDN(CString::from_vec_with_nul_unchecked(v))
    }

    /// Creates a FQDN from an ascii string.
    ///
    /// Successive FQDN labels are supposed to be separated by a dot ('.')
    /// and all the used characters should be compliant with
    /// the RFC.
    pub fn from_ascii_str(s: &str) -> Result<Self,Error>
    {
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
            Some(&b'.') => {
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

        // check against 253 since we have the trailing char and the first label length to consider
        #[cfg(feature="domain-name-length-limited-to-255")]
        if toparse.len() > 253 {
            return Err(Error::TooLongDomainName);
        }

        // now, check each FQDN subpart and concatenate them
        toparse
            .split(|&c| c == b'.')
            .try_fold(Vec::with_capacity(s.len()+1),
                      |mut bytes, label|
                          match label.len() {

                              #[cfg(feature="domain-label-length-limited-to-63")]
                              l if l > 63 => Err(Error::TooLongLabel),

                              #[cfg(not(feature="domain-label-length-limited-to-63"))]
                              l if l > 255 => Err(Error::TooLongLabel),

                              0 => Err(Error::EmptyLabel),

                              l => {
                                  let mut iter = label.iter();

                                  // first, prepend the label length
                                  bytes.push(l as u8);

                                  // check and push all the other characters...
                                  iter.try_for_each(|&c| {
                                      bytes.push(check_and_lower_any_char(c)?);
                                      Ok(())
                                  } )?;

                                  Ok(bytes)
                              }
                          })
            .map(|bytes| {
                Self(unsafe { CString::from_vec_unchecked(bytes)})
            })
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


impl From<&Fqdn> for FQDN
{
    #[inline]
    fn from(s: &Fqdn) -> FQDN { FQDN(s.0.into()) }
}

impl From<Box<Fqdn>> for FQDN
{
    #[inline]
    fn from(s: Box<Fqdn>) -> FQDN {
        let cstr : Box<CStr> = unsafe { std::mem::transmute(s) };
        FQDN(cstr.into())
    }
}

impl TryFrom<CString> for FQDN
{
    type Error = Error;

    fn try_from(bytes: CString) -> Result<FQDN, Self::Error>
    {
        let mut bytes = bytes.into_bytes_with_nul().to_ascii_lowercase();
        if check_byte_sequence(bytes.as_ref()).is_ok() {
            Ok(unsafe { Self::from_vec_with_nul_unchecked(bytes) })
        } else {
            bytes.pop(); // pop the trailing nul terminator
            std::str::from_utf8(&bytes)
                .map_err(|_| Error::InvalidLabelChar)
                .and_then(FQDN::from_str)
        }
    }
}


impl TryFrom<Vec<u8>> for FQDN
{
    type Error = Error;

    #[inline]
    fn try_from(bytes: Vec<u8>) -> Result<FQDN, Self::Error> {
        Self::from_vec(bytes)
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

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        #[cfg(feature="punycode")] { Self::punyencode(s) }
        #[cfg(not(feature="punycode"))] { Self::from_ascii_str(s) }

    }
}


#[cfg(feature = "serde")]
impl serde::Serialize for FQDN {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FQDN {
    #[inline]
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            CString::deserialize(deserializer)
                .and_then(|str| Self::try_from(str).map_err(serde::de::Error::custom))
    }
}
