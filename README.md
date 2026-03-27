# Rhizome

A print management layer for the Bambu A1 printer, supplying basic print functions as well as spool management, queuing, and print farm capabilities.

## Expected features

Expect for these features to be implemented in order of listing here.

- [x] Basic print/control functions
  - [x] Query status (HTTP req & websocket)
  - [x] Start, pause, resume, stop print
  - [x] Set print speed
  - [x] Set nozzle/bed temperature
  - [x] Set fan speed
  - [x] Set led state
  - [x] Run arbitrary g-code
- [x] Printer/Server configuration (set printer ip, serial, access code)
- [ ] Printer file management
- [ ] Authentication / API keys
- [ ] Queuing
  - [ ] Scheduling prints
  - [ ] Automatic plate clearing via Innocube swapmod (ideally, I'd support more tooling in the future)
- [ ] Printer Telemetry/History
- [ ] Spool management
  - [ ] AMS/AMS lite support (I don't have one or expect to get one anytime soon, project fund or code contribution needed for this)
- [ ] Client => Server commands over websocket
- [ ] Print profiles
- [ ] Webhooks

## Non-features

This tool won't aim to have these features, this list may grow over time. In many cases, these features would be nice to have but are outside the scope of this project.

- [ ] Cloud slicing
  - Side note: this is a really nice feature to have but a finicky idea at best, may revisit

## License

This project is covered under the MIT license. See [`LICENSE`](./LICENSE) for further details.
