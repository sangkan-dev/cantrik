//! Skill `.md` files and project `rules.md` injection (Sprint 13, PRD §7, §4.19).

mod load;
mod select;

pub use load::{
    ENV_NO_RULES, list_skill_paths, load_rules_text, project_rules_path, read_skill_file,
    skills_dir,
};
pub use select::{format_rules_block, format_skills_block, select_skills_for_prompt};
