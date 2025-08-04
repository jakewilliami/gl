use super::config;

#[derive(Clone)]
pub struct GitIdentity {
    pub email: String,
    pub names: Vec<String>,
}

impl GitIdentity {
    pub fn is_me(&self) -> bool {
        if config::ME_IDENTITY.contains(&self.email.as_str()) {
            return true;
        }

        self.names
            .iter()
            .any(|name| config::ME_IDENTITY.contains(&name.as_str()))
    }
}
