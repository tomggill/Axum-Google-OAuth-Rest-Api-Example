use std::sync::Arc;

use crate::{config::database::Database, error::app_error::AppError, repository::user_repository::{UserRepository, UserRepositoryTrait}, state::app_state::UserContext, User};


#[derive(Clone)]
pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_repository: UserRepository::new(db_conn),
        }
    }

    pub async fn find_or_insert_user(&self, user_data: &User) -> Result<UserContext, AppError> {
        let existing_user = self.user_repository.find_user_by_google_id(&user_data.sub).await?;
        if let Some(user_context) = existing_user {
            return Ok(user_context);
        }
    
        let user_id = self.user_repository.add_user(
                &user_data.sub,
                &user_data.email, 
                &user_data.given_name, 
                &user_data.family_name).await?;
    
        Ok(UserContext {
            user_id,
            email: user_data.email.clone(),
            name: user_data.given_name.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sqlx::MySqlPool;

    use crate::state::app_state::UserContext;
    use crate::User;
    use crate::config::database::Database;

    use super::UserService;

    async fn get_user_service(db: MySqlPool) -> UserService {
        let db_conn = Database { pool: db };
        UserService::new(&Arc::new(db_conn))
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/users.sql"))]
    async fn test_find_or_insert_user_for_existing_user(db: MySqlPool) {
        let user_service = get_user_service(db).await;

        let test_user = User {
            sub: "110235950686105464135".to_string(),
            given_name: "Tom".to_string(),
            family_name: "Gill".to_string(),
            email: "TestEmail@lift.com".to_string(),
        };

        let result = user_service.find_or_insert_user(&test_user).await;

        let expected_user_context = UserContext {
            user_id: 1,
            email: "TestEmail@lift.com".to_string(),
            name: "Tom".to_string(),
        };

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_user_context);
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/users.sql"))]
    async fn test_find_or_insert_user_for_new_user(db: MySqlPool) {
        let user_service = get_user_service(db).await;

        let test_user = User {
            sub: "897239842378324289342".to_string(),
            given_name: "George".to_string(),
            family_name: "Thomas".to_string(),
            email: "gt@lift.com".to_string(),
        };

        let result = user_service.find_or_insert_user(&test_user).await;

        let expected_user_context = UserContext {
            user_id: 3,
            email: "gt@lift.com".to_string(),
            name: "George".to_string(),
        };

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_user_context);
    }
}
