//! TUI module - Terminal User Interface

pub mod action_executor;
pub mod app;
pub mod big_text;
pub mod cli_bridge;
pub mod clipboard;
pub mod doctor_state;
pub mod effects;
pub mod handler;
pub mod perf_state;
pub mod pie_chart;
pub mod query_state;
pub mod screens;
pub mod spinner;
pub mod staking;
pub mod state;
pub mod theme;
pub mod toast;
pub mod transfer_state;
pub mod widgets;

pub use action_executor::{
    build_pending_action, dialog_type_to_action_type, execute_staking_action, parse_amount_to_wei,
    validate_dialog_input,
};
pub use app::TuiApp;
pub use big_text::{AnimatedLogo, BigTextMode, MonadLogo, WelcomeScreen};
pub use clipboard::{copy_to_clipboard, paste_from_clipboard};
pub use doctor_state::{CheckCategory, CheckStatus, DoctorCheck, DoctorState};
pub use effects::{
    AnimatedGradient, AnimationManager, GlowEffect, GlowIntensity, GlowWidgets, GradientState,
    GradientStop, PulseState,
};
pub use handler::{Action, DialogAction, DoctorAction, StakingAction, TransferAction};
pub use perf_state::{CpuCoreData, DiskData, NetworkThroughput, PerfState};
pub use pie_chart::{cpu_pie_chart, disk_pie_chart, memory_pie_chart, PieChart, PieChartSize};
pub use query_state::{QueryResult, QueryState, QueryType, ValidatorSetType};
pub use screens::{
    DashboardScreen, DoctorScreen, HelpScreen, Screen, ScreenRender, StakingScreen, TransferScreen,
};
pub use spinner::{LoadingSpinner, SpinnerManager, SpinnerStyle};
pub use staking::{
    DelegationInfo, PendingStakingAction, PendingWithdrawal, StakingActionResult,
    StakingActionType, StakingState,
};
pub use state::{AppState, NetworkData, SystemData, ValidatorData};
pub use theme::THEME;
pub use toast::{Toast, ToastHelpers, ToastManager, ToastType};
