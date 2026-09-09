# Slint Rust Template

A template for a Rust application that's using [Slint](https://slint.rs/) for the user interface.

## About

This template helps you get started developing a Rust application with Slint as toolkit
for the user interface. It demonstrates the integration between the `.slint` UI markup and
Rust code, how to react to callbacks, get and set properties, and use basic widgets.

## Usage

1. Install Rust by following its [getting-started guide](https://www.rust-lang.org/learn/get-started).
   Once this is done, you should have the `rustc` compiler and the `cargo` build system installed in your `PATH`.
2. Download and extract the [ZIP archive of this repository](https://github.com/slint-ui/slint-rust-template/archive/refs/heads/main.zip).
3. Rename the extracted directory and change into it:
    ```
    mv slint-rust-template-main my-project
    cd my-project    
    ```
4. Build with `cargo`:
    ```
    cargo build
    ```
5. Run the application binary:
    ```
    cargo run
    ```

We recommend using an IDE for development, along with our [LSP-based IDE integration for `.slint` files](https://github.com/slint-ui/slint/blob/master/tools/lsp/README.md). You can also load this project directly in [Visual Studio Code](https://code.visualstudio.com) and install our [Slint extension](https://marketplace.visualstudio.com/items?itemName=Slint.slint).

## Next Steps

We hope that this template helps you get started, and that you enjoy exploring making user interfaces with Slint. To learn more
about the Slint APIs and the `.slint` markup language, check out our [online documentation](https://slint.dev/docs).

Don't forget to edit this readme to replace it by yours, and edit the `name =` field in `Cargo.toml` to match the name of your
project.

## Lyra CST3530 touch

The `board-kms` build uses Slint's existing libinput backend for input.
`src/touch.rs` identifies `Hynitron CST3530 Touchscreen` and applies the inverse
of the dashboard's 270-degree rendering rotation: normalized portrait `(u, v)`
becomes landscape `(1-v, u)`. Calibration is local to the application; the
kernel and `touch-test` retain native portrait coordinates. If the display
rotation changes, update this calibration along with `SLINT_KMS_ROTATION`.

A visual-only overlay shows pink rings with white centers for active contacts,
using the same calibrated libinput coordinates delivered to Slint. Complete
frames are coalesced into the latest positions and drawn at most every 40 ms
(25 Hz). Unchanged positions do not redraw. Touch-up, cancellation and device
removal clear the dots on the next scheduled update, even when input goes idle.
Slint continues receiving input events at their original rate. It does not intercept input destined for controls. Set the window's
`show-touch-indicators` property to `false` to hide the diagnostic overlay.
The hook records up to ten contacts; widget gesture handling remains Slint's.

From `../rk3506`, `./lyra-build obd2` rebuilds the application and deploys it to
`/mnt/sdcard/obd2`, then restarts the dashboard. No firmware reflash is needed.
The coordinate and contact-state checks can also run on the host:

```sh
rustc --edition 2024 --test src/touch_state.rs -o /tmp/touch-state-test
/tmp/touch-state-test
```
