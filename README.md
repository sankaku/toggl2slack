# toggl2slack
Fetch toggl report and send it to Slack.

## setting
```sh
cp .env.template .env
```

and edit `.env` file.

## build
```sh
cargo build --release
```

## run
[TODO] .env file?

Date format is `YYYY-MM-DD`.

```sh
./target/release/toggl2slack --date_from=2021-01-01 --date_to=2021-01-03
```

## Docker
### build
```sh
docker build . -t toggl2slack
```

### run
Date format is `YYYY-MM-DD`.

```sh
docker run --rm --env-file=.env toggl2slack /app/toggl2slack --date_from=2021-01-01 --date_to=2021-01-03
```
