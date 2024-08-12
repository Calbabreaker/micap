# Micap

Motion capture using IMUs

## App

To run the app:

```sh
cd app/
pnpm install
pnpm tauri dev
```

## Server

Alternatively, run the standalone server seperately to speed up compilation:

```sh
cd server/
cargo run
```

Svelte app:

```sh
cd app/
pnpm dev
```

## Firmware

To build and upload the firmware:

```sh
cd firmware/
pio run -t upload
```

## Dummy Client

To run the dummy UDP client for testing/benching (without a physical device):

```sh
cd dummy-client/
cargo run
```
