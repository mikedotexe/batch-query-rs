# Batch query CosmWasm

With JSON RPC, you can batch request payloads in a single POST. While there are helpful abstractions available, to have full control we'll use `reqwest`, CronCat types, and encoding libraries to send the request.

When the response comes back, we use enums to essentially handle the response and turn it into the proper type.

## Usage

    cargo run

Your terminal will show debug logs, the final three of which are the responses from calling the `config` query method on these three CronCat contracts:

- Manager
- Tasks
- Agents
