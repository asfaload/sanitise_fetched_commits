fn check_change(ctx) {
    if ctx.kind == "deletion" {
        return;
    }
    if glob_match("**/*.json", ctx.path) {
        if !validate_json(ctx.content) {
            return violation("Content validation failed in " + ctx.path + ": Invalid JSON in " + ctx.path);
        }
    }
    if glob_match("**/*.csv", ctx.path) {
        if !validate_csv(ctx.content) {
            return violation("Content validation failed in " + ctx.path + ": Invalid CSV in " + ctx.path);
        }
    }
}
