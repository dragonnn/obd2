# nRF Manager architecture proposal

This document proposes the architecture; it does not describe an implemented
system. Diagrams are embedded SVG files in [docs/diagrams](docs/diagrams),
alongside their editable Graphviz `.dot` sources. They can also be opened
directly in an SVG viewer or browser.

## State machine boundaries

Use **two `hsmc` machines running in separate Embassy tasks**:

- **Vehicle supervisor:** decides whether the system is driving, observing a
  parked car, charging, or sleeping.
- **Lyra controller:** handles power, boot verification, UART health, retries,
  and graceful shutdown.

CAN, UART, e-ink and radio have their own tasks. They do not all need state
machines. The car can remain in `Driving` while Lyra moves through booting,
running and recovery.

The local `hsmc` supports hierarchical states but has no parallel regions. Its
[semantics document](<crates/hsmc/docs/002. hsmc-semantics-formal.md>) recommends
separate machines connected by channels for independent lifecycles.

## Vehicle supervisor

State hierarchy:

![Vehicle supervisor state hierarchy](docs/diagrams/vehicle-hierarchy.svg)

Flow within each vehicle mode:

![Vehicle supervisor transitions and ignition handling](docs/diagrams/vehicle-flow.svg)

`IgnitionOff` handles ignition-on for all its children. Charging, sleeping and
waiting for shutdown therefore share the same return path.

`LyraManaged` does not mirror the Lyra controller's boot, online and retry
states. It means the vehicle supervisor has requested Lyra ON and delegated
that lifecycle to the Lyra controller.

| Vehicle state | Lyra | nRF CAN activity | Display |
| --- | --- | --- | --- |
| Driving | Requested on; controller maintains it | Disabled, including during Lyra recovery | PID data received from Lyra |
| WaitingForLyraOff | Graceful shutdown in progress | Local polling disabled until handover | Last values, with their age |
| Observing | Off | Periodic OBC and car open/closed checks | Parked/charging detection status |
| Charging | Off | Periodic OBC and BMS reads | Charging information |
| ParkedSleep | Off | Sparse, bounded probe windows | Retained image; refresh when needed |

`ParkedSleep` is the name for the requested final "shutdown" mode because the
nRF must still wake for ignition, CAN probes and modem communication. Select a
hardware sleep mode that supports the required wake sources.

### Observation window

Store timestamps in the supervisor context:

- If the car is already closed at ignition-off, start the approximately
  five-minute observation window then.
- Otherwise, anchor that window to the subsequent closure.
- Use a longer, bounded policy while the car is open or its closure state is
  unknown.
- Repeated "still closed" responses must not restart the timer.

An OBC response enters `Charging`. BMS availability is tracked separately.
Confirm charging loss across multiple failed polls rather than treating one
missing response as the end of charging. Preserve the difference between
"not charging" and "data unavailable" in telemetry.

## Lyra controller

The Lyra controller owns the power pins. Other tasks request a desired state
and receive status updates; they never toggle Lyra power themselves.

![Lyra power, boot, recovery and shutdown lifecycle](docs/diagrams/lyra-controller.svg)

### Boot, health and recovery

- **Boot readiness means an application handshake.** Linux boot output or
  arbitrary UART bytes do not establish that the PID service can answer
  requests.
- **Health uses valid protocol traffic and correlated replies.** Track PID
  freshness separately so that a responsive UART with stale vehicle data is
  visible.
- **Retries remain periodic while ignition is on.** Use a minimum power-off
  interval and a capped retry delay to avoid rapid cycling.
- **Requested OFF supersedes boot and recovery.** In `RetryWait`, power is
  already off, so the controller can enter `PowerOff` directly.

Boot deadlines, health intervals, missed-reply thresholds and retry delays are
configuration values to tune against actual Lyra behaviour.

### Graceful shutdown

1. On ignition-off, request Lyra OFF and record the absolute deadline:
   `ignition_fell_at + 30 seconds`.
2. Send a shutdown request and wait for its acknowledgement.
3. After acknowledgement, send a ping every second.
4. After the configured missed-reply threshold, cut power.
5. Cut power at the absolute deadline even if the acknowledgement never
   arrives or Lyra continues responding.

ACKs, pings and repeated OFF requests must not extend the deadline. Account
for command delivery and the power-switching sequence when enforcing it.

Define ACK as **shutdown accepted**. The Lyra ping responder must remain alive
until removing power is acceptable. Stopping the UART service early must not
be mistaken for completed shutdown. Use a small configurable missed-reply
threshold rather than cutting power after one lost reply, while remaining
within the overall deadline.

If ignition returns during shutdown, update the desired state to ON. Once
shutdown has begun, complete the shutdown/cut and then boot again. Do not
assume Linux can reverse a shutdown already in progress. The vehicle
supervisor can already be in `Driving` while the Lyra controller finishes
this sequence.

## Tasks and ownership

| Component | Responsibility | Use `hsmc`? |
| --- | --- | --- |
| Ignition monitor | Read initial level, debounce edges, preserve timestamps | No |
| Vehicle supervisor | Vehicle modes, observation deadlines, CAN policy and handover | Yes |
| Lyra controller | Power lifecycle, health deadlines, shutdown and retry policy | Yes |
| UART service | Persistent RX parser, serialized TX, request IDs and reply routing | Usually no |
| CAN service | Own controllers; execute polling profiles and bounded PID transactions | Usually no |
| Telemetry store | Merge values, track freshness/source, publish snapshots | No |
| Display task | Render latest snapshot, rate-limit refreshes, handle display busy | No initially |
| Modem/radio task | Own IEEE 802.15.4, exchange telemetry and incoming information | Optional later |

Each hardware resource has one owner. The UART service may use separate RX
and TX tasks, each owning its half of the transport.

### CAN handover

On ignition-on:

1. Stop scheduling local CAN requests.
2. Finish or abort the current transaction within a bounded time.
3. Report `CanQuiescent`.
4. Allow the Lyra controller to power up Lyra.

On ignition-off, enable local polling only after Lyra power has been cut.
There is no nRF CAN fallback during driving, including while Lyra boots or
recovers.

## Telemetry and modem data flow

![Telemetry, display and bidirectional modem data flow](docs/diagrams/telemetry-flow.svg)

Each stored PID carries at least:

- Value.
- Source: Lyra or local CAN.
- Observation time.
- Validity.

When ownership changes from Lyra to local CAN, old values remain identifiable
as stale. The display and modem consume the same normalized snapshot.

For information received from the modem, update local state immediately and
forward it when Lyra's application is ready. While Lyra is booting, retain
replaceable information as its latest value; give commands a bounded queue
and expiry. Old commands must not accumulate indefinitely while Lyra is off.

Radio reception windows can be adjusted by vehicle mode to balance delivery
latency and parking power consumption. The modem and nRF need an agreed
receive-window policy if immediate reception is not maintained.

## Inter-task communication

Use two communication patterns:

- **Bounded channels for commands and completion events:** `CanQuiescent`,
  `LyraPoweredOff`, shutdown ACKs and transaction results. Give each channel
  an intended consumer.
- **`Watch` for latest snapshots and current status:** display and radio can
  skip intermediate telemetry updates.

Embassy's [`Watch`](https://docs.embassy.dev/embassy-sync/0.8.0/default/watch/index.html)
supports multiple receivers and overwrites the previous value. Ordinary
[`Channel`](https://docs.embassy.dev/embassy-sync/0.8.0/default/channel/index.html)
receivers compete for messages; a single channel is not a broadcast bus.

Keep raw PID traffic out of the supervisor's event queue. Send it sparse
facts such as `ObcDetected`, `ChargingLost` and `CarClosedChanged`. Preserve
ignition level and edge timestamps, and deliver ignition changes reliably.

Tag asynchronous work with a generation number. A late OBC reply from an old
parked polling cycle must not switch the system into charging after ignition
has risen. The same principle applies to stale UART replies after a Lyra
restart and results from cancelled polling profiles.

## Using `hsmc`

- Put shared transitions on parent states, such as ignition-on on
  `IgnitionOff` and the overall shutdown deadline on `Stopping`.
- Use `on(after ...)` for fixed state-scoped deadlines and `on(every ...)`
  for periodic policy checks. For deadlines anchored to an external event,
  retain the absolute timestamp and account for elapsed time.
- The local crate has no transition guards. For context-dependent decisions,
  have an action evaluate the context and emit a typed follow-up event that
  selects the next state.
- Keep chart actions short. Send commands and handle completion events
  instead of awaiting an entire boot, CAN exchange or display refresh inside
  an action.
- Use `during:` for cancellation-safe waits. The local implementation can
  drop these futures when events or timers arrive, so persistent UART parsing
  and multi-step hardware transactions belong in their owning tasks. See the
  [cancellation contract](crates/hsmc/README.md#cancel-safety-contract).
- Keep long peripheral initialization and recovery outside the ignition
  supervision startup path.
- Do not terminate the vehicle chart when entering `ParkedSleep`; it must
  remain able to process wakeups and return to driving or charging.

Start with the two machines described here. Add another only when a
subsystem develops its own substantial lifecycle, such as radio
association/retry/sleep states or display recovery.

## Hardware scheduling considerations

- Share SPI only around actual transactions. An e-ink refresh must release
  the bus while waiting for the panel, allowing CAN access.
- Rate-limit display refreshes and coalesce pending updates to the newest
  snapshot.
- Validate parked probe behaviour on the car. Probe frequency affects both
  detection latency and whether ECUs are allowed to stay asleep.
- Keep all CAN transactions and receive windows bounded so that ignition
  changes remain responsive.
