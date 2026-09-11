/// Built-in main-category colors. First 19 match the seeded / overflow list in ui.md.
pub const CATEGORY_PALETTE: &[&str] = &[
    "#5b7c99", "#b4532a", "#78716c", "#a16207", "#0e7490", "#4f46e5", "#3f6212",
    "#9f1239", "#52525b", "#be185d", "#0f766e", "#57534e", "#1e3a5f", "#6b7280",
    "#854d0e", "#115e59", "#6b21a8", "#9a3412", "#164e63", "#1d4e89", "#3b6ea5",
    "#64748b", "#334155", "#0c4a6e", "#0369a1", "#155e75", "#134e4a", "#0d7377",
    "#365314", "#4d7c0f", "#166534", "#14532d", "#3f5d4a", "#92400e", "#78350f",
    "#8a7040", "#c2410c", "#7f1d1d", "#991b1b", "#9d174d", "#831843", "#5b21b6",
    "#4c1d95", "#4338ca", "#3730a3", "#44403c", "#7c2d12", "#451a03", "#3f3f46",
    "#71717a", "#1e293b", "#1e40af", "#1d4ed8", "#0f172a", "#312e81", "#701a75",
    "#86198f", "#6b3f2a", "#2f4f4f", "#556b2f", "#8b5e3c", "#4a5568", "#2c5282",
    "#9c4221",
];

pub fn all() -> Vec<String> {
    CATEGORY_PALETTE.iter().map(|s| (*s).to_string()).collect()
}

pub fn is_palette_color(hex: &str) -> bool {
    CATEGORY_PALETTE
        .iter()
        .any(|c| c.eq_ignore_ascii_case(hex.trim()))
}

pub fn next_unused(used: &[String]) -> String {
    CATEGORY_PALETTE
        .iter()
        .find(|c| !used.iter().any(|u| u.eq_ignore_ascii_case(c)))
        .copied()
        .unwrap_or("#6b7280")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn palette_is_64_unique() {
        assert_eq!(CATEGORY_PALETTE.len(), 64);
        let set: HashSet<_> = CATEGORY_PALETTE.iter().map(|c| c.to_ascii_lowercase()).collect();
        assert_eq!(set.len(), 64);
    }

    #[test]
    fn next_unused_skips_taken() {
        let used = vec!["#5b7c99".into(), "#B4532A".into()];
        assert_eq!(next_unused(&used), "#78716c");
        let all_used: Vec<String> = CATEGORY_PALETTE.iter().map(|s| (*s).to_string()).collect();
        assert_eq!(next_unused(&all_used), "#6b7280");
    }
}
