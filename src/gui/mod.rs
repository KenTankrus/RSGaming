pub mod dashboard;
pub mod investments;
pub mod schedules;
pub mod settings;
pub mod statistics;
pub mod telegram;
pub mod widgets;

/// Transient, per-frame UI state (form inputs, etc.) that doesn't belong in
/// any persisted model but needs to survive across frames.
#[derive(Default)]
pub struct UiState {
    pub new_investment: NewInvestmentForm,
    pub new_schedule: NewScheduleForm,
    pub new_event_rule: NewEventRuleForm,
    pub telegram_show_secrets: bool,
    /// Which investment (by id) is currently being edited, and its
    /// in-progress edit buffer. `None` means nothing is being edited.
    pub editing: Option<(String, EditInvestmentForm)>,
}

pub struct NewInvestmentForm {
    pub game: crate::models::investment::Game,
    pub item_name: String,
    pub item_id: String,
    pub purchase_date: String,
    pub purchase_price: String,
    pub quantity: String,
    pub notes: String,
    /// Set when "Add investment" is clicked with an unparseable purchase
    /// date; shown as an inline error and blocks the add until fixed.
    pub date_error: bool,
}

impl Default for NewInvestmentForm {
    fn default() -> Self {
        Self {
            game: Default::default(),
            item_name: String::new(),
            item_id: String::new(),
            purchase_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            purchase_price: String::new(),
            quantity: String::new(),
            notes: String::new(),
            date_error: false,
        }
    }
}

/// Which `Frequency` variant to build -- kept separate from `Frequency`
/// itself so the "add schedule" form can pick a kind before deciding on
/// its `EveryXDays`/`EveryXHours` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyKind {
    Daily,
    EveryXDays,
    Weekly,
    EveryOtherWeek,
    Monthly,
    EveryXHours,
}

impl FrequencyKind {
    pub const ALL: [FrequencyKind; 6] = [
        FrequencyKind::Daily,
        FrequencyKind::EveryXDays,
        FrequencyKind::Weekly,
        FrequencyKind::EveryOtherWeek,
        FrequencyKind::Monthly,
        FrequencyKind::EveryXHours,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FrequencyKind::Daily => "Daily",
            FrequencyKind::EveryXDays => "Every X days",
            FrequencyKind::Weekly => "Weekly",
            FrequencyKind::EveryOtherWeek => "Every other week",
            FrequencyKind::Monthly => "Monthly",
            FrequencyKind::EveryXHours => "Every X hours",
        }
    }
}

pub struct NewScheduleForm {
    pub name: String,
    pub kind: FrequencyKind,
    pub every_x: u32,
    pub day_of_week: chrono::Weekday,
    pub hour: u32,
    pub minute: u32,
}

impl Default for NewScheduleForm {
    fn default() -> Self {
        Self {
            name: "Portfolio summary".to_string(),
            kind: FrequencyKind::Daily,
            every_x: 3,
            day_of_week: chrono::Weekday::Mon,
            hour: 8,
            minute: 0,
        }
    }
}

/// Which `EventKind` variant to build for a new event-based alert rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKindChoice {
    PriceAbove,
    PriceBelow,
    ValueAbove,
    ValueBelow,
    ProfitAboveGp,
    ProfitAbovePercent,
    SignificantMove,
}

impl EventKindChoice {
    pub const ALL: [EventKindChoice; 7] = [
        EventKindChoice::PriceAbove,
        EventKindChoice::PriceBelow,
        EventKindChoice::ValueAbove,
        EventKindChoice::ValueBelow,
        EventKindChoice::ProfitAboveGp,
        EventKindChoice::ProfitAbovePercent,
        EventKindChoice::SignificantMove,
    ];

    pub fn label(self) -> &'static str {
        match self {
            EventKindChoice::PriceAbove => "Price above (gp)",
            EventKindChoice::PriceBelow => "Price below (gp)",
            EventKindChoice::ValueAbove => "Value above (gp, whole investment)",
            EventKindChoice::ValueBelow => "Value below (gp, whole investment)",
            EventKindChoice::ProfitAboveGp => "Profit above (gp)",
            EventKindChoice::ProfitAbovePercent => "Profit above (%)",
            EventKindChoice::SignificantMove => "Significant move (%)",
        }
    }
}

pub struct NewEventRuleForm {
    pub name: String,
    pub kind: EventKindChoice,
    /// `None` = applies to every active investment; `Some(id)` targets one.
    pub target_investment_id: Option<String>,
    pub threshold: String,
    pub error: Option<String>,
}

impl Default for NewEventRuleForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: EventKindChoice::SignificantMove,
            target_investment_id: None,
            threshold: "10".to_string(),
            error: None,
        }
    }
}

/// Scratch text-buffer form used while editing an existing investment
/// in-place (separate from `NewInvestmentForm`, which is for creating new
/// ones -- keeping them distinct avoids one screen's in-progress typing
/// bleeding into the other).
pub struct EditInvestmentForm {
    pub game: crate::models::investment::Game,
    pub item_name: String,
    pub item_id: String,
    pub purchase_date: String,
    pub purchase_price: String,
    pub quantity: String,
    pub notes: String,
    pub error: Option<String>,
}

impl EditInvestmentForm {
    pub fn from_investment(inv: &crate::models::investment::Investment) -> Self {
        Self {
            game: inv.game,
            item_name: inv.item_name.clone(),
            item_id: inv.item_id.map(|id| id.to_string()).unwrap_or_default(),
            purchase_date: inv.purchase_date.format("%Y-%m-%d").to_string(),
            purchase_price: inv.purchase_price.to_string(),
            quantity: inv.quantity.to_string(),
            notes: inv.notes.clone().unwrap_or_default(),
            error: None,
        }
    }
}
