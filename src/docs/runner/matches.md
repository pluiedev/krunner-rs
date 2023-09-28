Returns a list of matches corresponding to the given query.

This is the core method to implement for your runner, and where all the matching logic lies.

Note that this method would not be called for any match that didn't pass through the [filter](crate::Config::match_filter) — if queries that should trigger the runner don't find their way to this method, you might want to check if your filter is functioning correctly.
