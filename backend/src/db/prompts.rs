use anyhow::Result;
use crate::db::types::prompt::PromptRowWithModel;
use diff::{lines, Result as DiffResult};

#[derive(Clone, Debug)]
pub struct PromptRepository {
    pool: sqlx::SqlitePool,
}

impl PromptRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(PromptRepository { pool })
    }

    #[cfg(test)]
    pub async fn in_memory(pool: sqlx::SqlitePool) -> Result<Self> {
        Self::new(pool.clone()).await
    }

    pub async fn create_prompt(
        &self,
        key: &str,
        system_prompt: &str,
        user_prompt: &str,
        model_id: i64,
        max_tokens: i64,
        temperature: f64,
        json_mode: bool,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;

        // 1. get the latest version number for this prompt (if any)
        let latest_version: Option<i64> = sqlx::query!(
            "SELECT MAX(version_number) as version_number FROM prompt_version"
        )
        .fetch_optional(&mut *conn)
        .await?
        .and_then(|row| row.version_number);

        let next_version = latest_version.map(|v| v + 1).unwrap_or(1);

        // 2. insert the new prompt version
        let prompt_version_id = sqlx::query!(
            r#"
            INSERT INTO prompt_version (
                version_number,
                system_diff,
                user_diff,
                system,
                user,
                model_id,
                max_tokens,
                temperature,
                json_mode
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            next_version,
            "", // initial system_diff is empty
            "", // initial user_diff is empty
            system_prompt,
            user_prompt,
            model_id,
            max_tokens,
            temperature,
            json_mode
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

        // 3. insert the prompt, linking to the new version
        let prompt_id = sqlx::query!(
            r#"
            INSERT INTO prompt (
                key,
                prompt_version_id
            )
            VALUES (?, ?)
            "#,
            key,
            prompt_version_id
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

        Ok(prompt_id)
    }

    pub async fn get_prompt(&self, id: i64) -> Result<PromptRowWithModel> {
        let prompt = sqlx::query_as!(
            PromptRowWithModel,
            r#"
            SELECT
                p.id,
                p.key,
                pv.system,
                pv.user,
                pv.model_id,
                pv.max_tokens,
                pv.temperature,
                pv.json_mode,
                m.name as model_name,
                pr.name as provider_name,
                pv.system_diff,
                pv.user_diff,
                pv.version_number,
                pv.id as version_id,
                pv.created_at,
                pv.updated_at
            FROM prompt p
            JOIN prompt_version pv ON p.prompt_version_id = pv.id
            JOIN model m ON pv.model_id = m.id
            JOIN provider pr ON m.provider_id = pr.id
            WHERE p.id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(prompt)
    }

    pub async fn list_prompts(&self) -> Result<Vec<PromptRowWithModel>> {
        let prompts = sqlx::query_as!(
            PromptRowWithModel,
            r#"
            SELECT
                p.id,
                p.key,
                pv.system,
                pv.user,
                pv.model_id,
                pv.max_tokens,
                pv.temperature,
                pv.json_mode,
                m.name as model_name,
                pr.name as provider_name,
                pv.system_diff,
                pv.user_diff,
                pv.version_number,
                pv.id as version_id,
                pv.created_at,
                pv.updated_at
            FROM prompt p
            JOIN prompt_version pv ON p.prompt_version_id = pv.id
            JOIN model m ON pv.model_id = m.id
            JOIN provider pr ON m.provider_id = pr.id
            ORDER BY pv.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(prompts)
    }

    pub async fn update_prompt(
        &self,
        id: i64,
        key: &str,
        system_prompt: &str,
        user_prompt: &str,
        model_id: i64,
        max_tokens: i64,
        temperature: f64,
        json_mode: bool,
    ) -> Result<bool> {
        let mut conn = self.pool.acquire().await?;

        // 1. get the current prompt version
        let current_prompt = self.get_prompt(id).await?;

        // 2. calculate the diffs
        let system_diff = generate_diff(&current_prompt.system, system_prompt);
        let user_diff = generate_diff(&current_prompt.user, user_prompt);

        // 3. get the latest version number for this prompt
        let latest_version: Option<i64> = sqlx::query!(
            "SELECT MAX(version_number) as version_number FROM prompt_version"
        )
        .fetch_optional(&mut *conn)
        .await?
        .and_then(|row| row.version_number);

        let next_version = latest_version.map(|v| v + 1).unwrap_or(1);

        // 4. insert the new prompt version
        let prompt_version_id = sqlx::query!(
            r#"
            INSERT INTO prompt_version (
                version_number,
                system_diff,
                user_diff,
                system,
                user,
                model_id,
                max_tokens,
                temperature,
                json_mode
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            next_version,
            system_diff,
            user_diff,
            system_prompt,
            user_prompt,
            model_id,
            max_tokens,
            temperature,
            json_mode
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

        // 5. update the prompt to point to the new version
        let rows_affected = sqlx::query!(
            r#"
            UPDATE prompt
            SET
                key = ?,
                prompt_version_id = ?
            WHERE id = ?
            "#,
            key,
            prompt_version_id,
            id
        )
        .execute(&mut *conn)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    pub async fn delete_prompt(&self, id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM prompt
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }
}

fn generate_diff(text1: &str, text2: &str) -> String {
    let mut diff_string = String::new();
    let differences = lines(text1, text2);

    for difference in differences {
        match difference {
            DiffResult::Left(l) => diff_string.push_str(&format!("-{}\n", l)),
            DiffResult::Right(r) => diff_string.push_str(&format!("+{}\n", r)),
            _ => {}
        }
    }

    diff_string
}

