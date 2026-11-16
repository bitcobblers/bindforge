pub fn normalize_slug(display_name: &str) -> String {
    let mut slug = display_name.to_lowercase();
    
    // Replace all non-alphanumeric characters with '-'
    let mut result = String::new();
    for c in slug.chars() {
        if c.is_alphanumeric() {
            result.push(c);
        } else {
            result.push('-');
        }
    }
    slug = result;

    // Collapse consecutive hyphens
    let mut final_result = String::new();
    let mut last_was_hyphen = false;
    for c in slug.chars() {
        if c == '-' {
            if !last_was_hyphen {
                final_result.push(c);
                last_was_hyphen = true;
            }
        } else {
            final_result.push(c);
            last_was_hyphen = false;
        }
    }
    slug = final_result;

    // Trim hyphens from start and end
    slug = slug.trim_matches('-').to_string();

    format!("unknown/{}", slug)
}
