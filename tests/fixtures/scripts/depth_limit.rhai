fn check_change(ctx) {
    if ctx.kind == "deletion" {
        return;
    }
    let parts = ctx.path.split("/");
    for i in range(0, parts.len()) {
        let segment = parts[i];
        if (segment == "my-dir" || segment == "test-dir") && i >= 3 {
            return violation("Folder '" + segment + "' too deep (depth " + (i + 1) + "): " + ctx.path);
        }
    }
}
