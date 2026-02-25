fn check_change(ctx) {
    if ctx.kind == "deletion" {
        return violation("Deletion forbidden: " + ctx.path);
    }
}
