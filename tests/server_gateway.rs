use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use revolver::gateway::Gateway;
use revolver::engine::scheduler::run_scheduler;
use revolver::protocol::{EngineToUi, UiToEngine};

#[tokio::test]
async fn scheduler_emits_updates_via_gateway() {
    let (ui_to_engine_tx, ui_to_engine_rx) = mpsc::channel(16);
    let (engine_to_ui_tx, engine_to_ui_rx) = mpsc::channel(64);

    tokio::spawn(run_scheduler(ui_to_engine_rx, engine_to_ui_tx));
    let mut gateway = Gateway::new(ui_to_engine_tx, engine_to_ui_rx);

    gateway
        .send(UiToEngine::AddTask {
            name: "gateway test".to_string(),
        })
        .await;

    let msg = timeout(Duration::from_secs(3), gateway.recv())
        .await
        .expect("timeout waiting for engine message")
        .expect("engine channel closed");

    assert!(
        matches!(msg, EngineToUi::TaskUpdate(revolver::engine::scheduler::TaskUpdate::Upsert(_))),
        "expected an Upsert update"
    );

    gateway.send(UiToEngine::Shutdown).await;
}

#[tokio::test]
async fn cancel_task_sends_update() {
    let (ui_to_engine_tx, ui_to_engine_rx) = mpsc::channel(16);
    let (engine_to_ui_tx, engine_to_ui_rx) = mpsc::channel(64);

    tokio::spawn(run_scheduler(ui_to_engine_rx, engine_to_ui_tx));
    let mut gateway = Gateway::new(ui_to_engine_tx, engine_to_ui_rx);

    gateway
        .send(UiToEngine::AddTask {
            name: "cancel test".to_string(),
        })
        .await;

    let mut task_id = None;
    for _ in 0..5 {
        if let Ok(Some(EngineToUi::TaskUpdate(revolver::engine::scheduler::TaskUpdate::Upsert(
            snapshot,
        )))) = timeout(Duration::from_secs(2), gateway.recv()).await
        {
            task_id = Some(snapshot.id);
            break;
        }
    }

    let task_id = task_id.expect("did not receive initial task update");
    gateway.send(UiToEngine::CancelTask { id: task_id }).await;

    let mut saw_cancel = false;
    for _ in 0..10 {
        if let Ok(Some(EngineToUi::TaskUpdate(revolver::engine::scheduler::TaskUpdate::Upsert(
            snapshot,
        )))) = timeout(Duration::from_secs(2), gateway.recv()).await
        {
            if matches!(snapshot.status, revolver::task::TaskStatus::Cancelled) {
                saw_cancel = true;
                break;
            }
        }
    }

    assert!(saw_cancel, "never observed a cancelled status update");
    gateway.send(UiToEngine::Shutdown).await;
}
