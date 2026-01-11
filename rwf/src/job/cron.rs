//! Implements the UNIX cron syntax.
use super::Error;
use std::ops::Range;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
enum CronValue {
    Exact(i64),
    Every(i64),
    Range(Range<i64>),
    Any,
}

impl CronValue {
    /// Parse a cron syntax.
    fn parse(value: &str) -> Result<Self, Error> {
        match value.parse::<i64>() {
            Ok(value) => Ok(CronValue::Exact(value)),
            Err(_) => {
                if value.starts_with("*/") {
                    match value.replace("*/", "").parse() {
                        Ok(value) => Ok(CronValue::Every(value)),
                        Err(_) => Err(Error::CronValueError),
                    }
                } else if value.contains("-") {
                    let range_parts = value.split("-").collect::<Vec<_>>();
                    if range_parts.len() != 2 {
                        return Err(Error::CronValueError);
                    }

                    let start = if let Ok(value) = range_parts[0].parse::<i64>() {
                        value
                    } else {
                        return Err(Error::CronValueError);
                    };

                    let end = if let Ok(value) = range_parts[1].parse::<i64>() {
                        value
                    } else {
                        return Err(Error::CronValueError);
                    };

                    Ok(CronValue::Range(Range { start, end }))
                } else if value == "*" {
                    Ok(CronValue::Any)
                } else {
                    Err(Error::CronValueError)
                }
            }
        }
    }

    /// Indicates the cron task should run at the specified time.
    pub fn matches(&self, time: i64) -> bool {
        match self {
            Self::Exact(value) => *value == time,
            Self::Every(value) => time % value == 0,
            Self::Range(value) => value.contains(&time),
            Self::Any => true,
        }
    }
}

/// UNIX cron syntax.
#[derive(Clone, Debug)]
pub struct Cron {
    second: CronValue,
    minute: CronValue,
    hour: CronValue,
    dom: CronValue,
    month: CronValue,
    dow: CronValue,
}

impl Cron {
    /// Parse the UNIX cron syntax.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::job::Cron;
    /// let cron = Cron::parse("* * * * *").unwrap();
    /// ```
    pub fn parse(value: &str) -> Result<Self, Error> {
        let parts = value.split(" ").collect::<Vec<_>>();

        match parts.len() {
            // Second is specified
            6 => Ok(Self {
                second: CronValue::parse(parts[0])?,
                minute: CronValue::parse(parts[1])?,
                hour: CronValue::parse(parts[2])?,
                dom: CronValue::parse(parts[3])?,
                month: CronValue::parse(parts[4])?,
                dow: CronValue::parse(parts[5])?,
            }),

            // Second is omitted.
            5 => Ok(Self {
                second: CronValue::Exact(0),
                minute: CronValue::parse(parts[0])?,
                hour: CronValue::parse(parts[1])?,
                dom: CronValue::parse(parts[2])?,
                month: CronValue::parse(parts[3])?,
                dow: CronValue::parse(parts[4])?,
            }),

            _ => Err(Error::CronValueError),
        }
    }

    /// Should the cron execute at the provided time?
    pub fn should_run(&self, time: &OffsetDateTime) -> bool {
        let second = time.second();
        let minute = time.minute();
        let hour = time.hour();
        let day = time.day();
        let month = time.month();
        let weekday = time.weekday().number_from_sunday();

        self.second.matches(second as i64)
            && self.minute.matches(minute as i64)
            && self.hour.matches(hour as i64)
            && self.dom.matches(day as i64)
            && self.month.matches(month as i64)
            && self.dow.matches(weekday as i64)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cron_parse() {
        let cron = Cron::parse("* * * * * *").unwrap();
        let time = OffsetDateTime::now_utc();

        // Will run on every tick of the clock.
        assert!(cron.should_run(&time));

        let cron = Cron::parse("*/5 * * * * *").unwrap();
        let will_run = time.replace_second(25).unwrap();
        assert!(cron.should_run(&will_run));

        let will_not = will_run.replace_second(7).unwrap();
        assert!(!cron.should_run(&will_not));

        let cron = Cron::parse("5 7-8 * * * *").unwrap();
        let will_run = will_not
            .replace_second(5)
            .unwrap()
            .replace_minute(7)
            .unwrap();

        assert!(cron.should_run(&will_run));

        let will_not = will_run
            .replace_second(25)
            .unwrap()
            .replace_minute(8)
            .unwrap();

        assert!(!cron.should_run(&will_not));

        let cron = Cron::parse("* * * * *").unwrap();
        let time = OffsetDateTime::now_utc().replace_second(0).unwrap();

        assert!(cron.should_run(&time));
    }
}
