//! Long-running worker for `background_jobs` (Sprint 12).

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use cantrik_core::background::{
    BackgroundError, BackgroundJob, JobState, claim_next_queued_job,
    notification_channels_from_config, notify_approval_needed, set_rounds_done, touch_heartbeat,
    update_job_state,
};
use cantrik_core::config::{
    effective_background_max_llm_rounds, effective_voice_enabled, load_merged_config,
};
use cantrik_core::session::{SqlitePool, connect_pool, share_dir};
use cantrik_core::voice::speak_notification;
use fs2::FileExt;
use tokio::time::MissedTickBehavior;

use super::session_llm;

pub async fn run(poll_secs: u64) -> ExitCode {
    let lock_path = share_dir().join("daemon.lock");
    if let Some(parent) = lock_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let lock_file = match OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("cantrik daemon: cannot open {}: {e}", lock_path.display());
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = lock_file.try_lock_exclusive() {
        eprintln!(
            "cantrik daemon: another instance is running (lock {}: {e})",
            lock_path.display()
        );
        return ExitCode::from(1);
    }

    let pool = match connect_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("cantrik daemon: database: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut interval = tokio::time::interval(Duration::from_secs(poll_secs.max(1)));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    eprintln!(
        "cantrik daemon: started (poll {}s); lock {}",
        poll_secs.max(1),
        lock_path.display()
    );

    loop {
        tokio::select! {
            _ = async {
                #[cfg(unix)]
                {
                    use tokio::signal::unix::{signal, SignalKind};
                    let mut sigterm = signal(SignalKind::terminate()).ok();
                    let mut sigint = signal(SignalKind::interrupt()).ok();
                    match (sigterm.as_mut(), sigint.as_mut()) {
                        (Some(t), Some(i)) => {
                            tokio::select! {
                                _ = t.recv() => {}
                                _ = i.recv() => {}
                            }
                        }
                        (Some(t), None) => {
                            t.recv().await;
                        }
                        (None, Some(i)) => {
                            i.recv().await;
                        }
                        (None, None) => {
                            let _ = tokio::signal::ctrl_c().await;
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    let _ = tokio::signal::ctrl_c().await;
                }
            } => {
                eprintln!("cantrik daemon: shutting down (signal)");
                break;
            }
            _ = interval.tick() => {}
        }

        loop {
            let job = match claim_next_queued_job(&pool).await {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("cantrik daemon: claim job: {e}");
                    break;
                }
            };
            let Some(job) = job else { break };
            if let Err(e) = run_one_job(&pool, job).await {
                eprintln!("cantrik daemon: job error: {e}");
            }
        }
    }

    ExitCode::SUCCESS
}

async fn run_one_job(pool: &SqlitePool, job: BackgroundJob) -> Result<(), BackgroundError> {
    let cwd = PathBuf::from(&job.cwd);
    let config =
        load_merged_config(&cwd).map_err(|e| BackgroundError::Other(format!("config: {e}")))?;
    let max_rounds = effective_background_max_llm_rounds(&config.background) as i64;

    let user_line = format!(
        "[Background job {} — round {}/{}]\n{}",
        job.id,
        job.rounds_done + 1,
        max_rounds.max(1),
        job.goal
    );

    touch_heartbeat(pool, &job.id).await?;

    match session_llm::complete_with_session(&cwd, &config, &user_line).await {
        Ok(_) => {}
        Err(e) => {
            update_job_state(pool, &job.id, JobState::Failed, Some(&e.to_string()), None).await?;
            speak_notification(
                effective_voice_enabled(&config.ui),
                "Cantrik background job failed.",
            );
            return Ok(());
        }
    }

    touch_heartbeat(pool, &job.id).await?;
    let rounds = job.rounds_done + 1;
    set_rounds_done(pool, &job.id, rounds).await?;

    if rounds >= max_rounds {
        update_job_state(pool, &job.id, JobState::Completed, None, None).await?;
        speak_notification(
            effective_voice_enabled(&config.ui),
            "Cantrik background job finished.",
        );
    } else {
        let hint = format!("cantrik background resume {}", job.id);
        update_job_state(pool, &job.id, JobState::WaitingApproval, None, Some(&hint)).await?;
        let ch = notification_channels_from_config(&config.background, job.notify_on_approval);
        notify_approval_needed(&job.id, &hint, &ch).await;
    }

    Ok(())
}
