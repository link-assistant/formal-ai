### Fixed

- Connect the calendar and currency HTTP regression to its captured-publisher worker through the server's supported injection argument. Assert the actual worker identity so rate checks cannot silently depend on live publisher availability. Existing answer and derivation checks remain unchanged.
