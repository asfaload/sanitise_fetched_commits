fn check_change(ctx) {
    if ctx.kind == "deletion" {
        return;
    }
    if glob_match("**/*.tmp", ctx.path) || glob_match("**/tmp/**", ctx.path) || glob_match("**/temp/**", ctx.path) {
        return violation("Path matches forbidden pattern: " + ctx.path);
    }
}
