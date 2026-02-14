pub fn mask_email(email: &str) -> String {
    let Some((local, domain)) = email.split_once('@') else {
        return "***".to_string();
    };

    if local.is_empty() || domain.is_empty() {
        return "***".to_string();
    }

    let mut chars = local.chars();
    let first = chars.next().unwrap_or('*');
    format!("{first}***@{domain}")
}
