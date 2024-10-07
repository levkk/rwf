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
    pub fn parse(value: &str) -> Result<Self, Error> {
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

    pub fn matches(&self, time: i64) -> bool {
        match self {
            Self::Exact(value) => *value == time,
            Self::Every(value) => time % value == 0,
            Self::Range(value) => value.contains(&time),
            Self::Any => true,
        }
    }
}

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
    pub fn parse(value: &str) -> Result<Self, Error> {
        let parts = value.split(" ").collect::<Vec<_>>();

        if parts.len() != 6 {
            return Err(Error::CronValueError);
        }

        Ok(Self {
            second: CronValue::parse(parts[0])?,
            minute: CronValue::parse(parts[1])?,
            hour: CronValue::parse(parts[2])?,
            dom: CronValue::parse(parts[3])?,
            month: CronValue::parse(parts[4])?,
            dow: CronValue::parse(parts[5])?,
        })
    }

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

pub struct Schedule {
    crons: Vec<Cron>,
}
