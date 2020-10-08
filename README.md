# toggl2slack
Fetch toggl report and send it to Slack.

## build
```sh
cargo build --release
```

## run
```sh
./target/release/toggl2slack --date_from=2020-10-01 --date_to=2020-10-07 --toggl_token=<TOGGL_TOKEN> --workspace=<WORKSPACE_ID> --toggl_email=<TOGGL_EMAIL>
```
