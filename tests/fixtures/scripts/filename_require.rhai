fn check_change(ctx) {
    if glob_match("**/CHANGELOG.md", ctx.path) {
        this.found = true;
        return pass("Path matches required pattern: " + ctx.path);
    }
}

fn finalize() {
    if this.found != true {
        return violation("Required file pattern not found in commit. Expected one of: [\"**/CHANGELOG.md\"]");
    }
}
