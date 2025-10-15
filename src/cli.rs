use clap::{Args, Parser, Subcommand};

use crate::models::{Recur, Priority, State};

#[derive(Parser, Debug, Clone)]
#[command(name = "Jolonel", version = "1.0", author = "Muhammad", about = "Use me to track your progress on your Goals",
long_about = "I am Jolonel, a simple program made to track your progress easier, and keep you accountable. To get a brief how-to, run --docs")]
pub(crate) struct JNEL{
    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Mode {
    /// Create, Start, Edit (prioritize, tag, set deadline), Complete and Delete your tasks
    #[command(subcommand)]
    Do(DoCmd),

    /// Search, Filter and Sort your tasks
    #[command(subcommand)]
    View(ViewCmd),
    
    /// Modify the rules. 
    /// Create and Delete: Tags, Goals, Date/Time shorthands, Recurring rules
    #[command(subcommand)]
    Change(ChangeCmd),

    // Get analytics of your perfomance and streaks
    // #[command(subcommand)]
    // Report(time interval),


    // Undo
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum DoCmd {
    /// Create a new task.
    New {
        title: String,

        #[arg(short, long)]
        priority: Option<Priority>,
        
        #[command(flatten)]
        due: Option<DueArgs>,
        
        #[command(flatten)]
        recur: Option<Recurrence>,

        #[arg(short, long)]
        state: Option<State>,

        #[command(flatten)]
        goal: Option<GoalArgs>,

        #[command(flatten)]
        tag: Option<TagArgs>,
    },
    /*
    /// Start a task
    /// Prioritize a task
    /// Tag a task
    /// Set Deadline for a task
    /// Finish a task
    /// Delete a task
    */
}

#[derive(Debug, Clone, Copy, Args)]
pub struct Recurrence{
    #[arg(short, long)]
    pub recur: Option<Recur>,

    #[arg(short = 'A', long)]
    pub at_time: Option<chrono::NaiveTime>
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[group(required = false, multiple = false, args = ["due_short", "due_datetime"])]
pub(crate) struct DueArgs {
    /// Use the predefined shorthands
    #[arg(short, long)]
    pub(crate) due_short: Option<String>,
    /// Type the due datetime in the format "YYYY-MM-DD HH:MM:SS" (in double quotes)
    #[arg(short = 'D', long)]
    pub(crate) due_datetime: Option<String>,
}

// impl DueArgs {
//     pub fn get_due(&self) -> Option<NaiveDateTime> {
//         if
//     }
// }

#[derive(Args, Debug, Clone)]
#[group(required = false, multiple = false, args = ["goal", "new_goal"])]
pub(crate) struct GoalArgs {
    #[arg(short, long)]
    pub(crate) goal: Option<String>,

    #[arg(short = 'G', long)]
    pub(crate) new_goal: Option<String>,
}

#[derive(Args, Debug, Clone)]
#[group(required = false, args = ["tag", "new_tag"])]
pub(crate) struct TagArgs {
    #[arg(short, long)]
    pub(crate) tag: Option<Vec<String>>,

    #[arg(short = 'T', long)]
    pub(crate) new_tag: Option<Vec<String>>,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ViewCmd {
    All,
    Search { keyword: String },
    History,
}


#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ChangeCmd {
    #[command(subcommand)]
    Tag(TagCmd),
    #[command(subcommand)]
    Project(ProjectCmd),
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TagCmd {
    Add { name: String },
    Delete { name: String },
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ProjectCmd {
    Add { name: String },
    Delete { name: String },
}