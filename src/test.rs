#[cfg(test)]
mod tests {
    use crate::{LineType, parse_diff};

    #[test]
    fn test_parse_diff() {
        let sample_diff = r#"diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1,5 +1,6 @@
-This is the original file.
+This is the MODIFIED file.
 It has multiple lines.
-Some content here.
+Some NEW content here.
 More content.
+Additional line added.
 Final line."#;

        let files = parse_diff(sample_diff);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "test.txt");
        assert_eq!(files[0].status, 'M');

        // Check that we have the right number of lines
        assert!(files[0].lines.len() > 0);

        // Check for different line types
        let has_added = files[0]
            .lines
            .iter()
            .any(|l| l.line_type == LineType::Added);
        let has_removed = files[0]
            .lines
            .iter()
            .any(|l| l.line_type == LineType::Removed);
        let has_context = files[0]
            .lines
            .iter()
            .any(|l| l.line_type == LineType::Context);

        assert!(has_added, "Should have added lines");
        assert!(has_removed, "Should have removed lines");
        assert!(has_context, "Should have context lines");
    }

    #[test]
    fn test_parse_real_git_diff() {
        let real_diff = r#"diff --git test.txt test.txt
index 6643ba4..0b4147a 100644
--- test.txt
+++ test.txt
@@ -1,5 +1,6 @@
-This is the original file.
+This is the MODIFIED file.
 It has multiple lines.
-Some content here.
+Some NEW content here.
 More content.
+Additional line added.
 Final line."#;

        let files = parse_diff(real_diff);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "test.txt");

        // Verify we parsed the correct line types
        let added_lines: Vec<_> = files[0]
            .lines
            .iter()
            .filter(|l| l.line_type == LineType::Added)
            .collect();
        let removed_lines: Vec<_> = files[0]
            .lines
            .iter()
            .filter(|l| l.line_type == LineType::Removed)
            .collect();

        assert!(added_lines.len() >= 3, "Should have at least 3 added lines");
        assert!(
            removed_lines.len() >= 2,
            "Should have at least 2 removed lines"
        );

        // Verify content of some lines
        assert!(added_lines.iter().any(|l| l.content.contains("MODIFIED")));
        assert!(
            added_lines
                .iter()
                .any(|l| l.content.contains("NEW content"))
        );
        assert!(
            added_lines
                .iter()
                .any(|l| l.content.contains("Additional line"))
        );

        assert!(
            removed_lines
                .iter()
                .any(|l| l.content.contains("original file"))
        );
        assert!(
            removed_lines
                .iter()
                .any(|l| l.content.contains("Some content here"))
        );
    }
}
