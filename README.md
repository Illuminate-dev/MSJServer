# MSJServer

## Development Environment

`cargo install cargo-watch` <br />
`cargo watch -- cargo run`

## TODO

- migrate to using a db
- use or write a good templating system for html - !! subject to injection rn (wait actually maybe not) - maybe jira?
- add file upload for publishing (like docx or something)
- use guids or hashes for all users
- rn because of borrow rules some cloning happens that maybe doesnt need to + some code is repetitive and can be optimized
- add tests
- go through expects and unwraps to make sure that they only occur when intended
- add article images
- add more editor features
- add editor review system & perms
- turn authorization into middleware
- rework the enter page system cause it's kinda weird
- make sure usernames are case insensitive
- fix search sorting (or just wait until migrating to a db)
