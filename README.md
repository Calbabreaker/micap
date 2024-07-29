# Micap

Motion capture using IMUs

## App

To run the app:

```
cd app/
pnpm install
pnpm tauri dev
```

## Server

To run the standalone server:

```
cd server/
cargo run
```

## Firmware

To build and upload the firmware:

```
cd firmware/
pio run -t upload
```
