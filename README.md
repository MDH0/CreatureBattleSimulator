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
That instance needs a namespace user set up.
The application expects Rocket environment variables to be set.
More explicitly: username, password and db_url.
See https://rocket.rs/guide/v0.5/configuration/ on how to do this.

## Testing
Tests are being run with `cargo test`.

Because the project is using the testcontainers crate for the tests, there might be additional setup needed before you are able to run the tests.


## Local setup

Create a file called Rocket.toml with the following content:

```
[debug]
username  = "root"
password  = "secret"
db_url    = "127.0.0.1:8001"
```

start the database:

```
docker volume create dev-surreal-db
docker run --detach --restart always --name surrealdb -p 127.0.0.1:8001:8000 --user root -v dev-surreal-db:/database surrealdb/surrealdb:v1.5.4 start --user root --pass secret --log trace file://database
```

start the server:

```
cargo run
```

Access the server using `curl`:

```
$ curl -X POST  http://localhost:8000/games/
{"trace_id":"e22853ac-5f0f-4b20-9c2c-ea6c87a59199","game_id":"fpsnkr93wvydxzkn1gt7","state":"Pending"}


$ curl -X GET  http://localhost:8000/games/fpsnkr93wvydxzkn1gt7
{"trace_id":"fcfe4a30-b52b-4254-ad03-a16ff2082842","game_status":"Pending"}


$ curl -X PUT  http://localhost:8000/games/fpsnkr93wvydxzkn1gt7
{"trace_id":"25a27b10-b2bc-48dc-8247-5774dd14bc5a","message":"Joined the game."}


$ curl -X GET  http://localhost:8000/games/fpsnkr93wvydxzkn1gt7
{"trace_id":"d94a3ff9-9e51-4b5a-b974-157e384e0138","game_status":"Ongoing"}
```
