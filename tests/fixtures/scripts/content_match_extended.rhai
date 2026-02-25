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
    if glob_match("**/*.yaml", ctx.path) || glob_match("**/*.yml", ctx.path) {
        if !validate_yaml(ctx.content) {
            return violation("Content validation failed in " + ctx.path + ": Invalid YAML in " + ctx.path);
        }
    }
    if glob_match("**/*.toml", ctx.path) {
        if !validate_toml(ctx.content) {
            return violation("Content validation failed in " + ctx.path + ": Invalid TOML in " + ctx.path);
        }
    }
}
