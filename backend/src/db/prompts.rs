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
        json_schema: Option<&str>,
        prompt_type: &str,
        is_chat: bool,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;

        // 1. insert the prompt row with a null current_prompt_version_id
        let prompt = sqlx::query!(
            r#"
            INSERT INTO prompt (key, current_prompt_version_id)
            VALUES (?, ?)
            "#,
            key,
            Option::<i64>::None,
        )
        .execute(&mut *conn)
        .await?;
        let prompt_id = prompt.last_insert_rowid();

        // new prompt -> version 1
        let next_version = 1;

        // 2. insert the prompt_version row with prompt_id ref
        let prompt_version = sqlx::query!(
            r#"
            INSERT INTO prompt_version (
                prompt_id,
                version_number,
                system_diff,
                user_diff,
                system,
                user,
                model_id,
                max_tokens,
                temperature,
                json_mode,
                json_schema,
                prompt_type,
                is_chat
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            prompt_id,
            next_version,
            "", // initial system_diff is empty
            "", // initial user_diff is empty
            system_prompt,
            user_prompt,
            model_id,
            max_tokens,
            temperature,
            json_mode,
            json_schema,
            prompt_type,
            is_chat
        )
        .execute(&mut *conn)
        .await?;
        let prompt_version_id = prompt_version.last_insert_rowid();

        // 3. update the prompt with the current_prompt_version_id
        sqlx::query!(
            r#"
            UPDATE prompt
            SET current_prompt_version_id = ?
            WHERE id = ?
            "#,
            prompt_version_id,
            prompt_id
        )
        .execute(&mut *conn)
        .await?;

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
                pv.json_schema,
                pv.prompt_type,
                pv.is_chat,
                m.name as model_name,
                pr.name as provider_name,
                pr.base_url as provider_base_url,
                m.supports_json,
                m.supports_json_schema,
                m.supports_tools,
                pv.system_diff,
                pv.user_diff,
                pv.version_number,
                pv.id as version_id,
                pv.created_at,
                pv.updated_at
            FROM prompt p
            JOIN prompt_version pv ON p.current_prompt_version_id = pv.id
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
                pv.json_schema,
                pv.prompt_type,
                pv.is_chat,
                m.name as model_name,
                pr.name as provider_name,
                pr.base_url as provider_base_url,
                m.supports_json,
                m.supports_json_schema,
                m.supports_tools,
                pv.system_diff,
                pv.user_diff,
                pv.version_number,
                pv.id as version_id,
                pv.created_at,
                pv.updated_at
            FROM prompt p
            JOIN prompt_version pv ON p.current_prompt_version_id = pv.id
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
        json_schema: Option<&str>,
        prompt_type: &str,
        is_chat: bool,
    ) -> Result<bool> {
        let mut conn = self.pool.acquire().await?;

        // 1. fetch current prompt to compute diffs
        let current_prompt = self.get_prompt(id).await?;
        let current_user_prompt = current_prompt.user.unwrap_or("".to_string());

        let system_diff = generate_diff(&current_prompt.system, system_prompt);
        let user_diff = generate_diff(&current_user_prompt, user_prompt);

        // 2. get the latest version number for THIS prompt (using prompt_id)
        let latest_version: Option<i64> = sqlx::query!(
            r#"
            SELECT MAX(version_number) as version_number
            FROM prompt_version
            WHERE prompt_id = ?
            "#,
            id
        )
        .fetch_optional(&mut *conn)
        .await?
        .and_then(|row| row.version_number);
        let next_version = latest_version.map(|v| v + 1).unwrap_or(1);

        // 3. insert a new prompt_version referencing the prompt via prompt_id
        let prompt_version = sqlx::query!(
            r#"
            INSERT INTO prompt_version (
                prompt_id,
                version_number,
                system_diff,
                user_diff,
                system,
                user,
                model_id,
                max_tokens,
                temperature,
                json_mode,
                json_schema,
                prompt_type,
                is_chat
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            next_version,
            system_diff,
            user_diff,
            system_prompt,
            user_prompt,
            model_id,
            max_tokens,
            temperature,
            json_mode,
            json_schema,
            prompt_type,
            is_chat
        )
        .execute(&mut *conn)
        .await?;
        let prompt_version_id = prompt_version.last_insert_rowid();

        // 4. update the prompt row with the new key and new current_prompt_version_id
        let affected = sqlx::query!(
            r#"
            UPDATE prompt
            SET key = ?,
                current_prompt_version_id = ?
            WHERE id = ?
            "#,
            key,
            prompt_version_id,
            id
        )
        .execute(&mut *conn)
        .await?
        .rows_affected();

        Ok(affected > 0)
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
    
    pub async fn get_prompt_by_key(&self, key: &str) -> Result<PromptRowWithModel> {
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
                pv.json_schema,
                pv.prompt_type,
                pv.is_chat,
                m.name as model_name,
                pr.name as provider_name,
                pr.base_url as provider_base_url,
                m.supports_json,
                m.supports_json_schema,
                m.supports_tools,
                pv.system_diff,
                pv.user_diff,
                pv.version_number,
                pv.id as version_id,
                pv.created_at,
                pv.updated_at
            FROM prompt p
            JOIN prompt_version pv ON p.current_prompt_version_id = pv.id
            JOIN model m ON pv.model_id = m.id
            JOIN provider pr ON m.provider_id = pr.id
            WHERE p.key = ?
            "#,
            key
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(prompt)
    }

    pub async fn get_prompt_versions(&self, prompt_id: i64) -> Result<Vec<PromptRowWithModel>> {
        let versions = sqlx::query_as!(
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
                pv.json_schema,
                pv.prompt_type,
                pv.is_chat,
                m.name as model_name,
                pr.name as provider_name,
                pr.base_url as provider_base_url,
                m.supports_json,
                m.supports_json_schema,
                m.supports_tools,
                pv.system_diff,
                pv.user_diff,
                pv.version_number,
                pv.id as version_id,
                pv.created_at,
                pv.updated_at
            FROM prompt_version pv
            JOIN prompt p ON pv.prompt_id = p.id
            JOIN model m ON pv.model_id = m.id
            JOIN provider pr ON m.provider_id = pr.id
            WHERE pv.prompt_id = ?
            ORDER BY pv.version_number DESC
            "#,
            prompt_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(versions)
    }

    pub async fn set_active_prompt_version(&self, prompt_id: i64, version_id: i64) -> Result<PromptRowWithModel> {
        // Verify the version belongs to this prompt
        let version_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM prompt_version
            WHERE id = ? AND prompt_id = ?
            "#,
            version_id,
            prompt_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        if version_count == 0 {
            anyhow::bail!("Version not found or does not belong to this prompt");
        }

        // Update the prompt's current version
        let affected = sqlx::query!(
            r#"
            UPDATE prompt
            SET current_prompt_version_id = ?
            WHERE id = ?
            "#,
            version_id,
            prompt_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            anyhow::bail!("Prompt not found");
        }

        // Return the updated prompt
        self.get_prompt(prompt_id).await
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

