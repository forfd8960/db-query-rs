use sqlparser::ast::{Query, SetExpr, Statement};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use crate::models::error::AppError;

/// SQL validator using sqlparser-rs
pub struct SqlValidator;

impl SqlValidator {
    /// Validate that SQL is a SELECT statement
    /// 
    /// Returns Ok(()) if valid SELECT, Err otherwise
    pub fn validate_select_only(sql: &str) -> Result<(), AppError> {
        let dialect = PostgreSqlDialect {};
        
        let statements = Parser::parse_sql(&dialect, sql)
            .map_err(|e| AppError::ValidationError(format!("SQL parse error: {}", e)))?;
        
        if statements.is_empty() {
            return Err(AppError::ValidationError("No SQL statement provided".to_string()));
        }
        
        if statements.len() > 1 {
            return Err(AppError::ValidationError(
                "Multiple statements not allowed. Only one SELECT statement permitted.".to_string()
            ));
        }
        
        match &statements[0] {
            Statement::Query(_) => Ok(()),
            _ => Err(AppError::ValidationError(
                "Only SELECT statements are allowed. Other operations (INSERT, UPDATE, DELETE, DROP, etc.) are forbidden.".to_string()
            )),
        }
    }
    
    /// Check if SQL query has a LIMIT clause
    pub fn has_limit(sql: &str) -> Result<bool, AppError> {
        let dialect = PostgreSqlDialect {};
        
        let statements = Parser::parse_sql(&dialect, sql)
            .map_err(|e| AppError::ValidationError(format!("SQL parse error: {}", e)))?;
        
        if statements.is_empty() {
            return Ok(false);
        }
        
        match &statements[0] {
            Statement::Query(query) => Ok(Self::query_has_limit(query)),
            _ => Ok(false),
        }
    }
    
    /// Check if a Query AST node has a LIMIT clause
    fn query_has_limit(query: &Query) -> bool {
        // Check top-level limit
        if query.limit.is_some() {
            return true;
        }
        
        // Check body (could be a SetOperation with nested queries)
        match &*query.body {
            SetExpr::Select(_) => false,
            SetExpr::Query(nested_query) => Self::query_has_limit(nested_query),
            SetExpr::SetOperation { left, right, .. } => {
                Self::set_expr_has_limit(left) || Self::set_expr_has_limit(right)
            }
            _ => false,
        }
    }
    
    /// Check if a SetExpr has a limit
    fn set_expr_has_limit(expr: &SetExpr) -> bool {
        match expr {
            SetExpr::Query(query) => Self::query_has_limit(query),
            SetExpr::SetOperation { left, right, .. } => {
                Self::set_expr_has_limit(left) || Self::set_expr_has_limit(right)
            }
            _ => false,
        }
    }
    
    /// Inject LIMIT 1000 if missing
    /// 
    /// Appends " LIMIT 1000" to the SQL if no LIMIT clause exists
    pub fn inject_limit_if_missing(sql: &str) -> Result<String, AppError> {
        if Self::has_limit(sql)? {
            Ok(sql.to_string())
        } else {
            // Trim trailing semicolon if present
            let trimmed = sql.trim_end_matches(';').trim();
            Ok(format!("{} LIMIT 1000", trimmed))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_select_only_valid() {
        assert!(SqlValidator::validate_select_only("SELECT * FROM users").is_ok());
        assert!(SqlValidator::validate_select_only("SELECT id, name FROM users WHERE id > 10").is_ok());
    }

    #[test]
    fn test_validate_select_only_invalid() {
        assert!(SqlValidator::validate_select_only("DELETE FROM users").is_err());
        assert!(SqlValidator::validate_select_only("UPDATE users SET name = 'test'").is_err());
        assert!(SqlValidator::validate_select_only("DROP TABLE users").is_err());
        assert!(SqlValidator::validate_select_only("INSERT INTO users VALUES (1)").is_err());
    }

    #[test]
    fn test_has_limit() {
        assert_eq!(SqlValidator::has_limit("SELECT * FROM users LIMIT 10").unwrap(), true);
        assert_eq!(SqlValidator::has_limit("SELECT * FROM users").unwrap(), false);
    }

    #[test]
    fn test_inject_limit_if_missing() {
        let sql = "SELECT * FROM users";
        let result = SqlValidator::inject_limit_if_missing(sql).unwrap();
        assert_eq!(result, "SELECT * FROM users LIMIT 1000");

        let sql_with_limit = "SELECT * FROM users LIMIT 10";
        let result = SqlValidator::inject_limit_if_missing(sql_with_limit).unwrap();
        assert_eq!(result, "SELECT * FROM users LIMIT 10");
    }

    #[test]
    fn test_inject_limit_removes_trailing_semicolon() {
        let sql = "SELECT * FROM users;";
        let result = SqlValidator::inject_limit_if_missing(sql).unwrap();
        assert_eq!(result, "SELECT * FROM users LIMIT 1000");
    }
}
