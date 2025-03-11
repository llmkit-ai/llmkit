use anyhow::Result;
use crate::db::types::tool::ToolRow;

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
        let tool = sqlx::query!(
            r#"
            INSERT INTO tool (name, tool_name, description, parameters, strict)
            VALUES (?, ?, ?, ?, ?)
            "#,
            name,
            tool_name,
            description,
            parameters,
            strict,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(tool.last_insert_rowid())
    }

    pub async fn get_tool(&self, id: i64) -> Result<ToolRow> {
        let tool = sqlx::query_as!(
            ToolRow,
            r#"
            SELECT id, name, tool_name, description, parameters, strict, created_at, updated_at
            FROM tool
            WHERE id = ?
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

    pub async fn list_tools(&self) -> Result<Vec<ToolRow>> {
        let tools = sqlx::query_as!(
            ToolRow,
            r#"
            SELECT id, name, tool_name, description, parameters, strict, created_at, updated_at
            FROM tool
            ORDER BY name
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
        let affected = sqlx::query!(
            r#"
            UPDATE tool
            SET name = ?,
                tool_name = ?,
                description = ?,
                parameters = ?,
                strict = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            name,
            tool_name,
            description,
            parameters,
            strict,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(affected > 0)
    }

    pub async fn delete_tool(&self, id: i64) -> Result<bool> {
        // First delete any tool associations
        sqlx::query!(
            r#"
            DELETE FROM prompt_version_tool_access
            WHERE tool_id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        // Then delete the tool itself
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM tool
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    pub async fn get_tool_by_name(&self, name: &str) -> Result<ToolRow> {
        let tool = sqlx::query_as!(
            ToolRow,
            r#"
            SELECT id, name, tool_name, description, parameters, strict, created_at, updated_at
            FROM tool
            WHERE name = ?
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

    // Methods for managing tool to prompt version relationships
    
    pub async fn associate_tool_with_prompt_version(
        &self,
        tool_id: i64,
        prompt_version_id: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO prompt_version_tool_access (prompt_version_id, tool_id)
            VALUES (?, ?)
            "#,
            prompt_version_id,
            tool_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn remove_tool_prompt_version_association(
        &self,
        tool_id: i64,
        prompt_version_id: i64,
    ) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM prompt_version_tool_access
            WHERE prompt_version_id = ? AND tool_id = ?
            "#,
            prompt_version_id,
            tool_id
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("result: {:?}", result);
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn get_prompt_versions_by_tool(
        &self,
        tool_id: i64,
    ) -> Result<Vec<i64>> {
        let rows = sqlx::query!(
            r#"
            SELECT prompt_version_id
            FROM prompt_version_tool_access
            WHERE tool_id = ?
            "#,
            tool_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| row.prompt_version_id).collect())
    }
    
    pub async fn get_tools_by_prompt_version(
        &self,
        prompt_version_id: i64,
    ) -> Result<Vec<ToolRow>> {
        let tools = sqlx::query_as!(
            ToolRow,
            r#"
            SELECT t.id, t.name, t.tool_name, t.description, t.parameters, t.strict, t.created_at, t.updated_at
            FROM tool t
            JOIN prompt_version_tool_access pvta ON t.id = pvta.tool_id
            WHERE pvta.prompt_version_id = ?
            "#,
            prompt_version_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(tools)
    }
}
