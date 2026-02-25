fn check_change(ctx) {
    if !glob_match("**/*.protected", ctx.path) && !glob_match("**/protected/**", ctx.path) {
        return;
    }
    if ctx.kind == "deletion" {
        return violation("File deleted (all lines removed): " + ctx.path);
    }
    if ctx.deleted_lines.len() > 0 {
        return violation("Lines deleted in protected file: " + ctx.path);
    }
}
