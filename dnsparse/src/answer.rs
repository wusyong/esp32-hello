use crate::{Name, QueryKind, QueryClass};

pub struct Answer<'a> {
  pub name: Name<'a>,
  pub kind: QueryKind,
  pub class: QueryClass,
  pub ttl: u32,
  pub rdata: &'a [u8],
}
