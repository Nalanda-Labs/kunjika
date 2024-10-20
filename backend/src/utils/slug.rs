pub async fn create_slug(text: &str) -> String {
    let mut slug = "".to_string();
    let mut prev_dash = false;
    for c in text.chars() {
        if (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') {
            slug.push(c);
            prev_dash = false;
        } else if c == ' ' || c == ',' || c == '.' || c == '/' || c == '\\' || c == '_' || c == '='
        {
            if (!prev_dash) && (slug.len() > 0) {
                slug.push('-');
                prev_dash = true;
            }
        }
    }
    if prev_dash {
        slug = (&slug[0..slug.len() - 1]).to_string();
    }

    slug
}
