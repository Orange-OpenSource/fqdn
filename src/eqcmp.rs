use std::cmp::Ordering;
use crate::{Fqdn, FQDN};
use crate::check::check_and_lower_any_char;
//--------------------------------------------------------------------------------------

impl PartialOrd<FQDN> for Fqdn
{
    #[inline]
    fn partial_cmp(&self, other: &FQDN) -> Option<Ordering> { self.partial_cmp(other.as_ref()) }
}

impl PartialOrd<Fqdn> for FQDN
{
    #[inline]
    fn partial_cmp(&self, other: &Fqdn) -> Option<Ordering> { self.as_ref().partial_cmp(other) }

    #[inline]
    fn lt(&self, other: &Fqdn) -> bool { self.as_ref().lt(other) }

    #[inline]
    fn le(&self, other: &Fqdn) -> bool { self.as_ref().le(other) }

    #[inline]
    fn gt(&self, other: &Fqdn) -> bool { self.as_ref().gt(other) }

    #[inline]
    fn ge(&self, other: &Fqdn) -> bool { self.as_ref().ge(other) }
}

impl PartialEq<Fqdn> for FQDN
{
    #[inline]
    fn eq(&self, other: &Fqdn) -> bool { self.as_ref().eq(other) }
}


impl<S:AsRef<str>> PartialEq<S> for FQDN
{
    #[inline]
    fn eq(&self, other: &S) -> bool { self.as_ref().eq(other) }
}


impl PartialEq<FQDN> for Fqdn
{
    #[inline]
    fn eq(&self, other: &FQDN) -> bool { self.eq(other.as_ref()) }
}


impl<S:AsRef<str>> PartialEq<S> for Fqdn
{
    #[inline]
    fn eq(&self, other: &S) -> bool
    {
        let mut fqdn = self.as_bytes().iter().skip(1);
        let mut str = other.as_ref().as_bytes().iter();
        loop {
            match (fqdn.next(), str.next()) {
                (None, None) => return true,
                (None, Some(_)) => return false,
                #[cfg(not(feature = "domain-name-should-have-trailing-dot"))]
                (Some(0), None) => return true, // trailing dot missing on str
                (Some(_), None) => return false,
                (Some(_), Some(b'.')) => { /* continue */ }
                (Some(&a), Some(&b)) => {
                    if Ok(a) != check_and_lower_any_char(b) {
                        return false; // found mismatch
                    }
                }
            }
        }
    }
}

impl PartialEq<FQDN> for &str
{
    #[inline]
    fn eq(&self, other: &FQDN) -> bool { other.eq(self) }
}


impl PartialEq<Fqdn> for &str
{
    #[inline]
    fn eq(&self, other: &Fqdn) -> bool { other.eq(self) }
}


