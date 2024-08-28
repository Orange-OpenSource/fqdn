use std::cmp::Ordering;
use crate::{Fqdn, FQDN};

impl PartialEq for FQDN
{
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.as_ref().eq(other.as_ref()) }
}

impl Eq for FQDN { }

impl PartialOrd for FQDN
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }

    #[inline]
    fn lt(&self, other: &Self) -> bool { self.as_ref().lt(other.as_ref()) }

    #[inline]
    fn le(&self, other: &Self) -> bool { self.as_ref().le(other.as_ref()) }

    #[inline]
    fn gt(&self, other: &Self) -> bool { self.as_ref().gt(other.as_ref()) }

    #[inline]
    fn ge(&self, other: &Self) -> bool { self.as_ref().ge(other.as_ref()) }
}

impl Ord for FQDN
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering { self.as_ref().cmp(other.as_ref()) }
}

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

impl PartialEq<FQDN> for Fqdn
{
    #[inline]
    fn eq(&self, other: &FQDN) -> bool { self.eq(other.as_ref()) }
}

//--------------------------------------------------------------------------------------

impl PartialEq for Fqdn
{
    fn eq(&self, other: &Self) -> bool
    {
        let b1 = self.as_bytes();
        let b2 = other.as_bytes();
        (b1.len() == b2.len()) && {
            let i1 = b1.iter().map(|&i| i.to_ascii_lowercase());
            let i2 = b2.iter().map(|&i| i.to_ascii_lowercase());
            i1.eq(i2)
        }
    }
}

impl Eq for Fqdn { }

impl PartialOrd for Fqdn
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Fqdn
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        let i1 = self.as_bytes().iter().map(|&i| i.to_ascii_lowercase());
        let i2 = other.as_bytes().iter().map(|&i| i.to_ascii_lowercase());
        i1.cmp(i2)
    }
}
