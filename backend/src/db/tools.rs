use anyhow::Result;
use crate::db::types::tool::{ToolRow, ToolVersionRow, ToolWithVersion};

#[derive(Clone, Debug)]
pub struct ToolRepository {
    pool: sqlx::SqlitePool,
}

impl ToolRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(ToolRepository { pool })
    }

    #[cfg(test)]
    pub async fn in_memory(pool: sqlx::SqlitePool) -> Result<Self> {
        Self::new(pool.clone()).await
    }

    pub async fn create_tool(
        &self,
        name: &str,
        tool_name: &str,
        description: &str,
        parameters: &str,
        strict: bool,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;

        // 1. Insert the tool row with a null current_tool_version_id
        let tool = sqlx::query!(
            r#"
            INSERT INTO tool (name, current_tool_version_id)
            VALUES (?, ?)
            "#,
            name,
            Option::<i64>::None,
        )
        .execute(&mut *conn)
        .await?;
        let tool_id = tool.last_insert_rowid();

        // New tool -> version 1
        let next_version = 1;

        // 2. Insert the tool_version row with tool_id ref
        let tool_version = sqlx::query!(
            r#"
            INSERT INTO tool_version (
                tool_id,
                version_number,
                tool_name,
                description,
                parameters,
                strict
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            tool_id,
            next_version,
            tool_name,
            description,
            parameters,
            strict
        )
        .execute(&mut *conn)
        .await?;
        let tool_version_id = tool_version.last_insert_rowid();

        // 3. Update the tool with the current_tool_version_id
        sqlx::query!(
            r#"
            UPDATE tool
            SET current_tool_version_id = ?
            WHERE id = ?
            "#,
            tool_version_id,
            tool_id
        )
        .execute(&mut *conn)
        .await?;

        Ok(tool_id)
    }

    pub async fn get_tool(&self, id: i64) -> Result<ToolWithVersion> {
        let tool = sqlx::query_as!(
            ToolWithVersion,
            r#"
            SELECT
                t.id,
                t.name,
                t.current_tool_version_id,
                tv.version_number,
                tv.tool_name,
                tv.description,
                tv.parameters,
                tv.strict,
                tv.id as version_id,
                t.created_at,
                t.updated_at
            FROM tool t
            JOIN tool_version tv ON t.current_tool_version_id = tv.id
            WHERE t.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        match tool {
            Some(t) => Ok(t),
            None => anyhow::bail!("Tool not found"),
        }
    }

    pub async fn list_tools(&self) -> Result<Vec<ToolWithVersion>> {
        let tools = sqlx::query_as!(
            ToolWithVersion,
            r#"
            SELECT
                t.id,
                t.name,
                t.current_tool_version_id,
                tv.version_number,
                tv.tool_name,
                tv.description,
                tv.parameters,
                tv.strict,
                tv.id as version_id,
                t.created_at,
                t.updated_at
            FROM tool t
            JOIN tool_version tv ON t.current_tool_version_id = tv.id
            ORDER BY t.name
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tools)
    }

    pub async fn update_tool(
        &self,
        id: i64,
        name: &str,
        tool_name: &str,
        description: &str,
        parameters: &str,
        strict: bool,
    ) -> Result<bool> {
        let mut conn = self.pool.acquire().await?;

        // 1. Get the latest version number for THIS tool (using tool_id)
        let latest_version: Option<i64> = sqlx::query!(
            r#"
            SELECT MAX(version_number) as version_number
            FROM tool_version
            WHERE tool_id = ?
            "#,
            id
        )
        .fetch_optional(&mut *conn)
        .await?
        .and_then(|row| row.version_number);
        let next_version = latest_version.map(|v| v + 1).unwrap_or(1);

        // 2. Insert a new tool_version referencing the tool via tool_id
        let tool_version = sqlx::query!(
            r#"
            INSERT INTO tool_version (
                tool_id,
                version_number,
                tool_name,
                description,
                parameters,
                strict
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            id,
            next_version,
            tool_name,
            description,
            parameters,
            strict
        )
        .execute(&mut *conn)
        .await?;
        let tool_version_id = tool_version.last_insert_rowid();

        // 3. Update the tool row with the new name and new current_tool_version_id
        let affected = sqlx::query!(
            r#"
            UPDATE tool
            SET name = ?,
                current_tool_version_id = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            name,
            tool_version_id,
            id
        )
        .execute(&mut *conn)
        .await?
        .rows_affected();

        Ok(affected > 0)
    }

    pub async fn delete_tool(&self, id: i64) -> Result<bool> {
        let mut conn = self.pool.acquire().await?;

        // First delete all tool versions
        sqlx::query!(
            r#"
            DELETE FROM tool_version
            WHERE tool_id = ?
            "#,
            id
        )
        .execute(&mut *conn)
        .await?;

        // Then delete the tool itself
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM tool
            WHERE id = ?
            "#,
            id
        )
        .execute(&mut *conn)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    pub async fn get_tool_versions(&self, tool_id: i64) -> Result<Vec<ToolVersionRow>> {
        let versions = sqlx::query_as!(
            ToolVersionRow,
            r#"
            SELECT id, tool_id, version_number, tool_name, description, parameters, strict, created_at
            FROM tool_version
            WHERE tool_id = ?
            ORDER BY version_number DESC
            "#,
            tool_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(versions)
    }

    pub async fn get_tool_version(&self, version_id: i64) -> Result<ToolVersionRow> {
        let version = sqlx::query_as!(
            ToolVersionRow,
            r#"
            SELECT id, tool_id, version_number, tool_name, description, parameters, strict, created_at
            FROM tool_version
            WHERE id = ?
            "#,
            version_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match version {
            Some(v) => Ok(v),
            None => anyhow::bail!("Tool version not found"),
        }
    }

    pub async fn get_tool_by_name(&self, name: &str) -> Result<ToolWithVersion> {
        let tool = sqlx::query_as!(
            ToolWithVersion,
            r#"
            SELECT
                t.id,
                t.name,
                t.current_tool_version_id,
                tv.version_number,
                tv.tool_name,
                tv.description,
                tv.parameters,
                tv.strict,
                tv.id as version_id,
                t.created_at,
                t.updated_at
            FROM tool t
            JOIN tool_version tv ON t.current_tool_version_id = tv.id
            WHERE t.name = ?
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        match tool {
            Some(t) => Ok(t),
            None => anyhow::bail!("Tool not found"),
        }
    }

    // Methods for managing tool version to prompt version relationships
    
    pub async fn associate_tool_version_with_prompt_version(
        &self,
        tool_version_id: i64,
        prompt_version_id: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO prompt_version_tool_access (prompt_version_id, tool_version_id)
            VALUES (?, ?)
            "#,
            prompt_version_id,
            tool_version_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn remove_tool_version_prompt_version_association(
        &self,
        tool_version_id: i64,
        prompt_version_id: i64,
    ) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM prompt_version_tool_access
            WHERE prompt_version_id = ? AND tool_version_id = ?
            "#,
            prompt_version_id,
            tool_version_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn get_prompt_versions_by_tool_version(
        &self,
        tool_version_id: i64,
    ) -> Result<Vec<i64>> {
        let rows = sqlx::query!(
            r#"
            SELECT prompt_version_id
            FROM prompt_version_tool_access
            WHERE tool_version_id = ?
            "#,
            tool_version_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| row.prompt_version_id).collect())
    }
    
    pub async fn get_tool_versions_by_prompt_version(
        &self,
        prompt_version_id: i64,
    ) -> Result<Vec<ToolVersionRow>> {
        let tool_versions = sqlx::query_as!(
            ToolVersionRow,
            r#"
            SELECT tv.id, tv.tool_id, tv.version_number, tv.tool_name, tv.description, tv.parameters, tv.strict, tv.created_at
            FROM tool_version tv
            JOIN prompt_version_tool_access pvta ON tv.id = pvta.tool_version_id
            WHERE pvta.prompt_version_id = ?
            "#,
            prompt_version_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(tool_versions)
    }
}
