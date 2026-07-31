use chrono::{DateTime, Datelike, Local, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};

/// Time-based notification cadence for portfolio summaries.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    EveryXDays(u32),
    Weekly,
    EveryOtherWeek,
    Monthly,
    EveryXHours(u32),
}

impl Frequency {
    /// Whether this frequency is gated by a specific local time of day.
    /// `EveryXHours` is purely interval-based and ignores `time_of_day`.
    pub fn uses_time_of_day(self) -> bool {
        !matches!(self, Frequency::EveryXHours(_))
    }

    /// Whether this frequency is tied to a specific day of the week.
    pub fn uses_day_of_week(self) -> bool {
        matches!(self, Frequency::Weekly | Frequency::EveryOtherWeek)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSchedule {
    pub id: String,
    pub name: String,
    pub frequency: Frequency,
    /// Local wall-clock time this should fire at (ignored for
    /// `EveryXHours`). Defaults to midnight for schedules saved before this
    /// field existed.
    #[serde(default = "default_time_of_day")]
    pub time_of_day: NaiveTime,
    /// Which day of the week this applies to, for `Weekly` /
    /// `EveryOtherWeek` schedules. Ignored otherwise. Defaults to Monday.
    #[serde(default = "default_weekday")]
    pub day_of_week: Weekday,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
}

fn default_weekday() -> Weekday {
    Weekday::Mon
}

fn default_time_of_day() -> NaiveTime {
    NaiveTime::from_hms_opt(0, 0, 0).expect("0:00:00 is always a valid time")
}

impl TimeSchedule {
    /// Whether this schedule is due to fire, checked against the current
    /// local time (schedules are configured in local wall-clock time/day,
    /// per how people actually think about "send me a summary at 8am").
    pub fn is_due(&self, now: DateTime<Local>) -> bool {
        if !self.enabled {
            return false;
        }

        // Interval-based: not gated by time-of-day or weekday at all.
        if let Frequency::EveryXHours(hours) = self.frequency {
            return match self.last_run {
                None => true,
                Some(last) => Utc::now() - last >= chrono::Duration::hours(hours.max(1) as i64),
            };
        }

        // Never fire twice on the same calendar day.
        let today = now.date_naive();
        if let Some(last) = self.last_run {
            if last.with_timezone(&Local).date_naive() == today {
                return false;
            }
        }

        if now.time() < self.time_of_day {
            return false;
        }

        match self.frequency {
            Frequency::Daily => true,
            Frequency::EveryXDays(x) => match self.last_run {
                None => true,
                Some(last) => (today - last.with_timezone(&Local).date_naive()).num_days() >= x.max(1) as i64,
            },
            Frequency::Weekly => now.weekday() == self.day_of_week,
            Frequency::EveryOtherWeek => {
                now.weekday() == self.day_of_week
                    && match self.last_run {
                        None => true,
                        Some(last) => (today - last.with_timezone(&Local).date_naive()).num_days() >= 13,
                    }
            }
            Frequency::Monthly => match self.last_run {
                None => true,
                Some(last) => (today - last.with_timezone(&Local).date_naive()).num_days() >= 28,
            },
            Frequency::EveryXHours(_) => unreachable!("handled above"),
        }
    }
}

/// Event-based alert conditions, evaluated against an investment's latest
/// known price / profit metrics each scheduler tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    PriceAbove(u64),
    PriceBelow(u64),
    ValueAbove(u64),
    ValueBelow(u64),
    ProfitAboveGp(i64),
    ProfitAbovePercent(f64),
    /// Percentage move (absolute value) since the previously recorded price.
    SignificantMove(f64),
}

impl EventKind {
    pub fn describe(self) -> String {
        match self {
            EventKind::PriceAbove(p) => format!("Price above {p} gp"),
            EventKind::PriceBelow(p) => format!("Price below {p} gp"),
            EventKind::ValueAbove(p) => format!("Value above {p} gp (whole investment)"),
            EventKind::ValueBelow(p) => format!("Value below {p} gp (whole investment)"),
            EventKind::ProfitAboveGp(p) => format!("Profit above {p} gp"),
            EventKind::ProfitAbovePercent(p) => format!("Profit above {p:.1}%"),
            EventKind::SignificantMove(p) => format!("Price moves by {p:.1}% or more"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRule {
    pub id: String,
    pub name: String,
    /// `None` means the rule applies to every active investment.
    pub item_id: Option<String>,
    pub kind: EventKind,
    pub enabled: bool,
    pub last_triggered: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleStore {
    pub time_schedules: Vec<TimeSchedule>,
    pub event_rules: Vec<EventRule>,
}
