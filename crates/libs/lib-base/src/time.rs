use chrono::{DateTime, Local};
pub use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

// region:    --- Time
pub fn now_utc() -> OffsetDateTime {
	OffsetDateTime::now_utc()
}

pub fn format_time(time: OffsetDateTime) -> String {
	time.format(&Rfc3339).unwrap()
}

pub fn now_utc_plus_sec_str(sec: f64) -> String {
	let new_time = now_utc() + Duration::seconds_f64(sec);
	format_time(new_time)
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
	OffsetDateTime::parse(moment, &Rfc3339)
		.map_err(|_| Error::FailToDateParse(moment.to_string()))
}

pub fn date_time_with_zone() -> DateTime<Local> {
	// let dt = Local::now().to_rfc3339();
	// let dt = Local::now();
	// let ret = dt.to_rfc3339();
	// let dt = format_time(now_utc());
	// let res = DateTime::<Local>::to_rfc3339(dt.as_str());

	let dt = Local::now();
	let naive_utc = dt.naive_utc();
	let offset = dt.offset().clone();
	DateTime::<Local>::from_naive_utc_and_offset(naive_utc, offset)
}

// endregion: --- Time

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	FailToDateParse(String),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
