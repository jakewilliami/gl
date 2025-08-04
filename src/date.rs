use chrono::{DateTime, FixedOffset, Local};

#[derive(Clone)]
pub struct CommitDate {
    pub abs: DateTime<FixedOffset>,
    pub rel: String,
}

impl CommitDate {
    pub fn format_abs(&self) -> String {
        self.abs.format("%a %d %b %Y").to_string()
    }
}

trait ToChrono {
    fn to_fixed_offset(&self) -> DateTime<FixedOffset>;
}

// Convert gix::date::Time to DateTime<FixedOffset>
//
// NOTE also: gix-implemented conversion from gix::date::Time to jiff::Zoned:
//   https://github.com/GitoxideLabs/gitoxide/blob/ccd6525c/gix-date/src/time/format.rs#L53-L58
impl ToChrono for gix::date::Time {
    fn to_fixed_offset(&self) -> DateTime<FixedOffset> {
        let utc_time = DateTime::from_timestamp(self.seconds, 0).unwrap();
        let offset = if self.offset > 0 {
            FixedOffset::east_opt(self.offset).unwrap()
        } else {
            FixedOffset::west_opt(self.offset).unwrap()
        };
        utc_time.with_timezone(&offset)
    }
}

trait ToRelative {
    fn to_relative(&self) -> String;
}

impl ToRelative for gix::date::Time {
    // We want to convert a given time into a relative (from now) human-readable string.
    // `gix` uses `jiff`'s `Span` with relative dates:
    //   https://github.com/GitoxideLabs/gitoxide/blob/ccd6525c1b39cea9c12a88d6ac4adf06d0df0b9c/gix-date/src/parse.rs#L169
    //
    // But this does the conversion in the other direction, so we have to implement it
    // ourselves.
    //
    // Converting a given time into a relative, human-readable string is ported directly
    // from `git`:
    //   https://github.com/git/git/blob/facbe4f6/date.c#L135-L208
    fn to_relative(&self) -> String {
        fn time_ago(diff: i64, unit: &str) -> String {
            if diff == 1 {
                format!("{diff} {unit} ago")
            } else {
                format!("{diff} {unit}s ago")
            }
        }

        let then = self.to_fixed_offset();
        let now = Local::now().with_timezone(then.offset());

        if now < then {
            return String::from("in the future");
        }

        let mut diff = (now - then).num_seconds();
        if diff < 90 {
            return time_ago(diff, "second");
        }

        // Turn it into minutes
        diff = (diff + 30) / 60;
        if diff < 90 {
            return time_ago(diff, "minute");
        }

        // Turn it into hours
        diff = (diff + 30) / 60;
        if diff < 36 {
            return time_ago(diff, "hour");
        }

        // We deal with number of days from here on
        diff = (diff + 12) / 24;
        if diff < 14 {
            return time_ago(diff, "day");
        }

        // Saw weeks for the past 10 weeks or so
        if diff < 70 {
            return time_ago((diff + 3) / 7, "week");
        }

        // Say months for the past 12 months or so
        if diff < 365 {
            return time_ago((diff + 15) / 30, "month");
        }

        // Give years and months for 5 years or so
        if diff < 1825 {
            let total_months = (diff * 12 * 2 + 365) / (365 * 2);
            let years = total_months / 12;
            let months = total_months % 12;

            if months != 0 {
                return format!("{}, {}", time_ago(years, "year"), time_ago(months, "month"));
            } else {
                return time_ago(years, "year");
            }
        }

        // Otherwise, just years.  Centuries is probably overkill
        time_ago((diff + 183) / 365, "year")
    }
}

// Convert gix::date::Time to CommitDate
impl From<gix::date::Time> for CommitDate {
    fn from(time: gix::date::Time) -> Self {
        let abs = time.to_fixed_offset();
        // let repr = time.format(gix::date::time::format::ISO8601);
        let rel = time.to_relative();
        Self { abs, rel }
    }
}
