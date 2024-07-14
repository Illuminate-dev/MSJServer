# MSJServer

## Development Environment

`cargo install cargo-watch` <br />
`cargo watch -- cargo run`

## TODO

- add session expiration
- migrate to using a db
- use or write a good templating system for html - !! subject to injection rn - maybe jira?
- add file upload for publishing
- use guids or hashes for all users
- rn because of borrow rules some cloning happens that maybe doesnt need to
- add tests
- go through expects and unwraps to make sure that they only occur when intended
- add article images
- add more editor features
