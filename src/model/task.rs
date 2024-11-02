use crate::ctx::Ctx;
use crate::model::{
    base::{self, DbBmc},
    Error, ModelManager, Result,
};
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::FromRow;

#[derive(Clone, Debug, Fields, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

#[derive(Fields, Deserialize)]
pub struct TaskForCreate {
    pub title: String,
}

#[derive(Fields, Deserialize)]
pub struct TaskForUpdate {
    pub title: Option<String>,
}

pub struct TaskBmc;

impl DbBmc for TaskBmc {
    const TABLE: &'static str = "tasks";
}

impl TaskBmc {
    pub async fn create(ctx: &Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, task_c).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, &mm, id).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, &mm, id).await
    }

    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        base::list::<Self, _>(ctx, &mm).await
    }

    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        task_u: TaskForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, &mm, id, task_u).await
    }
}

#[cfg(test)]
mod tests {
    use crate::_dev_utils;

    use super::*;
    use anyhow::Result;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok title";

        let task_c = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBmc::create(&ctx, &mm, task_c).await?;

        let (title,): (String,) = sqlx::query_as("SELECT title FROM task WHERE id = $1")
            .bind(id)
            .fetch_one(mm.db())
            .await?;

        assert_eq!(title, fx_title);

        let count = sqlx::query("DELETE FROM task WHERE id = $1")
            .bind(id)
            .execute(mm.db())
            .await?
            .rows_affected();

        assert_eq!(count, 1, "no row deleted");

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        let res = TaskBmc::get(&ctx, &mm, fx_id).await;

        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100
                }),
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_delete_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        let res = TaskBmc::delete(&ctx, &mm, fx_id).await;

        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100
                }),
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = ["test_list_ok title 1", "test_list_ok title 2"];

        _dev_utils::seed_test(&ctx, &mm, &fx_titles).await?;

        let tasks = TaskBmc::list(&ctx, &mm).await?;

        let tasks: Vec<Task> = tasks
            .into_iter()
            .filter(|t| t.title.starts_with("test_list_ok title"))
            .collect();

        assert_eq!(tasks.len(), 2, "number of seeded tasks");

        for task in tasks.iter() {
            TaskBmc::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_update_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_update_ok title";
        let fx_title_updated = "test_update_ok title updated";
        let fx_task = _dev_utils::seed_test(&ctx, &mm, &[fx_title])
            .await?
            .remove(0);

        TaskBmc::update(
            &ctx,
            &mm,
            fx_task.id,
            TaskForUpdate {
                title: Some(fx_title_updated.to_string()),
            },
        );

        let task = TaskBmc::get(&ctx, &mm, fx_task.id).await?;

        assert_eq!(task.title, fx_title_updated);

        Ok(())
    }
}