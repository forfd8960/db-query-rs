use chrono::{DateTime, Utc};

/// Database connection model (persisted in SQLite)
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub id: i64,
    pub name: String,
    pub connection_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DatabaseConnection {
    /// Mask the password in the connection URL for display purposes
    /// 
    /// Replaces password with "***" in PostgreSQL connection URLs
    /// Format: postgresql://user:password@host:port/db -> postgresql://user:***@host:port/db
    pub fn mask_password(&self) -> String {
        mask_connection_url(&self.connection_url)
    }
}

/// Mask password in a PostgreSQL connection URL
pub fn mask_connection_url(url: &str) -> String {
    // Parse URL to find password section
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            // Check if there's a protocol separator before the colon
            if let Some(proto_end) = url.find("://") {
                if colon_pos > proto_end + 3 {
                    // Found password section: replace it with ***
                    let mut masked = String::from(&url[..=colon_pos]);
                    masked.push_str("***");
                    masked.push_str(&url[at_pos..]);
                    return masked;
                }
            }
        }
    }
    // If URL doesn't match expected pattern, return as-is
    url.to_string()
}

/// Extract database name from PostgreSQL connection URL
/// 
/// Extracts the database name from the URL path
/// Format: postgresql://user:password@host:port/dbname -> "dbname"
pub fn extract_db_name(url: &str) -> Result<String, String> {
    // Find the last '/' which should precede the database name
    if let Some(slash_pos) = url.rfind('/') {
        let db_name = &url[slash_pos + 1..];
        
        // Remove query parameters if present
        let db_name = if let Some(query_pos) = db_name.find('?') {
            &db_name[..query_pos]
        } else {
            db_name
        };
        
        if !db_name.is_empty() {
            return Ok(db_name.to_string());
        }
    }
    
    Err("Invalid PostgreSQL URL: cannot extract database name".to_string())
}

/// Validate PostgreSQL connection URL format
pub fn validate_postgres_url(url: &str) -> Result<(), String> {
    // Check for postgresql:// or postgres:// scheme
    if !url.starts_with("postgresql://") && !url.starts_with("postgres://") {
        return Err("URL must start with postgresql:// or postgres://".to_string());
    }
    
    // Check for @ symbol (indicates host section)
    if !url.contains('@') {
        return Err("URL must contain '@' (user@host)".to_string());
    }
    
    // Validate database name can be extracted
    extract_db_name(url)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let url = "postgresql://user:secretpass@localhost:5432/mydb";
        let masked = mask_connection_url(url);
        assert_eq!(masked, "postgresql://user:***@localhost:5432/mydb");
    }

    #[test]
    fn test_mask_password_no_password() {
        let url = "postgresql://user@localhost:5432/mydb";
        let masked = mask_connection_url(url);
        assert_eq!(masked, "postgresql://user@localhost:5432/mydb");
    }

    #[test]
    fn test_extract_db_name() {
        let url = "postgresql://user:pass@localhost:5432/testdb";
        assert_eq!(extract_db_name(url).unwrap(), "testdb");
    }

    #[test]
    fn test_extract_db_name_with_query() {
        let url = "postgresql://user:pass@localhost:5432/testdb?ssl=true";
        assert_eq!(extract_db_name(url).unwrap(), "testdb");
    }

    #[test]
    fn test_validate_postgres_url_valid() {
        assert!(validate_postgres_url("postgresql://user:pass@localhost:5432/db").is_ok());
        assert!(validate_postgres_url("postgres://user:pass@localhost:5432/db").is_ok());
    }

    #[test]
    fn test_validate_postgres_url_invalid() {
        assert!(validate_postgres_url("mysql://user:pass@localhost:3306/db").is_err());
        assert!(validate_postgres_url("postgresql://localhost:5432/db").is_err()); // Missing @
        assert!(validate_postgres_url("postgresql://user@localhost:5432/").is_err()); // No DB name
    }
}
