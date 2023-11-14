use core::fmt;

use sea_orm::sea_query;
use serde::{Deserialize, Deserializer, de::{self, SeqAccess}};

// region:    --- OrderBy
#[derive(Debug, Clone)]
pub enum OrderBy {
	Asc(String),
	Desc(String),
}

impl core::fmt::Display for OrderBy {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
		match self {
			OrderBy::Asc(val) => {
				fmt.write_str(val)?;
				fmt.write_str(" ")?;
				fmt.write_str("ASC")?;
			}
			OrderBy::Desc(val) => {
				fmt.write_str(val)?;
				fmt.write_str(" ")?;
				fmt.write_str("DESC")?;
			}
		};

		Ok(())
	}
}

impl<T: AsRef<str>> From<T> for OrderBy {
	fn from(val: T) -> Self {
		let raw: &str = val.as_ref();

		if let Some(stripped) = raw.strip_prefix('!') {
			OrderBy::Desc(stripped.to_string())
		} else {
			OrderBy::Asc(raw.to_string())
		}
	}
}

// endregion: --- OrderBy

// region:    --- OrderBys
#[derive(Debug, Clone)]
pub struct OrderBys(Vec<OrderBy>);

impl OrderBys {
	pub fn new(v: Vec<OrderBy>) -> Self {
		OrderBys(v)
	}
	pub fn order_bys(self) -> Vec<OrderBy> {
		self.0
	}

	pub fn into_sea_orm_col_order_iter(
		self,
	) -> impl Iterator<Item = (String, sea_query::Order)> {
		self.0.into_iter().map(OrderBy::into_sea_orm_col_iter)
	}
}

impl<'de> Deserialize<'de> for OrderBys {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(OrderBysVisitor)
	}
}

struct OrderBysVisitor;

impl<'de> de::Visitor<'de> for OrderBysVisitor {
	type Value = OrderBys; // for deserialize

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(
			formatter,
			"OrderBysVisitor visitor not implemented for this type."
		)
	}

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Ok(OrderBy::from(v).into())
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Ok(OrderBy::from(v.to_string()).into())
	}

	fn visit_seq<A: SeqAccess<'de>>(
		self,
		mut seq: A,
	) -> Result<Self::Value, A::Error>
	where
		A: de::SeqAccess<'de>,
	{
		let mut order_bys: Vec<OrderBy> = Vec::new();

		while let Some(string) = seq.next_element::<String>()? {
			order_bys.push(OrderBy::from(string));
		}

		Ok(OrderBys::new(order_bys))
	}
	// FIXME: Needs to add support for visit_seq
}

// This will allow us to iterate over &OrderBys
impl<'a> IntoIterator for &'a OrderBys {
	type Item = &'a OrderBy;
	type IntoIter = std::slice::Iter<'a, OrderBy>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

// This will allow us to iterate over OrderBys directly (consuming it)
impl IntoIterator for OrderBys {
	type Item = OrderBy;
	type IntoIter = std::vec::IntoIter<OrderBy>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

// NOTE: If we want the Vec<T> and T, we have to make the individual from
//       specific to the type. Otherwise, conflict.

impl From<&str> for OrderBys {
	fn from(val: &str) -> Self {
		OrderBys(vec![val.into()])
	}
}
impl From<&String> for OrderBys {
	fn from(val: &String) -> Self {
		OrderBys(vec![val.into()])
	}
}
impl From<String> for OrderBys {
	fn from(val: String) -> Self {
		OrderBys(vec![val.into()])
	}
}

impl From<OrderBy> for OrderBys {
	fn from(val: OrderBy) -> Self {
		OrderBys(vec![val])
	}
}

impl<T: AsRef<str>> From<Vec<T>> for OrderBys {
	fn from(val: Vec<T>) -> Self {
		let d = val
			.into_iter()
			.map(|o| OrderBy::from(o))
			.collect::<Vec<_>>();
		OrderBys(d)
	}
}

// endregion: --- OrderBys
impl OrderBy {
	pub fn into_sea_orm_col_iter(self) -> (String, sea_query::Order) {
		let (col, order) = match self {
			OrderBy::Asc(col) => (col, sea_query::Order::Asc),
			OrderBy::Desc(col) => (col, sea_query::Order::Desc),
		};
		println!("{}:{:?}", col, order);
		(col, order)
	}
}
