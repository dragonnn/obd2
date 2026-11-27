use super::*;
use core::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};
use std::sync::{Mutex, MutexGuard};

use crate::board::LyraPower;

static CLOCK_LOCK: Mutex<()> = Mutex::new(());

fn reset_clock() -> MutexGuard<'static, ()> {
    let lock = CLOCK_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    embassy_time::MockDriver::get().reset();
    lock
}

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("unexpected asynchronous wait"),
    }
}

fn setup() -> (Lyra, LyraPower, RpcSender) {
    let power = LyraPower::default();
    let rpc = RpcSender::default();
    let mut machine = Lyra::new_local(LyraContext::new(power.clone(), rpc.clone()));
    ready(machine.step(Duration::ZERO));
    (machine, power, rpc)
}

fn dispatch(machine: &mut Lyra, event: LyraEvent) {
    ready(machine.dispatch(event)).unwrap();
}

fn tick(machine: &mut Lyra, seconds: u64) {
    for _ in 0..seconds {
        embassy_time::MockDriver::get().advance(TimeDuration::from_secs(1));
        let mut next = ready(machine.step(Duration::from_secs(1)));
        // Match run(): process due timers and drain action-emitted events
        // before advancing the clock again.
        while !machine.queue.is_empty() || next == Some(Duration::ZERO) {
            next = ready(machine.step(Duration::ZERO));
        }
        // The local step driver supplies events normally produced by during activities.
        if matches!(
            machine.current_state(),
            LyraState::MinimumOffTime | LyraState::Retry
        ) && Instant::now() >= machine.context().power_on_allowed_at
        {
            dispatch(machine, LyraEvent::OffTimeElapsed);
        }
    }
}

fn online(machine: &mut Lyra, rpc: &RpcSender) {
    dispatch(machine, LyraEvent::PowerOn);
    let id = rpc.last::<EchoProbeTopic>();
    dispatch(machine, LyraEvent::EchoReply { id });
    assert_eq!(machine.current_state(), LyraState::Online);
}

#[test]
fn boot_requires_a_matching_probe_and_duplicate_power_on_does_not_reboot() {
    let _clock = reset_clock();
    let (mut machine, power, rpc) = setup();
    assert_eq!(machine.current_state(), LyraState::ReadyToPowerOn);
    assert!(!power.is_on());
    dispatch(&mut machine, LyraEvent::PowerOn);
    assert!(power.is_on());
    assert_eq!(machine.current_state(), LyraState::Starting);
    let id = rpc.last::<EchoProbeTopic>();
    dispatch(&mut machine, LyraEvent::EchoReply { id: id + 100 });
    assert_eq!(machine.current_state(), LyraState::Starting);
    dispatch(&mut machine, LyraEvent::EchoReply { id });
    assert_eq!(machine.current_state(), LyraState::Online);
    let changes = power.0.lock().unwrap().len();
    dispatch(&mut machine, LyraEvent::PowerOn);
    assert_eq!(power.0.lock().unwrap().len(), changes);
}

#[test]
fn boot_timeout_cuts_power_then_retries_with_fresh_ids() {
    let _clock = reset_clock();
    let (mut machine, power, rpc) = setup();
    dispatch(&mut machine, LyraEvent::PowerOn);
    let old_id = rpc.last::<EchoProbeTopic>();
    tick(&mut machine, 60);
    assert_eq!(machine.current_state(), LyraState::Retry);
    assert!(!power.is_on());
    let probes = rpc.count::<EchoProbeTopic>();
    tick(&mut machine, 4);
    assert_eq!(machine.current_state(), LyraState::Retry);
    assert!(!power.is_on());
    assert_eq!(rpc.count::<EchoProbeTopic>(), probes);
    tick(&mut machine, 1);
    assert_eq!(machine.current_state(), LyraState::Starting);
    assert!(power.is_on());
    assert!(rpc.last::<EchoProbeTopic>() > old_id);
    dispatch(&mut machine, LyraEvent::EchoReply { id: old_id });
    assert_eq!(machine.current_state(), LyraState::Starting);
}

#[test]
fn valid_echo_resets_missed_count_and_three_misses_trigger_recovery() {
    let _clock = reset_clock();
    let (mut machine, power, rpc) = setup();
    online(&mut machine, &rpc);
    tick(&mut machine, 2);
    let id = rpc.last::<EchoProbeTopic>();
    dispatch(&mut machine, LyraEvent::EchoReply { id });
    tick(&mut machine, 1);
    assert_eq!(machine.current_state(), LyraState::Online);
    tick(&mut machine, 3);
    assert_eq!(machine.current_state(), LyraState::Retry);
    assert!(!power.is_on());
    let shutdown_requests = rpc.count::<ShutdownRequestTopic>();
    tick(&mut machine, 2);
    let changes = power.0.lock().unwrap().len();
    dispatch(&mut machine, LyraEvent::PowerOff);
    assert_eq!(machine.current_state(), LyraState::MinimumOffTime);
    assert_eq!(rpc.count::<ShutdownRequestTopic>(), shutdown_requests);
    tick(&mut machine, 3);
    assert_eq!(machine.current_state(), LyraState::ReadyToPowerOn);
    assert!(!power.is_on());
    assert_eq!(power.0.lock().unwrap().len(), changes);
}

#[test]
fn shutdown_cuts_power_five_seconds_after_the_third_missed_echo() {
    let _clock = reset_clock();
    let (mut machine, power, rpc) = setup();
    online(&mut machine, &rpc);
    dispatch(&mut machine, LyraEvent::PowerOff);
    let id = rpc.last::<ShutdownRequestTopic>();
    assert!(power.is_on());
    assert_eq!(machine.current_state(), LyraState::AwaitShutdownAck);
    dispatch(&mut machine, LyraEvent::ShutdownAck { id: id + 1 });
    assert_eq!(machine.current_state(), LyraState::AwaitShutdownAck);
    tick(&mut machine, 3);
    assert!(power.is_on(), "silence before ACK must not cut power early");
    assert_eq!(rpc.last::<ShutdownRequestTopic>(), id);
    dispatch(&mut machine, LyraEvent::ShutdownAck { id });
    assert_eq!(machine.current_state(), LyraState::PingUntilSilent);
    tick(&mut machine, 2);
    assert!(power.is_on());
    dispatch(&mut machine, LyraEvent::ShutdownAck { id });
    tick(&mut machine, 1);
    assert_eq!(machine.current_state(), LyraState::WaitBeforePowerOff);
    let probes = rpc.count::<EchoProbeTopic>();
    tick(&mut machine, 4);
    assert_eq!(machine.current_state(), LyraState::WaitBeforePowerOff);
    assert!(power.is_on());
    tick(&mut machine, 1);
    assert_eq!(machine.current_state(), LyraState::MinimumOffTime);
    assert!(!power.is_on());
    assert_eq!(rpc.count::<EchoProbeTopic>(), probes);
    tick(&mut machine, 8);
    assert_eq!(machine.current_state(), LyraState::ReadyToPowerOn);
    assert!(!power.is_on());
}

#[test]
fn power_on_during_shutdown_waits_five_seconds_after_the_power_cut() {
    let _clock = reset_clock();
    let (mut machine, power, rpc) = setup();
    online(&mut machine, &rpc);
    dispatch(&mut machine, LyraEvent::PowerOff);
    let id = rpc.last::<ShutdownRequestTopic>();
    dispatch(&mut machine, LyraEvent::PowerOn);
    assert_eq!(machine.current_state(), LyraState::AwaitShutdownAck);
    dispatch(&mut machine, LyraEvent::ShutdownAck { id });
    let changes = power.0.lock().unwrap().len();
    tick(&mut machine, 3);
    assert_eq!(machine.current_state(), LyraState::WaitBeforePowerOff);
    dispatch(&mut machine, LyraEvent::PowerOn);
    tick(&mut machine, 4);
    assert_eq!(power.0.lock().unwrap().len(), changes);
    tick(&mut machine, 1);
    assert_eq!(machine.current_state(), LyraState::MinimumOffTime);
    assert!(!power.is_on());
    tick(&mut machine, 4);
    assert!(!power.is_on());
    tick(&mut machine, 1);
    assert_eq!(machine.current_state(), LyraState::Starting);
    assert!(power.is_on());
    assert_eq!(&power.0.lock().unwrap()[changes..], &[false, true]);
}

#[test]
fn ignition_off_during_minimum_off_time_cancels_the_pending_start() {
    let _clock = reset_clock();
    let (mut machine, power, rpc) = setup();
    online(&mut machine, &rpc);
    dispatch(&mut machine, LyraEvent::PowerOff);
    let id = rpc.last::<ShutdownRequestTopic>();
    dispatch(&mut machine, LyraEvent::ShutdownAck { id });
    tick(&mut machine, 8);
    assert_eq!(machine.current_state(), LyraState::MinimumOffTime);
    dispatch(&mut machine, LyraEvent::PowerOn);
    assert!(!power.is_on());
    tick(&mut machine, 4);
    dispatch(&mut machine, LyraEvent::PowerOff);
    tick(&mut machine, 1);
    assert_eq!(machine.current_state(), LyraState::ReadyToPowerOn);
    assert!(!power.is_on());
}

fn check_absolute_shutdown_deadline(ack_at_secs: Option<u64>, power_on_pending: bool) {
    let latch_time = TimeDuration::from_millis(15);
    let power = LyraPower(Default::default(), Some(latch_time));
    let rpc = RpcSender::default();
    let channel = Box::leak(Box::new(
        Channel::<CriticalSectionRawMutex, LyraEvent, QN>::new(),
    ));
    let mut machine = Lyra::new(LyraContext::new(power.clone(), rpc.clone()), channel);
    let mut running = pin!(machine.run());
    let mut cx = Context::from_waker(Waker::noop());
    assert!(running.as_mut().poll(&mut cx).is_pending());
    channel.try_send(LyraEvent::PowerOn).unwrap();
    assert!(power.0.lock().unwrap().is_empty());
    embassy_time::MockDriver::get().advance(latch_time);
    assert!(running.as_mut().poll(&mut cx).is_pending());
    assert_eq!(&*power.0.lock().unwrap(), &[false, true]);
    channel.try_send(LyraEvent::PowerOff).unwrap();
    assert!(running.as_mut().poll(&mut cx).is_pending());
    let id = rpc.last::<ShutdownRequestTopic>();
    for seconds in 1..=29 {
        embassy_time::MockDriver::get().advance(TimeDuration::from_secs(1));
        channel.try_send(LyraEvent::PowerOff).unwrap();
        channel.try_send(LyraEvent::EchoReply { id: 0 }).unwrap();
        assert!(running.as_mut().poll(&mut cx).is_pending());
        assert!(power.is_on());
        if ack_at_secs == Some(seconds) {
            channel.try_send(LyraEvent::ShutdownAck { id }).unwrap();
            assert!(running.as_mut().poll(&mut cx).is_pending());
        }
    }
    if power_on_pending {
        channel.try_send(LyraEvent::PowerOn).unwrap();
        assert!(running.as_mut().poll(&mut cx).is_pending());
    }
    embassy_time::MockDriver::get().advance(TimeDuration::from_millis(979));
    assert!(running.as_mut().poll(&mut cx).is_pending());
    assert!(power.is_on());
    let changes = power.0.lock().unwrap().len();
    embassy_time::MockDriver::get().advance(TimeDuration::from_millis(1));
    assert!(running.as_mut().poll(&mut cx).is_pending());
    assert!(power.is_on());
    embassy_time::MockDriver::get().advance(latch_time);
    assert!(running.as_mut().poll(&mut cx).is_pending());
    assert_eq!(&power.0.lock().unwrap()[changes..], &[false]);
    embassy_time::MockDriver::get().advance(TimeDuration::from_millis(4_999));
    assert!(running.as_mut().poll(&mut cx).is_pending());
    assert!(!power.is_on());
    embassy_time::MockDriver::get().advance(TimeDuration::from_millis(1));
    assert!(running.as_mut().poll(&mut cx).is_pending());
    assert_eq!(power.is_on(), power_on_pending);
}

#[test]
fn absolute_shutdown_deadline_survives_duplicates_and_late_ack() {
    let _clock = reset_clock();
    check_absolute_shutdown_deadline(Some(29), false);
}

#[test]
fn absolute_shutdown_deadline_can_end_the_five_second_delay_early() {
    let _clock = reset_clock();
    check_absolute_shutdown_deadline(Some(26), false);
}

#[test]
fn absolute_shutdown_deadline_cuts_power_without_any_ack() {
    let _clock = reset_clock();
    check_absolute_shutdown_deadline(None, false);
}

#[test]
fn deferred_power_on_waits_five_seconds_after_the_deadline_cuts_power() {
    let _clock = reset_clock();
    check_absolute_shutdown_deadline(None, true);
}
