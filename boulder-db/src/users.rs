use crate::errors::DatabaseError;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub username: String,
    pub password: String,
    pub role: Role,
}

impl User {
    pub fn role(&self) -> Role {
        self.role.clone()
    }

    pub fn grant_user_role(mut self, role: Role) -> Result<(), DatabaseError> {
        if self.role == role {
            return Err(DatabaseError::RoleAlreadyExists);
        }

        self.role = role;
        Ok(())
    }

    pub fn revoke_user_role(mut self, _role: Role) -> Result<(), DatabaseError> {
        self.role = Role::Guest;

        Ok(())
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop, Serialize, Deserialize, PartialEq, Debug, sqlx::Type)]
#[sqlx(type_name = "role")]
#[sqlx(rename_all = "lowercase")]
pub enum Role {
    Guest,
    User,
    Editor,
    AlmostRoot,
    Root,
}
