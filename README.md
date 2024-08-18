# CreatureBattleSimulator
As you might have guessed, the name is very temporary and will change in the future.

## Motivation
I really enjoyed playing competitive Pokemon.
But playing the most recent game (Scarlet and Violet) and seeing all the power creep happening, it didn't feel as fun anymore.
So I decided to create my own version.
This way, we have the potential, to create a better competitive integrity, by applying regular balance changes and possibly the willingness, to change core mechanics.

## Contribution
While the project is currently in a starting phase, all contributions, from ideas to pull requests are welcomed.

## Building
It is as easy as running `cargo build`.

## Deploying

If you (for some reason) want to deploy it, make sure you have a SurrealDB instance running somewhere.
That instance needs a namespace use set up.
The application expects Rocket environment variables to be set.
More explicitly: username, password and db_url.
See https://rocket.rs/guide/v0.5/configuration/ on how to do this.

## Testing
Tests are being run with `cargo test`.

Because the project is using the testcontainers crate for the tests, there might be additional setup needed before you are able to run the tests.